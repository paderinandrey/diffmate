macro_rules! crash_point {
    ($name:literal) => {
        #[cfg(feature = "crash-points")]
        crate::crash::reached($name).await;
    };
}

pub(crate) use crash_point;

#[cfg(feature = "crash-points")]
mod points {
    use std::sync::atomic::{AtomicU8, Ordering};
    #[cfg(test)]
    use std::sync::{Mutex, PoisonError};

    static CODE: AtomicU8 = AtomicU8::new(0);
    #[cfg(test)]
    static TURN: Mutex<()> = Mutex::new(());

    pub(super) fn matches(point: &str) -> bool {
        let code = CODE.load(Ordering::SeqCst);
        code != 0 && code == code_of(point)
    }

    #[cfg(test)]
    pub(super) fn arm(point: &str) {
        CODE.store(code_of(point), Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(super) fn disarm() {
        CODE.store(0, Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(super) fn lock_turn() -> std::sync::MutexGuard<'static, ()> {
        TURN.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn code_of(point: &str) -> u8 {
        match point {
            "publish-before-sending" => 1,
            "publish-after-sending" => 2,
            _ => 0,
        }
    }
}

#[cfg(feature = "crash-points")]
pub(crate) async fn reached(point: &'static str) {
    use std::io::Write;

    let from_env =
        std::env::var_os("DIFFMATE_CRASH_POINT").as_deref() == Some(std::ffi::OsStr::new(point));
    if from_env || points::matches(point) {
        {
            let mut stderr = std::io::stderr().lock();
            let _ = writeln!(stderr, "crash point reached: {point}");
            let _ = stderr.flush();
        }
        std::future::pending::<()>().await;
    }
}

#[cfg(all(feature = "crash-points", test))]
pub(crate) struct Hold {
    done: Option<tokio::sync::oneshot::Sender<()>>,
}

#[cfg(all(feature = "crash-points", test))]
impl Hold {
    pub(crate) async fn point(name: &'static str) -> Self {
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        tokio::task::spawn_blocking(move || {
            let _turn = points::lock_turn();
            let _ = ready_tx.send(());
            let _ = done_rx.blocking_recv();
        });
        ready_rx.await.expect("crash point lock");
        points::arm(name);
        Self {
            done: Some(done_tx),
        }
    }
}

#[cfg(all(feature = "crash-points", test))]
impl Drop for Hold {
    fn drop(&mut self) {
        points::disarm();
        if let Some(done) = self.done.take() {
            let _ = done.send(());
        }
    }
}
