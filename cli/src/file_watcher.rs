use crate::model::Action;
use notify::{RecursiveMode, Watcher};
use std::fs;
use tokio::sync::mpsc;

pub async fn async_watch(path: String, action_tx: mpsc::Sender<Action>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let path_clone = path.clone();

    let mut watcher = notify::recommended_watcher(tx)?;

    println!("Watcher starting");

    let path = "/home/tylord/dev/tabfs-rs/test";

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&path.as_ref(), RecursiveMode::Recursive)?;

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

fn watch_event_to_action(event: notify::Event, base_path: &String) -> Option<Action> {
    match event.kind {
        notify::event::EventKind::Create(_) => {
            let canonical_path = fs::canonicalize(base_path);

            // println!("canonical_path: {:?} {:?}", base_path, canonical_path);

            if let Ok(canonical_path) = canonical_path {
                let path = event.paths[0].strip_prefix(canonical_path);
                // println!("path: {:?} {:?}", path, base_path);

                if let Ok(path) = path {
                    return Some(Action::OpenTab(path.to_string_lossy().to_string()));
                }
            }
        }
        notify::event::EventKind::Remove(_) => {
            let canonical_path = fs::canonicalize(base_path);

            // println!("canonical_path: {:?} {:?}", base_path, canonical_path);

            if let Ok(canonical_path) = canonical_path {
                let path = event.paths[0].strip_prefix(canonical_path);
                // println!("path: {:?} {:?}", path, base_path);

                if let Ok(path) = path {
                    return Some(Action::CloseTab(path.to_string_lossy().to_string()));
                }
            }
        }
        _ => {
            println!("unhandled event: {:?}", event);
        }
    }

    None
}
