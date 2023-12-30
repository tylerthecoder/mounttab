use crate::model::{Workspace, WorkspaceManger};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{fs, path::PathBuf};

impl Workspace {
    pub fn from_path(path: &PathBuf) -> Workspace {
        let browser_tabs_path = path.join("browser-tabs.json");

        // If the file exists, read it, otherwise create a new one
        if browser_tabs_path.exists() {
            let file = fs::File::open(browser_tabs_path).unwrap();
            let workspace: Workspace = serde_json::from_reader(file).unwrap();
            workspace
        } else {
            let workspace = Workspace { tabs: Vec::new() };
            let _ = workspace.save_to_path(&path);
            workspace
        }
    }

    pub fn save_to_path(&self, path: &PathBuf) -> Result<(), ()> {
        let browser_tabs_path = path.join("browser-tabs.json");

        let seralized_workspace = serde_json::to_string(self)
            .map_err(|err| eprintln!("Error serializing workspace: {}", err))?;

        fs::write(browser_tabs_path, seralized_workspace)
            .map_err(|err| eprintln!("Error writing workspace to file: {}", err))?;

        Ok(())
    }
}

pub fn start_file_watcher(workman: &WorkspaceManger, path: PathBuf) {
    let path_clone = path.clone();
    let path_clone_2 = path.clone();
    let workman_clone = workman.clone();
    let workman_clone_2 = workman.clone();
    tokio::task::spawn(async move {
        listen_for_actions(&workman_clone, path_clone).await;
    });
    tokio::task::spawn(async move {
        let _ = watch_file(&workman_clone_2, path_clone_2).await;
    });
}

async fn listen_for_actions(workman: &WorkspaceManger, path: PathBuf) {
    println!("Listening for actions");
    while let Ok((source, action)) = workman.tx.subscribe().recv().await.map_err(|err| {
        eprintln!("Error receiving message from workspace manager: {}", err);
    }) {
        println!("Got action {:?} from: {}", action, source);

        if source == "fs" {
            continue;
        }

        let mut workspace = workman.workspace.write().await;

        workspace.apply_action(action);

        workspace.save_to_path(&path).unwrap_or_else(|_| {
            eprintln!("Error saving workspace to file");
        });
    }
    println!("Done printing actions");
}

async fn watch_file(workman: &WorkspaceManger, path: PathBuf) -> Result<(), ()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                tx.blocking_send(event).expect("send event");
            }
        },
        notify::Config::default(),
    )
    .map_err(|err| eprintln!("Error creating watcher: {}", err))?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|err| {
            eprintln!("Error watching path: {}", err);
        })?;

    println!("Watcher started");

    while let Some(_event) = rx.recv().await {
        println!("Got event: {:?}", _event);

        let new_workspace = Workspace::from_path(&path);
        let workspace = workman.workspace.read().await;
        let actions = workspace.actions_from_diff(new_workspace);
        for action in actions {
            println!("Sending action to workspace manager: {:?}", action);

            let count = workman.tx.send(("fs", action)).map_err(|err| {
                eprintln!("Error sending message to workspace manager: {}", err);
            })?;

            println!("Num recieved: {:?}", count);
        }
    }

    Ok(())
}
