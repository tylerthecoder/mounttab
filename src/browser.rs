extern crate ctrlc;
use chromiumoxide::{browser::Browser, cdp::browser_protocol::target::CreateTargetParams};
use std::{
    io::Read,
    process::{self, Command},
};
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
    // Step 1, try to commect to the current browser

    let res = chromiumoxide::Browser::connect("http://127.0.0.1:9222").await;

    match res {
        Ok((browser, mut handler)) => {
            println!("Connecting to existing browser and opening new tab");
            // Open a new tab in a new window

            tokio::task::spawn(async move {
                loop {
                    let _ = handler.next().await.unwrap();
                }
            });

            let new_page_params = CreateTargetParams::builder()
                .url("http://localhost:9222")
                .new_window(true)
                .build();

            match new_page_params {
                Ok(params) => {
                    println!("new_page_params: {:?}", params);
                    match browser.new_page(params).await {
                        Ok(_) => {
                            println!("New tab opened");
                        }
                        Err(err) => {
                            println!("Error opening new tab: {}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Error creating new_page_params: {}", err);
                }
            }
        }
        Err(_) => {
            println!("No browser found, starting new browser")
        }
    }

    println!("Starting new browser");

    // Trying to connect to a browser

    // If no browser is open, then

    let tabs = workman.workspace.read().await.tabs.clone();

    let mut chrome_command = Command::new("chromium");

    chrome_command
        .arg("--remote-debugging-port=9222")
        .arg("--new-window");

    for tab in tabs {
        chrome_command.arg(tab);
    }

    // Print the command
    println!("{:?}", chrome_command);

    let mut chrome_ps = chrome_command.spawn()?;

    println!("Browser started");
    let _ = ctrlc::set_handler(move || {
        println!("ctrlc handler");
        chrome_ps.kill().unwrap();
        process::exit(0);
    });

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

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
}
