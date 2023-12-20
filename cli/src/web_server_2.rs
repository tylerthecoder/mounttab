use crate::model2::{WorkspaceAction, WorkspaceComs};
use futures_util::{SinkExt, StreamExt};
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

pub async fn start_web_server(workspace_coms: WorkspaceComs) {
    let workspace_middleware = warp::any().map(move || workspace_coms.clone());

    let connect_to_workspace = warp::path("chat")
        .and(warp::ws())
        .and(workspace_middleware)
        .map(|ws: warp::ws::Ws, workspace_coms: WorkspaceComs| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| client_connected(socket, workspace_coms))
        });

    let server_app = warp::get().and(connect_to_workspace);
    warp::serve(server_app).run(([127, 0, 0, 1], 3030)).await;
}

async fn client_connected(socket: WebSocket, coms: WorkspaceComs) {
    // try to connect the user to the worksapce, if someone is already connected, then log an error
    let (mut user_ws_tx, mut user_ws_rx) = socket.split();

    let coms_clone = coms.clone();

    // Recieves message from websocket and calls function on workspace manager
    tokio::task::spawn(async move {
        while let Some(msg_res) = user_ws_rx.next().await {
            // Use an async block to process each message
            let res: Result<_, ()> = async {
                let msg = msg_res.map_err(|err| {
                    eprintln!("Error reading message from websocket: {}", err);
                    ()
                })?;

                let str_msg = msg.to_str().map_err(|_e| {
                    eprintln!("Error converting message to string");
                })?;

                let from_client: WorkspaceAction =
                    serde_json::from_str(str_msg).map_err(|err| {
                        eprintln!("Error deserializing message from browser: {}", err);
                        ()
                    })?;

                coms_clone.tx.send(("Socket", from_client)).map_err(|err| {
                    eprintln!("Error sending message to workspace manager: {}", err);
                    ()
                })?;

                Ok(())
            }
            .await;

            if let Err(_) = res {
                eprintln!("Error processing message");
            }
        }
    });

    while let Ok((action_source, action)) = coms.tx.subscribe().recv().await {
        if action_source == "Socket" {
            continue;
        }

        let res: Result<_, ()> = async {
            let action_str = serde_json::to_string(&action).map_err(|e| {
                eprintln!("error serializing action: {}", e);
            })?;

            let msg = Message::text(action_str);

            user_ws_tx.send(msg).await.map_err(|e| {
                eprintln!("Error sending message to browser: {}", e);
            })?;

            Ok(())
        }
        .await;

        if let Err(_) = res {
            eprintln!("Error processing message");
        }
    }
}
