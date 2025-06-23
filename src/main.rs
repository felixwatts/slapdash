mod model;
mod view;
mod controller;
mod db;
mod config;
mod cli;

use std::env;
use std::fs::File;
use std::io::Read;
use tide_sqlx::SQLxMiddleware;
use sqlx::Sqlite;

const SQLITE_DB_URL: &'static str = "sqlite:slapdash.db?mode=rwc";

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv::dotenv().ok();

    let listen_addr = expect_env_var("LISTEN_ADDR");
    let secret = expect_env_var("SECRET");

    let config = load_config()?;
    let db = init_db().await?;

    let mut app = tide::with_state((secret, config));

    app.with(SQLxMiddleware::<Sqlite>::from(db));

    app.at("/").get(controller::get);
    app.at("/:secret/:series/:value").get(controller::put);

    app.listen(listen_addr).await?;

    Ok(())
}

async fn init_db() -> tide::Result<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePool::connect(SQLITE_DB_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

fn load_config() -> tide::Result<model::Dashboard> {
    let mut file = File::open("slapdash.xml").map_err(tide::Error::from_display)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(tide::Error::from_display)?;
    let config: config::Widget = quick_xml::de::from_str(&contents).map_err(tide::Error::from_display)?;
    let dashboard = config.to_dashboard();
    Ok(dashboard)
}

fn expect_env_var(name: &'static str) -> String{
    match env::var(name) {
        Ok(val) => val.clone(),
        Err(_) => { panic!("Missing required environment variable: {name}"); }
    }
}