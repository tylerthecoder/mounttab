use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize)]
pub enum ToBrowserMessage {
    AllWorkspaces(Vec<ApiWorkspace>),
    // Only send to the browser when it is "connected" to a workspace
    WorkspaceAction(WorkspaceAction),
    LoadWorkspace(ApiWorkspace),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FromBrowserMessage {
    // User wants to start sending actions from this worksapce to this browser.
    StartWorkspace(String),
    WorkspaceAction(String, WorkspaceAction),
}

#[derive(Clone, Debug)]
pub struct Browser {
    pub id: usize,
    pub tx: mpsc::Sender<ToBrowserMessage>,
}

type ToClientTx = mpsc::Sender<ToBrowserMessage>;

#[derive(Serialize, Deserialize, Debug)]
enum AppAction {
    OpenWorkspace(String),
    WorkspaceAction(String, WorkspaceAction),
    CloseWorkspace(String),
}

/** A workspace is a directory on the computer that contains all the tabs */
#[derive(Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub tabs: Vec<Tab>,
    pub action_listener: Option<ToClientTx>,
}

impl Workspace {
    pub fn to_api_workspace(&self) -> ApiWorkspace {
        ApiWorkspace {
            id: self.id.clone(),
            name: self.name.clone(),
            tabs: self.tabs.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiWorkspace {
    pub id: String,
    pub name: String,
    pub tabs: Vec<Tab>,
}

/** Each tab is a directory of config
 * A directory would look like this
 * workspace
 * - $(tab.name)
 *  - url.txt: contians the url string
 *  - is_open: contains true or false
 * */
#[derive(Serialize, Deserialize, Clone)]
pub struct Tab {
    // The name should be unique across all tabs functions as an id
    pub name: String,
    pub url: String,
    pub is_open: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorkspaceAction {
    OpenTab(String),
    CloseTab(String),
    // Tab name , Tab url
    ChangeTabUrl(String, String),
    CreateTab(String),
    RemoveTab(String),
}

#[derive(Default, Clone)]
pub struct WorkspaceManager {
    workspaces: Arc<RwLock<Vec<Workspace>>>,
}

impl WorkspaceManager {
    pub async fn load_workspaces(&self) {
        println!("Loading workspaces");
        let mut workspaces = self.workspaces.write().await;
        workspaces.push(Workspace::new_from_fs(
            "/home/tylord/dev/tabfs-rs/test".as_ref(),
        ));
        println!("Loaded {} workspaces", workspaces.len());
    }

    pub async fn add_listener(&mut self, to_client_tx: ToClientTx, workspace_id: String) {
        let mut workspace = self.get_mut_workspace_by_id(workspace_id).await;
        workspace.action_listener = Some(to_client_tx.clone());
    }

    pub async fn browser_connected(
        &self,
        browser: &Browser,
        browser_rx: &mut UnboundedReceiverStream<FromBrowserMessage>,
    ) {
        let mut workspaces = self.workspaces.write().await;

        while let Some(from_browser_message) = browser_rx.next().await {
            println!("Got message from browser: {:?}", from_browser_message);
            match from_browser_message {
                FromBrowserMessage::StartWorkspace(id) => {
                    // Add the browser to the workspace so it can start recieving messages
                    let mut workspace = self.get_mut_workspace_by_id(id.clone()).await;
                    // workspace.browser = Some(browser.clone());
                    //
                    // self.start(id, browser).await;
                }
                FromBrowserMessage::WorkspaceAction(id, action) => {
                    let workspace = self.get_workspace_by_id(id).await;

                    match apply_action_to_fs(&workspace.path.as_ref(), &action) {
                        Ok(()) => {
                            println!("Applied action to fs");
                        }
                        Err(err) => {
                            println!("Error applying action to fs {}", err);
                        }
                    }
                }
            }
        }
    }

    pub async fn apply_action_to_workspace(&self, workspace_id: String, action: WorkspaceAction) {
        let workspace = self.get_workspace_by_id(workspace_id).await;

        match apply_action_to_fs(&workspace.path.as_ref(), &action) {
            Ok(()) => {
                println!("Applied action to fs");
            }
            Err(err) => {
                println!("Error applying action to fs {}", err);
            }
        }
    }

    pub async fn start(&self, workspace_id: String) {
        println!("Starting workspace: {:?}", workspace_id);

        let workspace = self.get_workspace_by_id(workspace_id.clone()).await;
        let manager = self.clone();

        let b_action = ToBrowserMessage::LoadWorkspace(workspace.to_api_workspace());

        // browser.tx.send(b_action).await.unwrap_or_else(|e| {
        //     eprintln!("Error sending to browser: {}", e);
        // });

        println!("Sent load workspace message");

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel::<WorkspaceAction>(101);
            println!("spawning file watcher");
            tokio::spawn(async move {
                let res = file_watcher::async_watch(&workspace.path.as_ref(), tx).await;
                if let Err(e) = res {
                    eprintln!("error watching file: {}", e);
                }
                println!("Watch ended");
            });

            while let Some(action) = rx.recv().await {
                println!("Received action from file watcher: {:?}", action);

                // get the workspace and send action to browser if it exists
                let workspace = manager.get_workspace_by_id(workspace_id.clone()).await;

                if let Some(browser) = workspace.action_listener {
                    println!("Sent action to browser");

                    let b_action = ToBrowserMessage::WorkspaceAction(action.to_owned());

                    // browser.tx.send(b_action).await.unwrap_or_else(|e| {
                    //     eprintln!("Error sending to browser: {}", e);
                    // });
                } else {
                    println!("No browser found");
                }
            }
        });
    }

    pub async fn get_all_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.read().await.to_vec()
    }

    pub async fn get_workspace_by_id(&self, workspace_id: String) -> Workspace {
        let workspaces = self.get_all_workspaces().await;

        let workspace = workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
            .unwrap_or_else(|| panic!("Couldn't find workspace with id: {}", workspace_id.clone()))
            .clone();

        workspace
    }

    pub async fn get_mut_workspace_by_id(&self, workspace_id: String) -> Workspace {
        let mut workspaces = self.workspaces.write().await;
        let workspace = workspaces
            .iter_mut()
            .find(|workspace| workspace.id == workspace_id)
            .unwrap_or_else(|| panic!("Couldn't find workspace with id: {}", workspace_id.clone()))
            .clone();
        workspace
    }

    // Add the workspace to a list on a file
    pub fn make_worksapce(&mut self, path: &Path) {}
}
