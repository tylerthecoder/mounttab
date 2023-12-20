use crate::model2::{Workspace, WorkspaceComs};
use notify::{RecursiveMode, Watcher};
use std::{fs, path::Path};

impl Workspace {
    pub fn from_path(path: &Path) -> Workspace {
        let browser_tabs_path = path.join("browser-tabs.json");

        // If the file exists, read it, otherwise create a new one
        if browser_tabs_path.exists() {
            let file = fs::File::open(browser_tabs_path).unwrap();
            let workspace: Workspace = serde_json::from_reader(file).unwrap();
            workspace
        } else {
            let workspace = Workspace { tabs: Vec::new() };
            workspace.save_to_path(path);
            workspace
        }
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), ()> {
        let browser_tabs_path = path.join("browser-tabs.json");

        let seralized_workspace = serde_json::to_string(self).map_err(|err| {
            eprintln!("Error serializing workspace: {}", err);
            ()
        })?;

        fs::write(browser_tabs_path, seralized_workspace).map_err(|err| {
            eprintln!("Error writing workspace to file: {}", err);
            ()
        })?;

        Ok(())
    }

    pub async fn listen_for_actions(&mut self, path: &Path, coms: WorkspaceComs) {
        while let Ok((source, action)) = coms.tx.subscribe().recv().await {
            if source == "fs" {
                continue;
            }

            self.apply_action(action);

            self.save_to_path(path).unwrap_or_else(|_| {
                eprintln!("Error saving workspace to file");
            });
        }
    }

    pub fn watch(&self, path: &Path, coms: WorkspaceComs) -> Result<(), ()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let path_clone = path.clone();

        notify::recommended_watcher(tx)
            .map_err(|err| {
                eprintln!("Error creating watcher: {}", err);
                ()
            })?
            .watch(path, RecursiveMode::Recursive)
            .map_err(|err| {
                eprintln!("Error watching path: {}", err);
                ()
            })?;

        println!("Watcher started");

        rx.iter().for_each(|event| {
            let new_workspace = Workspace::from_path(&path_clone);
            let actions = self.actions_from_diff(new_workspace);
            for action in actions {
                coms.tx.send(("fs", action));
            }
        });

        Ok(())
    }
}
