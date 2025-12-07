use crate::daemon::ActiveJobData;
use indicatif::ProgressBar;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub trait ProgressObserver: Send + Sync {
    fn inc(&self, delta: u64);
    fn finish(&self);
    fn message(&self, message: String);
}

pub struct ConsoleObserver {
    pub pb: ProgressBar,
}

impl ProgressObserver for ConsoleObserver {
    fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }

    fn finish(&self) {
        self.pb.finish_with_message("Done!");
    }

    fn message(&self, message: String) {
        self.pb.set_message(message);
    }
}

pub struct DaemonObserver {
    pub job_data: Arc<ActiveJobData>,
}

impl ProgressObserver for DaemonObserver {
    fn inc(&self, delta: u64) {
        self.job_data
            .downloaded_bytes
            .fetch_add(delta, Ordering::Relaxed);
    }

    fn finish(&self) {
        self.message("Done".into());
    }

    fn message(&self, message: String) {
        let job_ref = self.job_data.clone();
        tokio::spawn(async move {
            let mut state = job_ref.state.lock().await;
            *state = message;
        });
    }
}
