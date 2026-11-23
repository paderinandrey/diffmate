use super::{AGENT_STOP_GRACE, GroupSignaller, RustixSignaller, StopSignal, drained, stop_group};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::Instant;

struct Fake {
    started: Instant,
    empties_at: Option<Duration>,
    sent: Mutex<Vec<(u32, StopSignal, Duration)>>,
}

impl Fake {
    fn new(empties_at: Option<Duration>) -> Fake {
        Fake {
            started: Instant::now(),
            empties_at,
            sent: Mutex::new(Vec::new()),
        }
    }

    fn sent(&self) -> Vec<(u32, StopSignal, Duration)> {
        self.sent.lock().unwrap().clone()
    }
}

impl GroupSignaller for Fake {
    fn signal(&self, pgid: u32, signal: StopSignal) -> io::Result<()> {
        let at = self.started.elapsed();
        self.sent.lock().unwrap().push((pgid, signal, at));
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
    let fake = Fake::new(Some(Duration::ZERO));

    stop_group(&fake, 4242, AGENT_STOP_GRACE).await.unwrap();

    assert_eq!(fake.sent(), [(4242, StopSignal::Term, Duration::ZERO)]);
}

#[tokio::test(start_paused = true)]
async fn kill_follows_exactly_one_grace_period_after_term() {
    let fake = Fake::new(None);

    stop_group(&fake, 4242, AGENT_STOP_GRACE).await.unwrap();

    let expected = [
        (4242, StopSignal::Term, Duration::ZERO),
        (4242, StopSignal::Kill, AGENT_STOP_GRACE),
    ];
    assert_eq!(fake.sent(), expected);
}

#[tokio::test(start_paused = true)]
async fn stopping_returns_once_the_rest_of_the_group_exits() {
    let lingers = Duration::from_secs(1);
    let fake = Fake::new(Some(lingers));

    stop_group(&fake, 4242, AGENT_STOP_GRACE).await.unwrap();

    assert_eq!(fake.sent(), [(4242, StopSignal::Term, Duration::ZERO)]);
    assert_eq!(fake.started.elapsed(), lingers);
}

#[tokio::test(start_paused = true)]
async fn the_guard_must_wait_for_the_group_even_after_a_second_grace() {
    let gone = AGENT_STOP_GRACE * 3;
    let fake = Fake::new(Some(gone));

    stop_group(&fake, 4242, AGENT_STOP_GRACE).await.unwrap();
    drained(&fake, 4242).await.unwrap();

    assert_eq!(fake.sent().len(), 2);
    assert_eq!(fake.started.elapsed(), gone);
}

#[tokio::test(start_paused = true)]
async fn group_ids_that_could_hit_everything_are_rejected() {
    let fake = Fake::new(Some(Duration::ZERO));

    for pgid in [0, 1, u32::MAX] {
        let refused = stop_group(&fake, pgid, AGENT_STOP_GRACE).await;
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
    let mut child = Command::new("sleep").arg("300").spawn().unwrap();
    let pgid = child.id();

    assert!(!RustixSignaller.alive(pgid).unwrap());
    RustixSignaller.signal(pgid, StopSignal::Term).unwrap();
    RustixSignaller.signal(pgid, StopSignal::Kill).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn stopping_an_owned_group_with_only_its_unreaped_leader_is_not_an_error() {
    use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};

    let mut child = Command::new("true").process_group(0).spawn().unwrap();
    let leader = Pid::from_child(&child);
    let options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
    waitid(WaitId::Pid(leader), options).unwrap().unwrap();

    let result = RustixSignaller.signal(child.id(), StopSignal::Kill);
    child.wait().unwrap();
    result.unwrap();
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
