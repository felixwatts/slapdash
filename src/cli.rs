use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
use regex::Regex;

/// Validates that a string is a valid socket address
fn validate_socket_addr(addr: &str) -> Result<String, String> {
    addr.parse::<SocketAddr>()
        .map(|_| ())
        .map_err(|e| format!("Invalid socket address '{addr}': {e}"))?;
    Ok(addr.to_string())
}

/// Validates that a string contains only lowercase letters, underscores, and hyphens
fn validate_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Cannot be empty".to_string());
    }
    
    if !Regex::new("^[a-z0-9_-]+$").unwrap().is_match(name){
        return Err("Must contain only lowercase letters, underscores, and hyphens".to_string());
    }
    
    Ok(name.to_string())
}

#[derive(Parser)]
#[command(name = "slapdash")]
#[command(about = "A dashboard and monitoring tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the server
    Serve {
        /// Listen address (e.g., 127.0.0.1:8080, [::1]:8080)
        #[arg(short, long, value_parser = validate_socket_addr)]
        listen_addr: Option<SocketAddr>,
        /// A secret string. Anyone who knows or guesses this string can push data to the dashboard
        #[arg(short, long)]
        secret: Option<String>,
    },
    
    /// Dashboard management commands
    Dashboard {
        #[command(subcommand)]
        command: DashboardCommands,
    },

    /// Push a data point to the dashboard
    Push{
        /// The name of the series that the data point belongs to
        #[arg(value_parser = validate_name)]
        series: String,
        /// The data point, a number
        value: f32
    },

    /// Push multiple data points to the dashboard from a CSV file.
    #[command(
        long_about = "Push multiple data points to the dashboard from a CSV file.\n\
The CSV file should contain columns: series, time, and value. Each row represents a data point to be pushed. There should be no header row. \
The 'series' column specifies the series name, 'time' is the timestamp and must be formatted YYYY-MM-dd HH:mm:ss, and 'value' is the data point value and must parse as an f32. \
This is an example row:\n\n
my_example_series,2024-06-13 15:30:00,32.14
"
    )]
    PushAll {
        /// CSV filename. The CSV should contain columns: series, time and value.
        filename: PathBuf
    },

    /// List the dashboards
    List,
}

#[derive(Subcommand)]
pub enum DashboardCommands {
    /// Create a new dashboard
    New {
        /// Name of the dashboard (lowercase letters, underscores, and hyphens only)
        #[arg(value_parser = validate_name)]
        name: String,
    },
}
