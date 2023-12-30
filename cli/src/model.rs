use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub type Url = String;
pub type ActionSource = &'static str;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub tabs: Vec<Url>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorkspaceAction {
    OpenTab(Url),
    CloseTab(Url),
}

impl Workspace {
    fn find_tab(&self, url: &Url) -> Option<&Url> {
        self.tabs.iter().find(|tab| *tab == url)
    }

    pub fn apply_action(&mut self, action: WorkspaceAction) {
        match action {
            WorkspaceAction::OpenTab(url) => {
                self.tabs.push(url);
            }
            WorkspaceAction::CloseTab(url) => {
                self.tabs.retain(|tab| *tab != url);
            }
        }
    }

    pub fn actions_from_diff(&self, workspace: Workspace) -> Vec<WorkspaceAction> {
        let mut actions = Vec::new();

        let they_have_i_missing = workspace.tabs.iter().filter(|tab| {
            let tab_exists = self.find_tab(tab).is_some();
            !tab_exists
        });

        let i_have_they_missing = self.tabs.iter().filter(|tab| {
            let tab_exists = workspace.find_tab(tab).is_some();
            !tab_exists
        });

        for tab in they_have_i_missing {
            actions.push(WorkspaceAction::OpenTab(tab.clone()));
        }

        for tab in i_have_they_missing {
            actions.push(WorkspaceAction::CloseTab(tab.clone()));
        }

        actions
    }
}

#[derive(Clone)]
pub struct WorkspaceManger {
    pub tx: tokio::sync::broadcast::Sender<(ActionSource, WorkspaceAction)>,
    pub workspace: Arc<RwLock<Workspace>>,
}

impl WorkspaceManger {
    pub fn new(workspace: Workspace) -> WorkspaceManger {
        let (tx, _rx) = broadcast::channel::<(ActionSource, WorkspaceAction)>(16);
        WorkspaceManger {
            tx,
            workspace: Arc::new(RwLock::new(workspace)),
        }
    }
}
