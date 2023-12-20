use serde::{Deserialize, Serialize};

type Url = String;
pub type ActionSource = &'static str;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tab {
    pub url: Url,
}

#[derive(Serialize, Deserialize)]
pub struct Workspace {
    pub tabs: Vec<Tab>,
}

#[derive(Clone)]
pub struct WorkspaceComs {
    // send a message to the workspace
    pub tx: tokio::sync::broadcast::Sender<(ActionSource, WorkspaceAction)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorkspaceAction {
    OpenTab(Url),
    CloseTab(Url),
}

impl Workspace {
    fn find_tab(&self, url: &Url) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.url == *url)
    }

    pub fn apply_action(&mut self, action: WorkspaceAction) {
        match action {
            WorkspaceAction::OpenTab(url) => {
                let tab = Tab { url };
                self.tabs.push(tab);
            }
            WorkspaceAction::CloseTab(url) => {
                self.tabs.retain(|tab| tab.url != url);
            }
        }
    }

    pub fn actions_from_diff(&self, workspace: Workspace) -> Vec<WorkspaceAction> {
        let mut actions = Vec::new();

        let they_have_i_missing = workspace.tabs.iter().filter(|tab| {
            let tab_exists = self.find_tab(&tab.url).is_some();
            !tab_exists
        });

        let i_have_they_missing = self.tabs.iter().filter(|tab| {
            let tab_exists = workspace.find_tab(&tab.url).is_some();
            !tab_exists
        });

        for tab in they_have_i_missing {
            actions.push(WorkspaceAction::OpenTab(tab.url.clone()));
        }

        for tab in i_have_they_missing {
            actions.push(WorkspaceAction::CloseTab(tab.url.clone()));
        }

        actions
    }
}
