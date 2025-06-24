mod model;
mod view;
mod controller;
mod db;
mod config;
mod cli;

use std::net::SocketAddr;
use tide_sqlx::SQLxMiddleware;
use sqlx::Sqlite;
use clap::Parser;
use cli::Cli;
use crate::cli::Commands;
use crate::cli::DashboardCommands;
use crate::config::Dashboards;
use config::Config;

const SQLITE_DB_URL: &str = "sqlite:slapdash.db?mode=rwc";


#[async_std::main]
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
    }

    Ok(())
}

async fn serve(config: Config, listen_addr: &Option<SocketAddr>, secret: &Option<String>) -> anyhow::Result<()> {
    let db = init_db().await?;
    let secret = secret.as_ref().unwrap_or(&config.settings.secret).to_string();
    let dashboard_list = config.dashboards.list().join(", ");
    let listen_addr = listen_addr.unwrap_or(config.settings.listen_addr);
    let mut app = tide::with_state(config);
    app.with(SQLxMiddleware::<Sqlite>::from(db));
    app.at("/").get(controller::get);
    app.at("/:dashboard").get(controller::get);
    app.at("/:secret/:series/:value").get(controller::put);

    println!("Serving at: http://{listen_addr}/(<dashboard>)");
    println!("Dashboards: {dashboard_list}");
    println!("Push data: GET http://{}/{}/<series>/<value>", listen_addr, &secret);

    app.listen(listen_addr).await?;
    Ok(())
}

async fn init_db() -> anyhow::Result<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePool::connect(SQLITE_DB_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}