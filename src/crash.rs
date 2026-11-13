macro_rules! crash_point {
    ($name:literal) => {
        #[cfg(feature = "crash-points")]
        crate::crash::env_point($name).await;
    };
}

pub(crate) use crash_point;

pub(crate) trait Probe: Send + Sync {
    fn matches(&self, point: &str) -> bool {
        let _ = (self, point);
        false
    }

    fn park(&self, point: &str) {
        let _ = (self, point);
    }
}

pub(crate) struct Off;

impl Probe for Off {}

pub(crate) async fn at(point: &'static str, probe: &dyn Probe) {
    if !armed(point, probe) {
        return;
    }
    report(point);
    probe.park(point);
    std::future::pending::<()>().await;
}

#[cfg(feature = "crash-points")]
pub(crate) async fn env_point(point: &'static str) {
    at(point, &Off).await;
}

fn armed(point: &str, probe: &dyn Probe) -> bool {
    #[cfg(feature = "crash-points")]
    {
        env_hit(point) || probe.matches(point)
    }
    #[cfg(not(feature = "crash-points"))]
    {
        let _ = (point, probe);
        false
    }
}

#[cfg(feature = "crash-points")]
fn env_hit(point: &str) -> bool {
    std::env::var_os("DIFFMATE_CRASH_POINT").as_deref() == Some(std::ffi::OsStr::new(point))
}

fn report(point: &str) {
    use std::io::Write;

    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "crash point reached: {point}");
    let _ = stderr.flush();
}

#[cfg(all(feature = "crash-points", test))]
pub(crate) struct Hold {
    code: std::sync::atomic::AtomicU8,
    parked: std::sync::atomic::AtomicU8,
}

#[cfg(all(feature = "crash-points", test))]
impl Hold {
    pub(crate) fn point(name: &str) -> Self {
        Self {
            code: std::sync::atomic::AtomicU8::new(code_of(name)),
            parked: std::sync::atomic::AtomicU8::new(0),
        }
    }

    pub(crate) fn parked(&self, name: &str) -> bool {
        let code = code_of(name);
        code != 0 && self.parked.load(std::sync::atomic::Ordering::SeqCst) == code
    }
}

#[cfg(all(feature = "crash-points", test))]
impl Probe for Hold {
    fn matches(&self, point: &str) -> bool {
        let code = self.code.load(std::sync::atomic::Ordering::SeqCst);
        code != 0 && code == code_of(point)
    }

    fn park(&self, point: &str) {
        let code = code_of(point);
        if code != 0 {
            self.parked.store(code, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

#[cfg(all(feature = "crash-points", test))]
impl Drop for Hold {
    fn drop(&mut self) {
        self.code.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(all(feature = "crash-points", test))]
fn code_of(point: &str) -> u8 {
    match point {
        "publish-before-sending" => 1,
        "publish-after-sending" => 2,
        _ => 0,
    }
}
