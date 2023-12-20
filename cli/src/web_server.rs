use crate::model::{FromBrowserMessage, ToBrowserMessage, WorkspaceManager};
use futures_util::{SinkExt, StreamExt};
use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};
use warp::Filter;

pub async fn start_web_server(workspace_manager: WorkspaceManager) {
    let workspace_manager = warp::any().map(move || workspace_manager.clone());

    let start_workspace = warp::path("start_workspace")
        .and(workspace_manager.clone())
        .and(warp::path::param())
        .map(
            |workspace_manager: WorkspaceManager, workspace_id: String| {
                workspace_manager.start(workspace_id);
                "Ok"
            },
        );

    let get_all_workspaces = warp::path("get_all_workspaces")
        .and(workspace_manager.clone())
        .and_then(get_api_workspaces);

    let connect_to_workspace = warp::path("chat")
        .and(warp::ws())
        .and(workspace_manager)
        .map(|ws: warp::ws::Ws, worksapce_manager| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, worksapce_manager))
        });

    let server_app = warp::get().and(start_workspace.or(connect_to_workspace));

    warp::serve(server_app).run(([127, 0, 0, 1], 3030)).await;
}

async fn get_api_workspaces(
    workspace_manager: WorkspaceManager,
) -> Result<impl warp::Reply, Infallible> {
    let workspaces = workspace_manager.get_all_workspaces().await;
    let api_workspaces: Vec<crate::model::ApiWorkspace> = workspaces
        .iter()
        .map(|workspace| workspace.to_api_workspace())
        .collect();
    Ok(warp::reply::json(&api_workspaces))
}

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

async fn user_connected(ws: WebSocket, workspaces: WorkspaceManager) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("New browser connected! ID: {}", my_id);

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // convert the websocket streams to tokio streams
    let (to_browser_tx, mut to_browser_rx) = mpsc::channel::<ToBrowserMessage>(my_id);

    let mut workspace_clone = workspaces.clone();

    // Recieves message from websocket and calls function on workspace manager
    tokio::task::spawn(async move {
        while let Some(msg_res) = user_ws_rx.next().await {
            let res = msg_res
                .map_err(|err| println!("Error reading message from websocket: {}", err))
                .and_then(|msg| {
                    msg.to_str()
                        .map_err(|err| println!("Error converting message to string"))
                })
                .and_then(|msg| {
                    serde_json::from_str::<FromBrowserMessage>(msg).map_err(|err| {
                        println!("Error deserializing message from browser: {}", err)
                    })
                });

            match res {
                Ok(from_browser_message) => match from_browser_message {},
                Err(err) => {
                    eprintln!("Error reading message from websocket");
                }
            }

            // let msg = match msg_res {
            //     Ok(msg) => msg,
            //     Err(e) => {
            //         eprintln!("websocket error(uid={}): {}", my_id, e);
            //         continue;
            //     }
            // };

            let msg = if let Ok(s) = msg.to_str() {
                s
            } else {
                eprintln!("Message not a string");
                continue;
            };

            let from_browser_mes = match serde_json::from_str::<FromBrowserMessage>(msg) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!(
                        "Error serde parsing message from browser(msg: {}): {}",
                        msg, e
                    );
                    continue;
                }
            };

            match from_browser_mes {
                FromBrowserMessage::StartWorkspace(id) => {
                    // workspace_clone.add_listener(to_browser_tx, id).await;
                }
                FromBrowserMessage::WorkspaceAction(id, action) => {
                    workspace_clone.apply_action_to_workspace(id, action).await;
                }
            }
        }
    });

    // Wait for messages and send them to the tx
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
    // });

    // workspaces
    //     .browser_connected(&browser, &mut from_browser_rx)
    //     .await;

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &workspaces).await;
}
async fn user_disconnected(my_id: usize, _workspaces: &WorkspaceManager) {
    eprintln!("good bye user: {}", my_id);
}
