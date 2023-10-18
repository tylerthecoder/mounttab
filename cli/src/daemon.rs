use crate::model::{Browser, FromBrowserMessage, ToBrowserMessage, WorkspaceManager};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::{self};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn start_daemon() -> Result<(), std::io::Error> {
    // start websocket server

    let worksapce_manager = WorkspaceManager::default();

    let workspaces = warp::any().map(move || worksapce_manager.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(workspaces)
        .map(|ws: warp::ws::Ws, worksapce_manager| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, worksapce_manager))
        });

    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn user_connected(ws: WebSocket, workspaces: WorkspaceManager) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("New browser connected! ID: {}", my_id);

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // convert the websocket streams to tokio streams
    let (to_browser_tx, mut to_browser_rx) = mpsc::channel::<ToBrowserMessage>(my_id);
    let (from_browser_tx, from_browser_rx) = mpsc::unbounded_channel::<FromBrowserMessage>();
    let mut from_browser_rx = UnboundedReceiverStream::new(from_browser_rx);

    // Recieves message from websocket and forwards them
    tokio::task::spawn(async move {
        while let Some(msg_res) = user_ws_rx.next().await {
            let msg = match msg_res {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", my_id, e);
                    continue;
                }
            };

            let msg = if let Ok(s) = msg.to_str() {
                s
            } else {
                eprintln!("Message not a string");
                continue;
            };

            let from_browser_mes = match serde_json::from_str::<FromBrowserMessage>(msg) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error serde parsing message from browser: {}", e);
                    continue;
                }
            };

            match from_browser_tx.send(from_browser_mes) {
                Ok(()) => {
                    println!("Message sent successfully")
                }
                Err(err) => {
                    println!("Got an error sending to socket: {}", err)
                }
            }
        }
    });

    // Sends message to websocket
    tokio::task::spawn(async move {
        while let Some(to_browser_message) = to_browser_rx.recv().await {
            let action_str = match serde_json::to_string(&to_browser_message) {
                Ok(str) => str,
                Err(e) => {
                    eprintln!("error serializing action: {}", e);
                    continue;
                }
            };

            let msg = Message::text(action_str);

            match user_ws_tx.send(msg).await {
                Ok(()) => {
                    println!("Sent message to socket");
                }
                Err(err) => {
                    eprintln!("Error sending message: {}", err);
                }
            };
        }
    });

    let browser = Browser {
        id: my_id,
        tx: to_browser_tx,
    };

    workspaces
        .browser_connected(&browser, &mut from_browser_rx)
        .await;

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &workspaces).await;
}

async fn user_disconnected(my_id: usize, _workspaces: &WorkspaceManager) {
    eprintln!("good bye user: {}", my_id);
}
