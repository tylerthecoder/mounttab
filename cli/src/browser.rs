use chromiumoxide::browser::Browser;
use std::{io::Read, process::Command};
use tokio_stream::StreamExt;

use crate::model::{Workspace, WorkspaceManger};

impl Workspace {
    pub async fn from_browser(browser: &Browser) -> Result<Workspace, Box<dyn std::error::Error>> {
        let pages = browser.pages().await?;
        let mut tabs = vec![];

        for page in pages {
            let url = page.url().await?.ok_or("no_url")?;
            tabs.push(url);
        }

        Ok(Workspace { tabs })
    }
}

pub async fn start_browser(workman: &WorkspaceManger) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting browser");

    let mut chrome_command = Command::new("chromium")
        .arg("--profile-directory='Profile 1'")
        .arg("--remote-debugging-port=9222")
        .spawn()?;

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Read the chrome command's stdout every 100 ms
    // wait for the chrome command to output the word "DevTools"
    // loop {
    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    //
    //     let mut output_str = String::new();
    //     if let Some(mut output) = chrome_command.stdout.take() {
    //         let _ = output.read_to_string(&mut output_str);
    //     }
    //     println!("Waiting for chrome to start: {} |||", output_str);
    //     if output_str.contains("DevTools") {
    //         break;
    //     }
    // }

    let (browser, mut handler) = chromiumoxide::Browser::connect("http://127.0.0.1:9222").await?;

    let handle = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    // get all tabs and open them
    let tabs = workman.workspace.read().await.tabs.clone();

    for tab in tabs {
        println!("Opening tab {}", tab);
        browser.new_page(&tab).await?;
    }

    println!("Tabs opeened");

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let current_workspace = Workspace::from_browser(&browser).await?;

        println!("\n Browser workspace: {:?}", current_workspace);

        let mut workspace = workman.workspace.write().await;

        println!("Current workspace: {:?}", workspace);

        let actions = workspace.actions_from_diff(current_workspace);

        for action in actions {
            println!("Applying action {:?}", action);
            workspace.apply_action(action);
        }
    }

    // let _ = handle.await;

    // Ok(())
}
