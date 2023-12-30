use clap::{Parser, Subcommand};

use crate::app::start_app;
pub mod app;
pub mod browser;
pub mod json_storage;
pub mod model;
pub mod web_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start { path: Option<String> },
}

#[tokio::main]
async fn main() {
    println!("⛰️  Mount Tab");

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start { path }) => {
            println!("Starting");

            // convert path to a PathBuf
            let path = match path {
                Some(p) => std::path::PathBuf::from(p),
                None => std::env::current_dir().unwrap(),
            };

            match start_app(path).await {
                Ok(d) => {
                    println!("Program ended");
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        None => {
            println!("No command");
        }
    }
}
