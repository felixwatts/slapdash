mod model;
mod view;
mod controller;
mod db;
mod config;
mod cli;

use std::env;
use std::fs::{File, create_dir_all, write};
use std::io::Read;
use std::path::PathBuf;
use tide_sqlx::SQLxMiddleware;
use sqlx::Sqlite;
use clap::Parser;
use cli::Cli;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::fs;
use std::collections::HashMap;
use crate::cli::Commands;
use crate::cli::DashboardCommands;

const SQLITE_DB_URL: &'static str = "sqlite:slapdash.db?mode=rwc";
const SLAPDASH_XSD: &'static str = include_str!("../dashboard.xsd");
const EMPTY_DASHBOARD: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<column xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="../dashboard.xsd">
    <label text="Hello, world!" width="12" />
</column>
"#;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init()?;

    match cli.command {
        Commands::Serve{listen_addr} => serve(&listen_addr).await?,
        Commands::Dashboard { command } => match command {
            DashboardCommands::New { name } => {
                let msg = new_dashboard(&name)?;
                println!("{}", msg);
            }
        },
    }

    Ok(())
}

async fn serve(listen_addr: &str) -> anyhow::Result<()> {
    let config = load_dashboards()?;
    let dashboard_names = config.keys().cloned().collect::<Vec<String>>();
    let db = init_db().await?;
    let secret = get_secret()?;
    let mut app = tide::with_state((secret.clone(), config));
    app.with(SQLxMiddleware::<Sqlite>::from(db));
    app.at("/").get(controller::get);
    app.at("/:dashboard").get(controller::get);
    app.at("/:secret/:series/:value").get(controller::put);

    println!("Serving at: http://{}/<dashboard>", listen_addr);
    println!("Dashboards: {}", dashboard_names.join(", "));
    println!("Push data: GET http://{}/{}/<series>/<value>", listen_addr, &secret);

    app.listen(listen_addr).await?;
    Ok(())
}

fn get_secret() -> anyhow::Result<String> {
    let home_dir = env::var("HOME")?;
    let config_dir = PathBuf::from(home_dir).join(".slapdash");
    let secret_file = config_dir.join("secret.txt");
    if !secret_file.exists() {
        let secret = generate_secret();
        write(&secret_file, secret)?;
    }
    let secret = fs::read_to_string(secret_file)?;
    Ok(secret)
}

fn generate_secret() -> String {
    let rng = thread_rng();
    // Generate a 64-character alphanumeric string
    let secret: String = rng
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    secret
}

fn new_dashboard(name: &str) -> anyhow::Result<String> {
    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    let config_dir = PathBuf::from(home_dir).join(".slapdash");
    let dashboards_dir = config_dir.join("dashboards");
    let dashboard_file = dashboards_dir.join(format!("{}.xml", name));
    if dashboard_file.exists() {
        return Ok(format!("Dashboard already exists: {}", dashboard_file.display()));
    }
    write(&dashboard_file, EMPTY_DASHBOARD)?;
    Ok(format!("Created a new dashboard at: {}", dashboard_file.display()))
}

fn init() -> anyhow::Result<()> {
    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    let config_dir = PathBuf::from(home_dir).join(".slapdash");
    
    // Check if config directory already exists
    if config_dir.exists() {
        return Ok(());
    }
    
    // Create the main config directory
    create_dir_all(&config_dir)?;
    
    // Create the dashboards subdirectory
    let dashboards_dir = config_dir.join("dashboards");
    create_dir_all(&dashboards_dir)?;
    
    // Create secret.txt file with a cryptographically secure random string
    let secret_file = config_dir.join("secret.txt");
    let secret = generate_secret();
    write(&secret_file, secret)?;
    
    // Write the XSD schema file to the config directory
    let xsd_file = config_dir.join("dashboard.xsd");
    write(&xsd_file, SLAPDASH_XSD)?;
    
    // Create empty default.xml file
    new_dashboard("default")?;

    Ok(())
}

async fn init_db() -> anyhow::Result<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePool::connect(SQLITE_DB_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

fn load_dashboard(file_name: &str) -> anyhow::Result<model::Dashboard> {
    let mut file = File::open(file_name).map_err(anyhow::Error::from)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(anyhow::Error::from)?;
    let config: config::Widget = quick_xml::de::from_str(&contents).map_err(anyhow::Error::from)?;
    let dashboard = config.to_dashboard();
    Ok(dashboard)
}

fn load_dashboards() -> anyhow::Result<HashMap<String, model::Dashboard>> {
    let home_dir = env::var("HOME")?;
    let config_dir = PathBuf::from(home_dir).join(".slapdash");
    let dashboards_dir = config_dir.join("dashboards");
    let mut dashboards = HashMap::new();
    for entry in fs::read_dir(dashboards_dir)? {
        let entry = entry?;
        let file_name = entry.path();
        let file_name = file_name.to_str().unwrap();
        let dashboard = load_dashboard(file_name)?;
        let dashboard_name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
        dashboards.insert(dashboard_name, dashboard);
    }
    Ok(dashboards)
}