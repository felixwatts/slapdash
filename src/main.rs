#[macro_use]
extern crate dotenv_codegen;

mod model;
mod view;
mod controller;
mod db;
use std::fs::File;
use std::io::Read;
use sqlx::postgres::Postgres;
use tide_sqlx::SQLxMiddleware;


#[async_std::main]
async fn main() -> tide::Result<()> {
    let config = load_config()?;

    let mut app = tide::with_state(config);

    app.with(SQLxMiddleware::<Postgres>::new(dotenv!("DATABASE_URL")).await?);

    app.at("/").get(controller::get);
    app.at("/:secret/:series/:value").get(controller::put);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn load_config() -> tide::Result<model::Config> {
    let mut file = File::open("slapdash.json").map_err(|e| tide::Error::from_display(e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| tide::Error::from_display(e))?;
    Ok(serde_json::from_str(&contents).map_err(|e| tide::Error::from_display(e))?)
}