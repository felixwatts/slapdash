mod model;
mod view;
mod controller;
mod db;
use std::env;
use std::fs::File;
use std::io::Read;
use sqlx::postgres::Postgres;
use tide_sqlx::SQLxMiddleware;

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv::dotenv().ok();


    let database_url = expect_env_var("DATABASE_URL");
    let listen_addr = expect_env_var("LISTEN_ADDR");
    let secret = expect_env_var("SECRET");

    let config = load_config()?;

    let mut app = tide::with_state((secret, config));

    app.with(SQLxMiddleware::<Postgres>::new(&database_url).await?);

    app.at("/").get(controller::get);
    app.at("/:secret/:series/:value").get(controller::put);

    app.listen(listen_addr).await?;
    Ok(())
}

fn load_config() -> tide::Result<model::Config> {
    let mut file = File::open("slapdash.json").map_err(tide::Error::from_display)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(tide::Error::from_display)?;
    serde_json::from_str(&contents).map_err(tide::Error::from_display)
}

fn expect_env_var(name: &'static str) -> String{
    match env::var(name) {
        Ok(val) => val.clone(),
        Err(_) => { panic!("Missing required environment variable: {name}"); }
    }
}