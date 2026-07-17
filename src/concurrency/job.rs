use tokio::sync::oneshot;

pub struct Job {
    pub execute: Box<dyn FnOnce() + Send>,
}
pub struct JobHandle<R> {
    receiver: oneshot::Receiver<R>,
}

impl<R> JobHandle<R> {
    pub(crate) fn new(receiver: oneshot::Receiver<R>) -> Self {
        Self { receiver }
    }

    pub async fn wait(self) -> R {
        self.receiver.await.unwrap()
    }
}
