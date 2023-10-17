use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone, Debug)]
pub struct User {
    id: usize,
    tx: mpsc::UnboundedSender<Message>,
}

impl User {
    pub async fn send_message<T>(&mut self, data: T) -> Result<(), ()>
    where
        T: Serialize,
    {
        let action_str = match serde_json::to_string(&data) {
            Ok(str) => str,
            Err(e) => {
                eprintln!("error serializing action: {}", e);
                return Err(());
            }
        };

        let msg = Message::text(action_str);

        // send message to the user
        self.tx.send(msg).unwrap_or_else(|e| {
            eprintln!("websocket send error: {}", e);
        });

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct UserHolder {
    users: Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>,
}

impl UserHolder {
    pub async fn add_user(
        &mut self,
        user_id: usize,
        ws: WebSocket,
    ) -> Result<(User, SplitStream<WebSocket>), ()> {
        // Split the socket into a sender and receive of messages.
        let (mut user_ws_tx, user_ws_rx) = ws.split();

        // Use an unbounded channel to handle buffering and flushing of messages
        // to the websocket...
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        // This redirects the value of rx to the user_ws_tx
        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("websocket send error: {}", e);
                    })
                    .await;
            }
        });

        // Save the sender in our list of connected users.
        self.users.write().await.insert(user_id, tx.to_owned());

        Ok((User { tx, id: user_id }, user_ws_rx))
    }

    async fn get_user(&self, user_id: usize) -> Result<User, ()> {
        let read_guard = self.users.read().await;
        let result = read_guard.iter().find(|(&uid, _tx)| uid == user_id);

        let (uid, tx) = match result {
            Some(x) => x,
            None => {
                eprintln!("user not found: {}", user_id);
                return Err(());
            }
        };

        Ok(User {
            id: *uid,
            tx: tx.clone(),
        })
    }

    async fn remove_user(&self, user_id: usize) -> () {
        self.users.write().await.remove(&user_id);
        ()
    }
}
