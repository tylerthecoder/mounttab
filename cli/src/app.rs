use crate::model2::{ActionSource, Workspace, WorkspaceAction, WorkspaceComs};
use crate::web_server_2::start_web_server;
use std::path::Path;
use std::process::Command;
use tokio::sync::broadcast;

pub async fn start_app(path: &Path) -> Result<(), std::io::Error> {
    let mut workspace = Workspace::from_path(path);
    let (tx, _rx) = broadcast::channel::<(ActionSource, WorkspaceAction)>(16);

    let coms = WorkspaceComs { tx };

    workspace.listen_for_actions(path, coms.clone());

    // Create a web browser
    Command::new("brave")
        .output()
        .expect("failed to launch browser");

    start_web_server(coms).await;

    Ok(())
}
