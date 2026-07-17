use crate::concurrency::job::{Job, JobHandle};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use tokio::sync::oneshot;

pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = std::thread::spawn(move || {
            loop {
                let job = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv()
                };

                match job {
                    Ok(job) => {
                        (job.execute)();
                    }

                    Err(_) => {
                        // All senders have been dropped.
                        // Time to shut down this worker.
                        break;
                    }
                }
            }
        });

        Self { handle }
    }
}

impl WorkerPool {
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F, R>(&self, job: F) -> JobHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        let queued_job = Job {
            execute: Box::new(move || {
                let result = job();
                let _ = tx.send(result);
            }),
        };

        self.sender
            .as_ref()
            .expect("WorkerPool has been shut down.")
            .send(queued_job)
            .expect("Failed to submit job.");

        JobHandle::new(rx)
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        // Dropping the final sender closes the request queue.
        self.sender.take();

        // Wait for every worker to finish.
        for worker in self.workers.drain(..) {
            worker.handle.join().expect("Worker thread panicked.");
        }
    }
}
