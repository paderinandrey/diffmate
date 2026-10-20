use super::{AGENT_STOP_GRACE, GroupSignaller, RustixSignaller, StopSignal, stop_group};
use std::future::Future;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::Instant;

struct Fake {
    started: Instant,
    release_on: StopSignal,
    released: Arc<Notify>,
    sent: Mutex<Vec<(u32, StopSignal, Duration)>>,
}

impl Fake {
    fn new(release_on: StopSignal) -> Fake {
        Fake {
            started: Instant::now(),
            release_on,
            released: Arc::new(Notify::new()),
            sent: Mutex::new(Vec::new()),
        }
    }

    fn wait(&self) -> impl Future<Output = ()> + use<> {
        let released = Arc::clone(&self.released);
        async move { released.notified().await }
    }

    fn sent(&self) -> Vec<(u32, StopSignal, Duration)> {
        self.sent.lock().unwrap().clone()
    }
}

impl GroupSignaller for Fake {
    fn signal(&self, pgid: u32, signal: StopSignal) -> io::Result<()> {
        let at = self.started.elapsed();
        self.sent.lock().unwrap().push((pgid, signal, at));
        if signal == self.release_on {
            self.released.notify_one();
        }
        Ok(())
    }
}

#[tokio::test(start_paused = true)]
async fn a_child_that_exits_after_term_is_never_killed() {
    let fake = Fake::new(StopSignal::Term);

    stop_group(&fake, 4242, AGENT_STOP_GRACE, fake.wait())
        .await
        .unwrap();

    assert_eq!(fake.sent(), [(4242, StopSignal::Term, Duration::ZERO)]);
}

#[tokio::test(start_paused = true)]
async fn kill_follows_exactly_one_grace_period_after_term() {
    let fake = Fake::new(StopSignal::Kill);

    stop_group(&fake, 4242, AGENT_STOP_GRACE, fake.wait())
        .await
        .unwrap();

    let expected = [
        (4242, StopSignal::Term, Duration::ZERO),
        (4242, StopSignal::Kill, AGENT_STOP_GRACE),
    ];
    assert_eq!(fake.sent(), expected);
}

#[tokio::test(start_paused = true)]
async fn group_ids_that_could_hit_everything_are_rejected() {
    let fake = Fake::new(StopSignal::Term);

    for pgid in [0, 1, u32::MAX] {
        let refused = stop_group(&fake, pgid, AGENT_STOP_GRACE, async {}).await;
        assert_eq!(refused.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let direct = RustixSignaller.signal(pgid, StopSignal::Kill);
        assert_eq!(direct.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    assert_eq!(fake.sent().len(), 0);
}

#[test]
fn an_empty_group_is_not_an_error() {
    let mut gone = Command::new("true");
    gone.process_group(0);
    let mut child = gone.spawn().unwrap();
    let pgid = child.id();
    child.wait().unwrap();

    RustixSignaller.signal(pgid, StopSignal::Term).unwrap();
    RustixSignaller.signal(pgid, StopSignal::Kill).unwrap();
}
