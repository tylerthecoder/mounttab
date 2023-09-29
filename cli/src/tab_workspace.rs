use std::sync::Arc;
use futures_util::Stream;
use crate::{model::{Tab, Action}, file_watcher};

struct Workspace {
    tab_list: Vec<Tab>,
    path: String,
    action_stream: Arc<dyn Stream<Item = Action>>,
}


impl Workspace {
    pub fn new(path: String) -> Result<Self, notify::Error> {
        let action_stream = file_watcher::watch(path.clone())?;
        Ok(
            Self {
                tab_list: Vec::new(),
                path,
                action_stream: Arc::new(action_stream),
            }
        )
    }
}


