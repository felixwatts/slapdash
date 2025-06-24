use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use regex::Regex;

/// Validates that a string is a valid socket address
fn validate_socket_addr(addr: &str) -> Result<String, String> {
    addr.parse::<SocketAddr>()
        .map(|_| ())
        .map_err(|e| format!("Invalid socket address '{addr}': {e}"))?;
    Ok(addr.to_string())
}

/// Validates that a string contains only lowercase letters, underscores, and hyphens
fn validate_dashboard_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Dashboard name cannot be empty".to_string());
    }
    
    if !Regex::new("^[a-z0-9_-]+$").unwrap().is_match(name){
        return Err("Dashboard name must contain only lowercase letters, underscores, and hyphens".to_string());
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
        /// A secret string. Anyone who knows or guesses this string can push data to the dashboard.
        #[arg(short, long, value_parser = validate_socket_addr)]
        secret: Option<String>,
    },
    
    /// Dashboard management commands
    Dashboard {
        #[command(subcommand)]
        command: DashboardCommands,
    },
}

#[derive(Subcommand)]
pub enum DashboardCommands {
    /// Create a new dashboard
    New {
        /// Name of the dashboard (lowercase letters, underscores, and hyphens only)
        #[arg(value_parser = validate_dashboard_name)]
        name: String,
    },
}
