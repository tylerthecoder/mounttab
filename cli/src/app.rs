use crate::json_storage::start_file_watcher;
use crate::model2::{ActionSource, Workspace, WorkspaceAction};
use crate::web_server_2::start_web_server;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub async fn start_app(path: PathBuf) -> Result<(), std::io::Error> {
    let workspace = Workspace::from_path(&path);
    let (tx, _rx) = broadcast::channel::<(ActionSource, WorkspaceAction)>(16);

    let workman = WorkspaceManger {
        tx,
        workspace: Arc::new(RwLock::new(workspace)),
    };

    // workspace.listen_for_actions(path, coms.clone());

    start_file_watcher(&workman, path);

    // Create a web browser
    // Command::new("brave")
    //     .output()
    //     .expect("failed to launch browser");

    start_web_server(workman).await;

    Ok(())
}

// #[derive(Clone)]
// pub struct WorkspaceComs {
//     // send a message to the workspace
//     pub tx: tokio::sync::broadcast::Sender<(ActionSource, WorkspaceAction)>,
// }

#[derive(Clone)]
pub struct WorkspaceManger {
    pub tx: tokio::sync::broadcast::Sender<(ActionSource, WorkspaceAction)>,
    pub workspace: Arc<RwLock<Workspace>>,
}

// impl WorkspaceManger {
//     // Get workspace for reading
//
//     pub fn get_workspace(&self) {
//         self.workspace.read()
//     }
//
//     // Get workspace for writing
//
//     // Send messages to workspace
//
//     // Create an action receiver
// }
