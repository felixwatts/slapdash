mod model;
mod view;
mod controller;
mod db;
mod config;
mod cli;

use std::net::SocketAddr;
use sqlx::Sqlite;
use clap::Parser;
use cli::Cli;
use crate::cli::Commands;
use crate::cli::DashboardCommands;
use crate::config::Dashboards;
use config::Config;
use axum::{
    routing::get,
    Router,
};

const SQLITE_DB_URL: &str = "sqlite:slapdash.db?mode=rwc";

#[derive(Clone)]
struct AppState {
    config: Config,
    db: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Serve{listen_addr, secret} => serve(config, &listen_addr, &secret).await?,
        Commands::Dashboard { command } => match command {
            DashboardCommands::New { name } => {
                let msg = Dashboards::new_dashboard(&name)?;
                println!("{msg}");
            }
        },
        Commands::Push { series, value } => push(&config, &series, value).await?
    }

    Ok(())
}

async fn push(config: &Config,series: &str, value: f32) -> anyhow::Result<()> {
    let listen_addr = config.settings.listen_addr;
    let secret = &config.settings.secret;
    let url = format!("http://{listen_addr}/{secret}/{series}/{value}");
    let response = reqwest::get(&url).await?;
    match response.status() {
        reqwest::StatusCode::OK => println!("Pushed data to {series}"),
        reqwest::StatusCode::BAD_REQUEST => println!("Failed to push data to {series}: {}", response.text().await?),
        _ => println!("Unexpected response from {url}: {}", response.text().await?),
    }
    Ok(())
}

async fn serve(config: Config, listen_addr: &Option<SocketAddr>, secret: &Option<String>) -> anyhow::Result<()> {
    let db = init_db().await?;
    let secret = secret.as_ref().unwrap_or(&config.settings.secret).to_string();
    let dashboard_list = config.dashboards.list().join(", ");
    let listen_addr = listen_addr.unwrap_or(config.settings.listen_addr);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(controller::get))
        .route("/:dashboard", get(controller::get))
        .route("/:secret/:series/:value", get(controller::put))
        .with_state(AppState { config, db });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    println!("Serving at: http://{listen_addr}/(<dashboard>)");
    println!("Dashboards: {dashboard_list}");
    println!("Push data: GET http://{}/{}/<series>/<value>", listen_addr, &secret);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn init_db() -> anyhow::Result<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePool::connect(SQLITE_DB_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}