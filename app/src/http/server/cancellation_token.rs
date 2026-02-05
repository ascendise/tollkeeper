use std::sync::mpsc::{self, channel};

pub fn create_cancellation_token() -> (CancelSender, CancelReceiver) {
    let (sender, receiver) = channel();
    (CancelSender(sender), CancelReceiver(receiver))
}

#[allow(dead_code)]
/// Signals to the thread that it needs to shut down
pub struct CancelSender(mpsc::Sender<bool>);
impl CancelSender {
    #[allow(dead_code)]
    pub fn send_shutdown(&self) -> Result<(), mpsc::SendError<bool>> {
        self.0.send(true)
    }
}

/// Receives signals to shut down the thread
pub struct CancelReceiver(mpsc::Receiver<bool>);
impl CancelReceiver {
    pub fn is_shutting_down(&self) -> bool {
        self.0.try_recv().is_ok()
    }
}
