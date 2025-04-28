use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use shared::{PlayerId, SystemMessages};

use crate::Sender;

#[derive(Default, Clone)]
pub struct Manager {
    inner: Arc<Mutex<HashMap<PlayerId, Sender>>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, message: SystemMessages) {
        for sender in self.inner.lock().await.values() {
            let _ = sender.send(message.clone());
        }
    }

    async fn broadcast_except(&self, id: PlayerId, message: SystemMessages) {
        for (client_id, sender) in self.inner.lock().await.iter() {
            if *client_id != id {
                let _ = sender.send(message.clone());
            }
        }
    }

    pub async fn broadcast_to(&self, id: PlayerId, message: SystemMessages) {
        if let Some(sender) = self.inner.lock().await.get(&id) {
            let _ = sender.send(message);
        }
    }

    pub async fn add(&self, id: PlayerId, sender: Sender) {
        self.inner.lock().await.insert(id, sender);
    }

    async fn remove(&self, id: PlayerId) {
        self.inner.lock().await.remove(&id);
    }
}

#[derive(Clone)]
pub struct ScopedManager {
    pub id: PlayerId,
    inner: Manager,
}

impl ScopedManager {
    pub fn new(id: PlayerId, parent: Manager) -> Self {
        Self { id, inner: parent }
    }

    pub async fn broadcast_to_self(&self, message: SystemMessages) {
        self.inner.broadcast_to(self.id, message).await
    }

    pub async fn broadcast_except_self(&self, message: SystemMessages) {
        self.inner.broadcast_except(self.id, message).await
    }

    pub async fn broadcast_to_all(&self, message: SystemMessages) {
        self.inner.broadcast(message).await;
    }

    pub async fn remove(&self, id: PlayerId) {
        self.inner.remove(id).await
    }
}