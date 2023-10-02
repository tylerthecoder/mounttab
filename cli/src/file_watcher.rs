use crate::model::Action;
use futures_util::Stream;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{fs, path::Path};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub fn watch(path: String) -> Result<impl Stream<Item = Action>, notify::Error> {
    let (tx, rx) = mpsc::channel::<Action>(100);
    let path_clone = path.clone();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let event = match res {
                Ok(event) => event,
                Err(e) => {
                    println!("watch error: {:?}", e);
                    return;
                }
            };

            let action = watch_event_to_action(event, &path_clone);

            if let Some(action) = action {
                let res = tx.blocking_send(action);
                if let Err(e) = res {
                    println!("error sending action: {:?}", e);
                }
            }
        },
        Config::default(),
    )?;

    println!("Watching path: {}", path);

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;

    Ok(ReceiverStream::new(rx))
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
                    return Some(Action::OpenTab(path.to_string_lossy().to_string()));
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
