use crate::daemon::start_daemon;

pub mod daemon;
pub mod file_watcher;
pub mod model;

#[tokio::main]
async fn main() {
    println!("⛰️ Mount Tab");

    let _ = start_daemon().await;
}
