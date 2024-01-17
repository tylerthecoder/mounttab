extern crate ctrlc;
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

pub fn start_browser(workman: &WorkspaceManger) {
    println!("Starting browser");
    let workman = workman.clone();
    tokio::spawn(async move {
        match start_browser_inner(&workman).await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error starting browser: {}", err);
            }
        }
    });
}

pub async fn start_browser_inner(
    workman: &WorkspaceManger,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut chrome_ps = Command::new("chromium")
        .arg("--profile-directory='Profile 1'")
        .arg("--remote-debugging-port=9222")
        .spawn()?;

    println!("Browser started");
    let _ = ctrlc::set_handler(move || {
        println!("ctrlc handler");
        chrome_ps.kill().unwrap();
    });

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

    // keep trying to connect until we are successful

    let browser;
    let mut handler;

    loop {
        let res = chromiumoxide::Browser::connect("http://127.0.0.1:9222").await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        match res {
            Ok(b) => {
                (browser, handler) = b;
                break;
            }
            Err(err) => {
                println!("Error connecting to browser: {}", err);
            }
        }
    }

    tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let tabs = workman.workspace.read().await.tabs.clone();

    for tab in tabs {
        println!("Opening tab {}", tab);
        browser.new_page(&tab).await?;
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let current_workspace = Workspace::from_browser(&browser).await?;
        let mut workspace = workman.workspace.write().await;

        let actions = workspace.actions_from_diff(current_workspace);

        for action in actions {
            workspace.apply_action(action.clone());
            workman.tx.send(("browser", action))?;
        }
    }

    println!("Closing browser");

    chrome_ps.kill()?;

    // let _ = handle.await;

    // Ok(())
}
