use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum WsEvent {
    NewPost { post_id: Uuid, author_id: Uuid },
    NewLike { post_id: Uuid, user_id: Uuid },
    NewFollow { follower_id: Uuid, followee_id: Uuid },
}

pub struct ConnectionManager {
    channels: RwLock<HashMap<Uuid, broadcast::Sender<WsEvent>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<WsEvent> {
        let mut channels = self.channels.write().await;
        let sender = channels.entry(user_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            tx
        });
        sender.subscribe()
    }

    pub async fn broadcast(&self, event: WsEvent, exclude: Option<Uuid>) {
        let channels = self.channels.read().await;
        for (user_id, tx) in channels.iter() {
            if let Some(excluded) = exclude {
                if *user_id == excluded {
                    continue;
                }
            }
            let _ = tx.send(event.clone());
        }
    }

    pub async fn notify_followers(&self, event: WsEvent, follower_id: Uuid) {
        let channels = self.channels.read().await;
        for (user_id, tx) in channels.iter() {
            let _ = tx.send(event.clone());
        }
    }
}
