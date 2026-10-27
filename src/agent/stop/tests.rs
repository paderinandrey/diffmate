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
    empties_at: Option<Duration>,
    released: Arc<Notify>,
    sent: Mutex<Vec<(u32, StopSignal, Duration)>>,
}

impl Fake {
    fn new(release_on: StopSignal, empties_at: Option<Duration>) -> Fake {
        Fake {
            started: Instant::now(),
            release_on,
            empties_at,
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

    fn alive(&self, _pgid: u32) -> io::Result<bool> {
        let killed = || self.sent().iter().any(|sent| sent.1 == StopSignal::Kill);
        Ok(match self.empties_at {
            Some(at) => self.started.elapsed() < at,
            None => !killed(),
        })
    }
}

#[tokio::test(start_paused = true)]
async fn a_child_that_exits_after_term_is_never_killed() {
    let fake = Fake::new(StopSignal::Term, Some(Duration::ZERO));

    stop_group(&fake, 4242, AGENT_STOP_GRACE, fake.wait())
        .await
        .unwrap();

    assert_eq!(fake.sent(), [(4242, StopSignal::Term, Duration::ZERO)]);
}

#[tokio::test(start_paused = true)]
async fn kill_follows_exactly_one_grace_period_after_term() {
    let fake = Fake::new(StopSignal::Kill, None);

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
async fn a_group_that_outlives_its_leader_is_killed_after_the_grace() {
    let fake = Fake::new(StopSignal::Term, None);

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
async fn stopping_returns_once_the_rest_of_the_group_exits() {
    let lingers = Duration::from_secs(1);
    let fake = Fake::new(StopSignal::Term, Some(lingers));

    stop_group(&fake, 4242, AGENT_STOP_GRACE, fake.wait())
        .await
        .unwrap();

    assert_eq!(fake.sent(), [(4242, StopSignal::Term, Duration::ZERO)]);
    assert_eq!(fake.started.elapsed(), lingers);
}

#[tokio::test(start_paused = true)]
async fn a_group_that_survives_kill_stops_the_wait_after_another_grace() {
    let fake = Fake::new(StopSignal::Term, Some(Duration::MAX));

    stop_group(&fake, 4242, AGENT_STOP_GRACE, fake.wait())
        .await
        .unwrap();

    assert_eq!(fake.sent().len(), 2);
    assert_eq!(fake.started.elapsed(), AGENT_STOP_GRACE * 2);
}

#[tokio::test(start_paused = true)]
async fn group_ids_that_could_hit_everything_are_rejected() {
    let fake = Fake::new(StopSignal::Term, Some(Duration::ZERO));

    for pgid in [0, 1, u32::MAX] {
        let refused = stop_group(&fake, pgid, AGENT_STOP_GRACE, async {}).await;
        assert_eq!(refused.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let direct = RustixSignaller.signal(pgid, StopSignal::Kill);
        assert_eq!(direct.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let probed = RustixSignaller.alive(pgid);
        assert_eq!(probed.unwrap_err().kind(), io::ErrorKind::InvalidInput);
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

    assert!(!RustixSignaller.alive(pgid).unwrap());
    RustixSignaller.signal(pgid, StopSignal::Term).unwrap();
    RustixSignaller.signal(pgid, StopSignal::Kill).unwrap();
}

#[test]
fn a_group_with_a_running_member_is_alive() {
    let mut sleeper = Command::new("sleep");
    sleeper.arg("300").process_group(0);
    let mut child = sleeper.spawn().unwrap();
    let pgid = child.id();

    let alive = RustixSignaller.alive(pgid);
    child.kill().unwrap();
    child.wait().unwrap();

    assert!(alive.unwrap());
}
