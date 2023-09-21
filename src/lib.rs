use std::fs;
use model::Action;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config};

pub mod browser_interface;
pub mod model;



fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");


    if let Err(error) = watch(path) {
        println!("Error: {:?}", error)
    }
}

fn watch(path: String) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                let actions = watch_event_to_action(event, &path);
                for action in actions {
                    println!("action: {:?}", action);
                }
            }
            Err(error) => println!("watch error: {:?}", error),
        }
    }

    Ok(())
}



fn watch_event_to_action(event: notify::Event, base_path: &String) -> Option<Action> {
    match event.kind {
        notify::event::EventKind::Create(_) => {

            let canonical_path = fs::canonicalize(base_path);

            println!("canonical_path: {:?} {:?}", base_path, canonical_path);

            if let Ok(canonical_path) = canonical_path {
                let path = event.paths[0].strip_prefix(canonical_path);
                println!("path: {:?} {:?}", path, base_path);

                if let Ok(path) = path {
                    return Some(Action::OpenTab(path.to_string_lossy().to_string()))
                }
            }


        }
        notify::event::EventKind::Remove(_) => {
            let canonical_path = fs::canonicalize(base_path);

            println!("canonical_path: {:?} {:?}", base_path, canonical_path);

            if let Ok(canonical_path) = canonical_path {
                let path = event.paths[0].strip_prefix(canonical_path);
                println!("path: {:?} {:?}", path, base_path);

                if let Ok(path) = path {
                    return Some(Action::CloseTab(path.to_string_lossy().to_string()))
                }
            }

        }
        _ => {
            println!("unhandled event: {:?}", event);
        }
    }


    None
}



