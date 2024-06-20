use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone)]
pub struct AppState {
    pub channel: Arc<Channel>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState { channel: Arc::new(Channel::new(16)) }
    }

    pub fn with_capacity(cap: usize) -> AppState {
        AppState { channel: Arc::new(Channel::new(cap)) }
    }
}

pub struct Channel {
    pub tx: Sender<String>,
    rx: Receiver<String>,
}

impl Channel {
    pub fn new(cap: usize) -> Channel {
        let (tx, rx) = broadcast::channel(cap);
        Channel { tx, rx }
    }

    pub fn subscribe(&self) -> (Sender<String>, Receiver<String>) {
        let rx = self.tx.subscribe();
        let tx = self.tx.clone();

        (tx, rx)
    }
}