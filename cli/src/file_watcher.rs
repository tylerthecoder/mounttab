use crate::model::{Tab, Workspace, WorkspaceAction};
use notify::{RecursiveMode, Watcher};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Component, Path},
};
use tokio::sync::mpsc;

pub async fn async_watch(
    path: &Path,
    action_tx: mpsc::Sender<WorkspaceAction>,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let path_clone = path.clone();

    let mut watcher = notify::recommended_watcher(tx)?;

    println!("Watcher starting: {}", path.to_str().unwrap());

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

        let actions = watch_event_to_actions(event, &path_clone);

        println!("Actions received: {:?}", actions);

        for action in actions {
            match action_tx.send(action).await {
                Ok(_res) => {}
                Err(e) => {
                    println!("Error sending action: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

fn watch_event_to_actions(event: notify::Event, base_path: &Path) -> Vec<WorkspaceAction> {
    let canonical_path = match fs::canonicalize(base_path) {
        Ok(path) => path,
        Err(e) => {
            println!("Error canonicalizing path: {:?}", e);
            return vec![];
        }
    };

    println!("canonical_path: {:?} {:?}", base_path, canonical_path);

    match event.kind {
        notify::event::EventKind::Create(_) => {
            let actions = event
                .paths
                .iter()
                .filter_map(|path| {
                    let workspace_path = path.strip_prefix(&canonical_path).ok()?;

                    let mut comps = workspace_path.components();
                    let tab = comps.next()?;

                    let tab_name = match tab {
                        std::path::Component::Normal(tab_name) => {
                            Some(tab_name.to_str().unwrap().to_string())
                        }
                        _ => None,
                    }?;

                    let file_name = match comps.next() {
                        Some(Component::Normal(file_name)) => Some(file_name),
                        None => {
                            println!("No filename");
                            None
                        }
                        _ => None,
                    };

                    // Just the directory was made
                    if file_name == None {
                        return Some(WorkspaceAction::CreateTab(tab_name));
                    }

                    println!("No action found");

                    None
                })
                .collect();

            return actions;
        }

        notify::EventKind::Modify(_) => {
            let actions = event
                .paths
                .iter()
                .filter_map(|path| {
                    let workspace_path = path.strip_prefix(&canonical_path).ok()?;

                    let mut comps = workspace_path.components();
                    let tab = comps.next()?;

                    let tab_name = match tab {
                        std::path::Component::Normal(tab_name) => {
                            Some(tab_name.to_str().unwrap().to_string())
                        }
                        _ => None,
                    }?;

                    println!("Tab name: {}", tab_name);

                    let file_name = match comps.next() {
                        Some(Component::Normal(file_name)) => {
                            println!("Got file: {}", file_name.to_str().unwrap().to_string());
                            Some(file_name)
                        }
                        None => {
                            println!("No filename");
                            None
                        }
                        _ => {
                            println!("Some other component");
                            None
                        }
                    };

                    if file_name == Some(OsStr::new("is_open")) {
                        let is_open = fs::read_to_string(path).ok()?;
                        let is_open = is_open.trim();

                        println!("Is open contents: {}", is_open.to_owned());

                        if is_open == "1".to_string() {
                            return Some(WorkspaceAction::OpenTab(tab_name));
                        } else if is_open == "0".to_string() {
                            return Some(WorkspaceAction::CloseTab(tab_name));
                        }
                    } else if file_name == Some(OsStr::new("url")) {
                        let tab_url = fs::read_to_string(path).ok()?;
                        let tab_url = tab_url.trim().to_string();

                        return Some(WorkspaceAction::ChangeTabUrl(tab_name, tab_url));
                    }

                    println!("No action found");

                    None
                })
                .collect();

            return actions;
        }
        notify::event::EventKind::Remove(_) => {
            let actions = event
                .paths
                .iter()
                .filter_map(|path| {
                    let workspace_path = path.strip_prefix(&canonical_path).ok()?;

                    let mut comps = workspace_path.components();
                    let tab = comps.next()?;

                    let tab_name = match tab {
                        std::path::Component::Normal(tab_name) => {
                            Some(tab_name.to_str().unwrap().to_string())
                        }
                        _ => None,
                    }?;

                    let file_name = match comps.next() {
                        Some(Component::Normal(file_name)) => Some(file_name),
                        None => {
                            println!("No filename");
                            None
                        }
                        _ => None,
                    };

                    // Just the directory was made
                    if file_name == None {
                        return Some(WorkspaceAction::RemoveTab(tab_name));
                    }

                    println!("No action found");

                    None
                })
                .collect();

            return actions;
        }
        _ => {
            println!("unhandled event");
        }
    }

    vec![]
}

pub fn apply_action_to_fs(path: &Path, action: &WorkspaceAction) -> io::Result<()> {
    match action {
        WorkspaceAction::OpenTab(tab) => {
            let dir_path = path.join(tab);
            let is_open_file = dir_path.join("is_open");
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            fs::write(is_open_file, "1")?;
        }
        WorkspaceAction::CloseTab(tab) => {
            let dir_path = path.join(tab);
            let is_open_file = dir_path.join("is_open");
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            fs::write(is_open_file, "0")?;
        }
        WorkspaceAction::CreateTab(tab) => {
            let dir_path = path.join(tab);
            let is_open_file = dir_path.join("is_open");
            let url_file = dir_path.join("url");
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            fs::write(is_open_file, "0")?;
            fs::write(url_file, "")?;
        }
        WorkspaceAction::RemoveTab(tab) => {
            let dir_path = path.join(tab);
            fs::remove_dir(dir_path)?;
        }
        WorkspaceAction::ChangeTabUrl(tab, url) => {
            let dir_path = path.join(tab);
            let url_file = dir_path.join("url");
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            fs::write(url_file, url)?;
        }
    };
    Ok(())
}

impl Workspace {
    pub fn new_from_fs(path: &Path) -> Workspace {
        let tab_dirs = fs::read_dir(path);
        Workspace {
            id: "".to_owned(),
            name: "Testing".to_owned(),
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
