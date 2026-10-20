use super::http;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

#[derive(Clone)]
pub(super) struct Recorded {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

type Seen = Arc<Mutex<Vec<Recorded>>>;

async fn bind() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    (listener, url)
}

async fn answer(stream: TcpStream, seen: Seen) {
    let mut reader = BufReader::new(stream);
    if let Ok(request) = http::read_request(&mut reader).await {
        let method = request.method.clone();
        seen.lock().unwrap().push(request);
        let _ = http::reply(&mut reader, &method).await;
    }
}

pub(super) struct FakeAnthropic {
    pub url: String,
    seen: Seen,
    accepted: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}

impl FakeAnthropic {
    pub async fn start() -> FakeAnthropic {
        let (listener, url) = bind().await;
        let seen = Seen::default();
        let accepted = Arc::new(AtomicUsize::new(0));
        let (shared, counter) = (Arc::clone(&seen), Arc::clone(&accepted));
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(answer(stream, Arc::clone(&shared)));
            }
        });
        FakeAnthropic {
            url,
            seen,
            accepted,
            task,
        }
    }

    pub fn connections(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }

    pub fn take(&self) -> Vec<Recorded> {
        std::mem::take(&mut *self.seen.lock().unwrap())
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.seen.lock().unwrap().clone()
    }
}

impl Drop for FakeAnthropic {
    fn drop(&mut self) {
        self.task.abort();
    }
}
