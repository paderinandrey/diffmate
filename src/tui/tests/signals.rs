use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::test(start_paused = true)]
async fn sigterm_drops_in_flight_work_and_returns() {
    tokio::time::resume();
    let mut signals = crate::review::signals::Signals::new().unwrap();
    let dropped = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&dropped);
    let work = async move {
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }
        let _guard = Guard(flag);
        std::future::pending::<Result<(), crate::error::Error>>().await
    };
    let mut task = std::pin::pin!(super::super::until_shutdown(&mut signals, work));
    tokio::select! {
        biased;
        result = &mut task => panic!("stopped before the signal: {result:?}"),
        () = tokio::task::yield_now() => {}
    }
    let signal = rustix::process::Signal::TERM;
    rustix::process::kill_process(rustix::process::getpid(), signal).unwrap();
    let result = tokio::select! {
        biased;
        result = &mut task => result,
        () = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            panic!("SIGTERM did not leave the tui loop")
        }
    };
    result.unwrap();
    assert!(dropped.load(Ordering::Relaxed));
}
