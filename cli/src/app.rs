use crate::browser::start_browser;
use crate::json_storage::start_file_watcher;
use crate::model::{Workspace, WorkspaceManger};
use crate::web_server::start_web_server;
use std::path::PathBuf;

pub async fn start_app(path: PathBuf) -> Result<(), std::io::Error> {
    let workspace = Workspace::from_path(&path);

    println!("Workspace: {:?}", workspace);

    let workman = WorkspaceManger::new(workspace);

    match start_browser(&workman).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error starting browser: {}", err);
        }
    }

    start_file_watcher(&workman, path);

    start_web_server(workman).await;

    Ok(())
}
