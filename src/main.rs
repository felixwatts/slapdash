mod model;
mod view;
mod controller;
mod db;
mod env;
mod cli;
mod server;

use clap::Parser;
use cli::Cli;
use crate::cli::Commands;
use crate::cli::DashboardCommands;
use crate::env::Dashboards;
use env::Environment;
use server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let env = Environment::load().await?;

    match cli.command {
        Commands::Serve{listen_addr, secret} => Server::serve(&listen_addr, &secret).await?,
        Commands::Dashboard { command } => match command {
            DashboardCommands::New { name } => {
                let msg = Dashboards::new_dashboard(&name)?;
                println!("{msg}");
            }
        },
        Commands::Push { series, value } => push(&env, &series, value).await?,
        Commands::List => {
            let dashboards = env.dashboards.list();
            println!("Dashboards:\n\t{}", dashboards.join("\n\t"));
        }
    }

    Ok(())
}

async fn push(env: &Environment,series: &str, value: f32) -> anyhow::Result<()> {
    let listen_addr = env.settings.listen_addr;
    let secret = &env.settings.secret;
    let url = format!("http://{listen_addr}/{secret}/{series}/{value}");
    let response = reqwest::get(&url).await?;
    match response.status() {
        reqwest::StatusCode::OK => println!("Pushed {value} to {series}"),
        reqwest::StatusCode::BAD_REQUEST => println!("Failed to push data to {series}: {}", response.text().await?),
        _ => println!("Unexpected response from {url}: {}", response.text().await?),
    }
    Ok(())
}