mod model;
mod view;
mod controller;
mod db;
mod env;
mod cli;

use std::net::SocketAddr;
use clap::Parser;
use cli::Cli;
use crate::cli::Commands;
use crate::cli::DashboardCommands;
use crate::env::Dashboards;
use env::Environment;
use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let env = Environment::load().await?;

    match cli.command {
        Commands::Serve{listen_addr, secret} => serve(env, &listen_addr, &secret).await?,
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

async fn serve(env: Environment, listen_addr: &Option<SocketAddr>, secret: &Option<String>) -> anyhow::Result<()> {
    let secret = secret.as_ref().unwrap_or(&env.settings.secret).to_string();
    let dashboard_list = env.dashboards.list();
    let listen_addr = listen_addr.unwrap_or(env.settings.listen_addr);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(controller::get_default))
        .route("/{dashboard}", get(controller::get))
        .route("/{secret}/{series}/{value}", get(controller::put))
        .with_state(env);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    println!("Serving at: http://{listen_addr}/(<dashboard>)");
    println!("Dashboards:\n\t{}", &dashboard_list.join("\n\t"));
    println!("Push data: GET http://{}/{}/<series>/<value>", listen_addr, &secret);

    axum::serve(listener, app).await?;

    Ok(())
}