use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
                if let Some(index) = self.tabs.iter().position(|x| *x == url) {
                    self.tabs.remove(index);
                }
            }
        }
    }

    pub fn actions_from_diff(&self, workspace: Workspace) -> Vec<WorkspaceAction> {
        let mut actions = Vec::new();
        let mut self_tab_count = HashMap::new();
        let mut workspace_tab_count = HashMap::new();

        // Count occurrences of each URL in the current workspace.
        for tab in &self.tabs {
            *self_tab_count.entry(tab.clone()).or_insert(0) += 1;
        }

        // Count occurrences of each URL in the provided workspace.
        for tab in &workspace.tabs {
            *workspace_tab_count.entry(tab.clone()).or_insert(0) += 1;
        }

        // Generate OpenTab actions for additional occurrences in the provided workspace.
        for (tab, count) in workspace_tab_count.iter() {
            let self_count = self_tab_count.get(tab).copied().unwrap_or(0);
            for _ in 0..(count - self_count) {
                actions.push(WorkspaceAction::OpenTab(tab.clone()));
            }
        }

        // Generate CloseTab actions for extra occurrences in the current workspace.
        for (tab, count) in self_tab_count.iter() {
            let workspace_count = workspace_tab_count.get(tab).copied().unwrap_or(0);
            for _ in 0..(count - workspace_count) {
                actions.push(WorkspaceAction::CloseTab(tab.clone()));
            }
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
