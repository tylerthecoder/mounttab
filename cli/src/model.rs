use crate::file_watcher;
use crate::file_watcher::apply_action_to_fs;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

#[derive(Serialize, Deserialize)]
pub enum ToBrowserMessage {
    AllWorksapces(Vec<Workspace>),
    // Only send to the browser when it is "connected" to a workspace
    WorkspaceAction(WorkspaceAction),
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
enum AppAction {
    OpenWorkspace(String),
    WorkspaceAction(String, WorkspaceAction),
    CloseWorkspace(String),
}

/** A workspace is a directory on the computer that contains all the tabs */
#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub name: String,
    pub path: String,
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
    pub name: String,
    pub url: String,
    pub is_open: bool,
    pub id: String,
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
    pub async fn browser_connected(
        &self,
        browser: &Browser,
        browser_rx: &mut UnboundedReceiverStream<FromBrowserMessage>,
    ) {
        let workspaces = self.get_all_workspaces().await;

        let all_workspaces_message = ToBrowserMessage::AllWorksapces(workspaces);

        match browser.tx.send(all_workspaces_message).await {
            Ok(()) => {
                println!("Sending message");
            }
            Err(err) => {
                println!("Error sending: {}", err);
            }
        };

        let ignore_next_action = Arc::<RwLock<bool>>::new(RwLock::new(false));

        while let Some(from_browser_message) = browser_rx.next().await {
            match from_browser_message {
                FromBrowserMessage::StartWorkspace(path) => {
                    let lock = Arc::clone(&ignore_next_action);
                    let path = Path::new(&path);
                    // maybe launch this in a thread
                    self.start(path, browser, lock);
                }
                FromBrowserMessage::WorkspaceAction(path, action) => {
                    let lock = Arc::clone(&ignore_next_action);
                    // apply action to workspace path
                    let path = Path::new(&path);
                    // we should stop the file watcher when we send this, or at least tell it to
                    // ignore the next event
                    let mut w = lock.write().await;
                    *w = true;
                    match apply_action_to_fs(path, &action) {
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

    fn start(&self, path: &Path, browser: &Browser, ignore_next_action: Arc<RwLock<bool>>) {
        let path_clone = path.to_owned();
        let browser_clone = browser.clone();

        // check if the workspace path is real
        if !path.exists() {
            eprintln!("File path doesn't exist");
        }

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel::<WorkspaceAction>(101);
            println!("spawning file watcher");
            tokio::spawn(async move {
                let res = file_watcher::async_watch(&path_clone, tx).await;
                if let Err(e) = res {
                    eprintln!("error watching file: {}", e);
                }
                println!("Watch ended");
            });

            while let Some(action) = rx.recv().await {
                println!("Got message from file watcher");
                // let should_ignore = ignore_next_action.read().await;
                //
                // if *should_ignore {
                //     let mut ignore_lock = ignore_next_action.write().await;
                //     println!("Ignoring action from file watcher: {:?}", action);
                //     *ignore_lock = false;
                // }
                println!("Received action from file watcher: {:?}", action);

                let b_action = ToBrowserMessage::WorkspaceAction(action.to_owned());

                browser_clone.tx.send(b_action).await.unwrap_or_else(|e| {
                    eprintln!("Error sending to browser: {}", e);
                });
            }
        });
    }

    pub async fn get_all_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.read().await.to_vec()
    }

    // Add the workspace to a list on a file
    pub fn make_worksapce(&mut self, path: &Path) {}
}
