use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

pub mod file_watcher;
pub mod model;
pub mod tab_workspace;

#[derive(Serialize, Deserialize, Debug)]
enum AppWebSocketMessage {
    OpenWorkspace(String),
    WorkspaceAction(String, model::Action),
    CloseWorkspace(String),
}

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // print a json serialized AppWebSocketMessage

    let res = serde_json::to_string_pretty(&AppWebSocketMessage::OpenWorkspace("test".to_string()));

    println!("json: {}", res.unwrap());

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();
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

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

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
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
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

async fn user_message(my_id: usize, msg: Message, users: &Users) -> Result<(), notify::Error> {
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

            // Creating a file watcher

            let mut action_stream = file_watcher::watch(path)?;

            println!("file watcher created");

            // let action_rx = workspaces.start(path)?;

            while let Some(action) = action_stream.next().await {
                println!("received action from file watcher: {:?}", action);

                // send message to websocket
                let read_guard = users.read().await;

                let result = read_guard.iter().find(|(&uid, _tx)| uid == my_id);

                let (_uid, tx) = match result {
                    Some(x) => x,
                    None => {
                        eprintln!("user not found: {}", my_id);
                        continue;
                    }
                };

                let action_str = match serde_json::to_string(&action) {
                    Ok(str) => str,
                    Err(e) => {
                        eprintln!("error serializing action: {}", e);
                        continue;
                    }
                };

                let msg = Message::text(action_str);

                // send message to the user
                tx.send(msg).unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                });
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

    // New message from this user, send it to everyone else (except same uid)...
    // for (&uid, tx) in users.read().await.iter() {
    //     if my_id != uid {
    //         if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
    //             // The tx is disconnected, our `user_disconnected` code
    //             // should be happening in another task, nothing more to
    //             // do here.
    //         }
    //     }
    // }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
