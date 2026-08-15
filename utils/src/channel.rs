use std::fmt::Debug;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub struct Channel<T> {
    sender: Sender<T>,
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T> Channel<T> {
    pub fn new(capacity: usize) -> (Self, Receiver<T>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { sender }, receiver)
    }

    pub fn unbounded() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
        mpsc::unbounded_channel()
    }

    pub async fn send(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(msg).await
    }

    pub fn try_send(&self, msg: T) -> Result<(), mpsc::error::TrySendError<T>> {
        self.sender.try_send(msg)
    }

    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }
}
