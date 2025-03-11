pub mod mcjars;
pub mod modrinth;
pub mod mojang;

use reqwest::Client;
use std::sync::{Arc, LazyLock, Mutex};

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct Progress {
    pub total: usize,

    progress: Arc<Mutex<[usize; 2]>>,
    thread: Option<tokio::task::JoinHandle<()>>,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            progress: Arc::new(Mutex::new([0, total])),
            thread: None,
        }
    }

    pub fn incr<N: Into<usize>>(&mut self, n: N) {
        self.progress.lock().as_mut().unwrap()[0] += n.into();
    }

    pub fn progress(&self) -> usize {
        self.progress.lock().unwrap()[0]
    }

    pub fn percent(&self) -> f64 {
        (self.progress() as f64 / self.total as f64) * 100.0
    }

    pub fn spinner<F>(&mut self, fmt: F)
    where
        F: Fn(&Progress, &str) -> String + Send + Sync + 'static,
    {
        let total = self.total;
        let progress = Arc::clone(&self.progress);

        let thread = tokio::spawn(async move {
            let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let mut i = 0;

            loop {
                eprint!(
                    "{}",
                    fmt(
                        &Progress {
                            total,
                            progress: Arc::clone(&progress),
                            thread: None
                        },
                        &spinner[i].to_string()
                    )
                );

                i = (i + 1) % spinner.len();
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });

        self.thread = Some(thread);
    }

    pub fn finish(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.thread.take().unwrap().abort();
    }
}

pub static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(format!("github.com/mcjars/mcvcli/{}", VERSION))
        .build()
        .unwrap()
});
