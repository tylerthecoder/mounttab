use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc::channel;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

use crate::model::Action;

pub mod file_watcher;
pub mod model;

#[derive(Serialize, Deserialize, Debug)]
enum AppWebSocketMessage {
    OpenWorkspace(String),
    WorkspaceAction(String, model::Action),
    CloseWorkspace(String),
}

/** Some messages the client will never send us */
#[derive(Serialize, Deserialize, Debug)]
enum SendableMessage {
    AllWorkspaces(Vec<String>),
}

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone, Debug)]
struct User {
    id: usize,
    tx: mpsc::UnboundedSender<Message>,
}

impl User {
    async fn send_message<T>(&mut self, data: T) -> Result<(), ()>
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
struct UserHolder {
    users: Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>,
}

impl UserHolder {
    async fn add_user(
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

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = UserHolder::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, mut users: UserHolder) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Save the sender in our list of connected users.
    let (mut user, mut ws_rx) = users.add_user(my_id, ws).await.unwrap();

    // Send the user a message about all the worksapces currently operating

    let all_workspaces_message =
        SendableMessage::AllWorkspaces(vec!["test".to_string(), "test2".to_string()]);

    let _ = user.send_message(all_workspaces_message).await;

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };

        let mes_res = user_message(my_id, msg, &users).await;

        if let Ok(()) = mes_res {
            println!("message handled successfully")
        }

        if let Err(e) = mes_res {
            eprintln!("error handling message: {}", e);
        }
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &UserHolder) -> Result<(), notify::Error> {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return Ok(());
    };

    println!("received message: {}", msg);

    // try converting the message to a AppWebSocketMessage
    let msg: Result<AppWebSocketMessage, serde_json::Error> = serde_json::from_str(msg);
    let msg = match msg {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("error parsing message: {}", e);
            return Ok(());
        }
    };

    match msg {
        AppWebSocketMessage::OpenWorkspace(path) => {
            println!("opening workspace: {}", path);

            let (tx, mut rx) = channel::<Action>(101);

            tokio::spawn(async move {
                println!("spawning file watcher");
                let res = file_watcher::async_watch(path, tx).await;
                if let Err(e) = res {
                    eprintln!("error watching file: {}", e);
                }
                println!("Watch ended");
            });

            println!("file watcher created");

            while let Some(action) = rx.recv().await {
                println!("Received action from file watcher: {:?}", action);

                let mut user = match users.get_user(my_id).await {
                    Ok(user) => user,
                    Err(_err) => {
                        eprintln!("There was an error");
                        continue;
                    }
                };

                let _ = user.send_message(action).await;
            }

            // convert error to warp::Error and return

            Ok(())
        }
        AppWebSocketMessage::WorkspaceAction(path, _action) => {
            println!("workspace action: {}", path);
            Ok(())
        }
        AppWebSocketMessage::CloseWorkspace(path) => {
            println!("closing workspace: {}", path);
            Ok(())
        }
    }
}

async fn user_disconnected(my_id: usize, users: &UserHolder) {
    eprintln!("good bye user: {}", my_id);

    let _ = users.remove_user(my_id).await;
}
