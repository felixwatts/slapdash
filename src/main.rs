mod model;
mod view;
mod controller;
mod db;
mod env;
mod cli;
mod server;

use std::path::PathBuf;
use anyhow::anyhow;
use chrono::NaiveDateTime;
use clap::Parser;
use cli::Cli;
use crate::cli::Commands;
use crate::cli::DashboardCommands;
use crate::env::Dashboards;
use env::Environment;
use server::Server;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        },
        Commands::PushAll { filename } => {
            push_all(&env, filename).await?;
        }
    }

    Ok(())
}

async fn push_all(env: &Environment, filename: PathBuf) -> anyhow::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let points: anyhow::Result<Vec<_>> = reader
        .lines()
        .enumerate()
        .map(|(line_num, line)| -> anyhow::Result<(String, NaiveDateTime, f32)>{
            let line = line?;
            
            let cols: Vec<_> = line.split(',').collect();
            if cols.len() != 3 {
                return Err(anyhow!("At line {}. Invalid format. Each row of the CSV file should contain 3 columns: series, time and point.", &line_num))
            }

            let series = cols[0].to_string();
            let time = NaiveDateTime::parse_from_str(cols[1], "%Y-%m-%d %H:%M:%S").map_err(|_| anyhow!("At line {}. Invalid format. The time column must be formatted as: 2024-06-13 15:30:00", &line_num))?;
            let value: f32 = cols[2].parse().map_err(|_| anyhow!("At line {}. Invalid format. The value column must parse as an f32", &line_num))?;

            Ok((series, time, value))
        })
        .collect();

    let points = points?;

    let mut db = env.db.acquire().await?;

    db::put_all(&mut db, points).await
}

async fn push(env: &Environment, series: &str, value: f32) -> anyhow::Result<()> {
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