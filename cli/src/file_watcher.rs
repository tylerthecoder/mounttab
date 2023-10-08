use crate::model::Action;
use notify::{RecursiveMode, Watcher};
use std::fs;
use tokio::sync::mpsc;

pub async fn async_watch(path: String, action_tx: mpsc::Sender<Action>) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::channel::<Action>(100);

    let path_clone = path.clone();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("got event: {:?}", event);
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let action = match watch_event_to_action(event, &path_clone) {
                    Some(action) => {
                        println!("Action received: {:?}", action);
                        action
                    }
                    None => {
                        println!("No action for event");
                        return ();
                    }
                };

                let _send_res = tx.send(action).await;
            });
        }
        Err(e) => {
            println!("watch error: {:?}", e);
        }
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(action) = rx.recv().await {
        println!(
            "Received action from file watcher in file watcher: {:?}",
            action
        );
        let _ = action_tx.send(action).await;
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
