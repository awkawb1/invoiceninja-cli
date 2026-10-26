mod body;
mod cli;
mod commands;
mod config;
mod output;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = config::build_client(&cli)?;

    match cli.command {
        Command::Clients(args) => commands::clients::run(&client, args, cli.output).await,
    }
}
