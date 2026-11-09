macro_rules! crash_point {
    ($name:literal) => {
        #[cfg(feature = "crash-points")]
        crate::crash::reached($name).await;
    };
}

pub(crate) use crash_point;

#[cfg(feature = "crash-points")]
pub(crate) async fn reached(point: &'static str) {
    use std::io::Write;

    if std::env::var_os("DIFFMATE_CRASH_POINT").as_deref() == Some(std::ffi::OsStr::new(point)) {
        {
            let mut stderr = std::io::stderr().lock();
            let _ = writeln!(stderr, "crash point reached: {point}");
            let _ = stderr.flush();
        }
        std::future::pending::<()>().await;
    }
}
