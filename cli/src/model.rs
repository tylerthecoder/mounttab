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

impl Workspace {
    pub fn new(name: String, path: String) -> Workspace {
        Workspace {
            name,
            path,
            tabs: vec![],
        }
    }

    pub fn start_watching() -> () {}

    pub fn apply_action(&mut self, action: WorkspaceAction) {
        match action {
            WorkspaceAction::OpenTab(tab) => self.open_tab(tab),
            WorkspaceAction::CloseTab(tab) => self.close_tab(tab),
            WorkspaceAction::RemoveTab(tab) => self.remove_tab(tab),
            WorkspaceAction::CreateTab(tab) => self.create_tab(tab),
            WorkspaceAction::ChangeTabUrl(tab, url) => self.change_tab_url(tab, url),
        }
    }

    fn get_tab_by_name(&self, name: String) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.name == name)
    }

    fn get_tab_by_name_mut(&mut self, name: String) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.name == name)
    }

    fn create_tab(&mut self, name: String) -> () {
        let tab = Tab {
            is_open: false,
            name,
            url: "".to_owned(),
            id: "".to_owned(),
        };

        self.tabs.push(tab);
    }

    fn remove_tab(&mut self, name: String) -> () {
        self.tabs.retain(|tab| tab.name != name)
    }

    fn open_tab(&mut self, name: String) -> () {
        let tab = self.get_tab_by_name_mut(name);
        if let Some(t) = tab {
            t.is_open = true
        } else {
            println!("Couldn't find the tab")
        }
    }

    fn close_tab(&mut self, name: String) -> () {
        let tab = self.get_tab_by_name_mut(name);
        if let Some(t) = tab {
            t.is_open = false
        } else {
            println!("Couldn't find the tab")
        }
    }

    fn change_tab_url(&mut self, tab_name: String, url: String) -> () {
        let tab = self.get_tab_by_name_mut(tab_name);
        if let Some(t) = tab {
            t.url = url
        } else {
            println!("Couldn't find the tab")
        }
    }
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

        browser.tx.send(all_workspaces_message);

        while let Some(from_browser_message) = browser_rx.next().await {
            match from_browser_message {
                FromBrowserMessage::StartWorkspace(path) => {
                    let path = Path::new(&path);
                    // maybe launch this in a thread
                    self.start(path, browser);
                }
                FromBrowserMessage::WorkspaceAction(path, action) => {
                    // apply action to workspace path
                    let path = Path::new(&path);
                    apply_action_to_fs(path, &action)
                }
            }
        }
    }

    fn start(&self, path: &Path, browser: &Browser) {
        let path_clone = path.to_owned();
        let browser_clone = browser.clone();

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
