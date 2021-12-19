use crossbeam_channel::{Sender, TrySendError};
use std::time::Duration;
use tokio::time;

pub struct AsyncSender<T> {
    sender: Sender<T>,
    pub timeout: Duration,
}

impl<T> core::clone::Clone for AsyncSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            timeout: self.timeout.clone(),
        }
    }
}

impl<T> AsyncSender<T> {
    pub fn new(sender: Sender<T>, timeout: Option<Duration>) -> Self {
        Self {
            sender,
            timeout: timeout.unwrap_or(Duration::from_millis(0x10)),
        }
    }

    /// async wrapper for Sender::send
    pub async fn send(&mut self, mut item: T) {
        while let Err(e) = self.sender.try_send(item) {
            item = match e {
                TrySendError::Full(item) => item,
                TrySendError::Disconnected(_) => {
                    panic!("receiving channel got closed, please dont do that")
                }
            };

            time::sleep(self.timeout).await;
        }
    }
}
