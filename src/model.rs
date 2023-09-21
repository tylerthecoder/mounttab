use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    OpenTab(String),
    CloseTab(String),
}

pub struct Tab {
    title: String,
    id: String,
}

pub struct TabList {
    tabs: Vec<Tab>,
}

impl TabList {
    pub fn handle_action(&mut self, action: Action) {
       match action {
           Action::OpenTab(title) => {
               let id = format!("tab-{}", self.tabs.len());
               self.tabs.push(Tab { title, id });
           }
           Action::CloseTab(id) => {
               self.tabs.retain(|tab| tab.id != id);
           }
       } 
    }
}
