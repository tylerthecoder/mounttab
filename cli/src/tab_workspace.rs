use crate::{model::Tab, file_watcher};

struct Workspace {
    tabList: Vec<Tab>,
    watcher: file_watcher::FileWatcher,
}


