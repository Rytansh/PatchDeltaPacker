use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};

const DOWNLOAD_WEIGHT: f64 = 0.80;
const VERIFY_WEIGHT: f64 = 0.05;
const INSTALL_WEIGHT: f64 = 0.15;
const RESOLUTION: u64 = 10_000;

pub struct UpdateProgress {
    progress: ProgressBar,

    patch_count: usize,
    current_patch: AtomicUsize,

    install_completed: AtomicUsize,
    install_total: AtomicUsize,
}

impl UpdateProgress {
    pub fn new(patch_count: usize) -> Self {
        let progress = ProgressBar::new(RESOLUTION);

        progress.set_style(
            ProgressStyle::with_template(
                "{msg}\n\
         {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        Self {
            progress,
            patch_count,
            current_patch: AtomicUsize::new(0),
            install_completed: AtomicUsize::new(0),
            install_total: AtomicUsize::new(0),
        }
    }

    pub fn begin_patch(&self, index: usize) {
        self.current_patch.store(index, Ordering::Relaxed);
    }

    pub fn set_message<S: Into<String>>(&self, message: S) {
        self.progress.set_message(message.into());
    }

    pub fn download_progress(&self, downloaded: u64, total: u64) {
        let fraction = if total == 0 {
            1.0
        } else {
            downloaded as f64 / total as f64
        };

        self.set_patch_progress(fraction * DOWNLOAD_WEIGHT);
    }

    pub fn finish_verification(&self) {
        self.set_patch_progress(DOWNLOAD_WEIGHT + VERIFY_WEIGHT);
    }

    pub fn begin_install(&self, operations: usize) {
        self.install_completed.store(0, Ordering::Relaxed);
        self.install_total.store(operations, Ordering::Relaxed);
    }

    pub fn complete_install_operation(&self) {
        let completed = self.install_completed.fetch_add(1, Ordering::Relaxed) + 1;

        let total = self.install_total.load(Ordering::Relaxed);

        let fraction = if total == 0 {
            1.0
        } else {
            completed as f64 / total as f64
        };

        self.set_patch_progress(DOWNLOAD_WEIGHT + VERIFY_WEIGHT + fraction * INSTALL_WEIGHT);
    }

    pub fn finish(&self) {
        self.progress.finish_with_message("Complete!");
    }

    fn set_patch_progress(&self, patch_fraction: f64) {
        let current = self.current_patch.load(Ordering::Relaxed);

        let overall = (current as f64 + patch_fraction) / self.patch_count as f64;

        self.progress
            .set_position((overall * RESOLUTION as f64) as u64);
    }
}
