use crate::{
    file_watcher,
    model::{Action, Tab},
};
use futures_util::Stream;
use tokio::sync::mpsc::Receiver;

pub struct Workspace {
    tab_list: Vec<Tab>,
    path: String,
    // action_stream: Arc<dyn Stream<Item = Action>>,
}

impl Workspace {
    pub fn new(path: String) -> Result<Self, notify::Error> {
        let action_stream = file_watcher::watch(path.clone())?;
        Ok(Self {
            tab_list: Vec::new(),
            path,
            // action_stream: Arc::new(action_stream),
        })
    }
}
//
// #[derive(Default)]
// pub struct WorkspaceManager {
//     workspaces: Vec<Workspace>,
// }
//
// impl WorkspaceManager {
//     pub fn start(&mut self, path: String) -> Result<Receiver<Action>, notify::Error> {
//         // let workspace = Workspace::new(path)?;
//         let action_stream = file_watcher::watch(path.clone())?;
//         // self.workspaces.push(workspace);
//         Ok(action_stream)
//     }
// }
