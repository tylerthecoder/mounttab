use crate::model::{Tab, Workspace, WorkspaceAction};
use notify::{RecursiveMode, Watcher};
use std::{fs, path::Path};
use tokio::sync::mpsc;

pub async fn async_watch(
    path: &Path,
    action_tx: mpsc::Sender<WorkspaceAction>,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let path_clone = path.clone();

    let mut watcher = notify::recommended_watcher(tx)?;

    println!("Watcher starting");

    // let path = "/home/tylord/dev/tabfs-rs/test".;

    watcher.watch(path, RecursiveMode::Recursive)?;

    println!("Watcher started");

    for res in rx {
        let event = match res {
            Ok(event) => event,
            Err(error) => {
                println!("watch error: {:?}", error);
                continue;
            }
        };
        println!("got event: {:?}", event);

        let action = match watch_event_to_action(event, &path_clone) {
            Some(action) => action,
            None => {
                println!("No action for event");
                continue;
            }
        };
        println!("Action received: {:?}", action);

        match action_tx.send(action).await {
            Ok(_res) => {}
            Err(e) => {
                println!("Error sending action: {:?}", e);
            }
        }
    }

    Ok(())
}

fn watch_event_to_action(event: notify::Event, base_path: &Path) -> Option<WorkspaceAction> {
    let canonical_path = match fs::canonicalize(base_path) {
        Ok(path) => path,
        Err(e) => {
            println!("Error canonicalizing path: {:?}", e);
            return None;
        }
    };

    println!("canonical_path: {:?} {:?}", base_path, canonical_path);

    match event.kind {
        notify::event::EventKind::Create(_) => {
            let path = event.paths[0].strip_prefix(canonical_path);

            if let Ok(path) = path {
                return Some(WorkspaceAction::OpenTab(path.to_string_lossy().to_string()));
            }
        }
        notify::event::EventKind::Remove(_) => {
            let path = event.paths[0].strip_prefix(canonical_path);

            if let Ok(path) = path {
                return Some(WorkspaceAction::CloseTab(
                    path.to_string_lossy().to_string(),
                ));
            }
        }
        _ => {
            println!("unhandled event: {:?}", event);
        }
    }
    None
}

pub fn apply_action_to_fs(path: &Path, action: &WorkspaceAction) {
    let is_open_file = path.join("is_open");
    let url_file = path.join("url");

    match action {
        WorkspaceAction::OpenTab(tab) => fs::write(is_open_file, "1"),
        WorkspaceAction::CloseTab(tab) => fs::write(is_open_file, "0"),
        WorkspaceAction::CreateTab(tab) => fs::create_dir(path),
        WorkspaceAction::RemoveTab(tab) => fs::remove_dir(path),
        WorkspaceAction::ChangeTabUrl(tab, url) => fs::write(url_file, url),
    };
}

impl Workspace {
    pub fn new_from_fs(path: &Path) -> Workspace {
        let tab_dirs = fs::read_dir(path);
        Workspace {
            name: "".to_owned(),
            tabs: tab_dirs
                .unwrap()
                .into_iter()
                .map(|d| Workspace::read_tab_from_dir(&d.unwrap().path()))
                .into_iter()
                .collect(),
            path: path.to_str().unwrap().to_owned(),
        }
    }

    fn read_tab_from_dir(tab_dir: &Path) -> Tab {
        let tab_name = tab_dir.components().last().unwrap();
        let is_open_file = tab_dir.join("is_open");
        let url_file = tab_dir.join("url");
        Tab {
            name: tab_name.as_os_str().to_str().unwrap().to_string(),
            is_open: fs::read_to_string(is_open_file).unwrap() == "1",
            url: fs::read_to_string(url_file).unwrap(),
            id: "".to_owned(),
        }
    }
}
