// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301
// USA

//! # main.rs
//!
//! Everything starts here.
//!
//! proxmox-pvenom: inspect and operate your ProxMox clusters from
//! the CLI with no API keys.
//! Copyright (C) 2025 Francesco Garbin
//!

use clap::Parser;
use anyhow::{bail, Result};
use std::env;
mod client;
use client::ProxmoxClient;
mod models;
mod commands;
mod vlog;

/// Proxmox Virtual Environment Node Observability Monitor
#[derive(Parser)]
#[command(name = "pvenom")]
#[command(author = "Francesco - GameVision Italia CTO")]
#[command(version = "0.1.0")]
#[command(about = "Monitor and observe Proxmox VE cluster nodes, VMs and LXC containers", long_about = None)]
struct Cli {
    /// Proxmox cluster controller IP or hostname
    #[arg(short = 'c', long = "controller", required = true)]
    controller: String,

    /// Username for authentication (e.g., root@pam)
    #[arg(short = 'u', long = "username", default_value = "root@pam")]
    username: String,

    /// Password for authentication
    #[arg(short = 'p', long = "password", env = "PVENOM_PASSWORD")]
    password: String,

    /// Use SSL certificate verification (yes or no)
    #[arg(short = 's', long = "secure", default_value = "yes", value_parser = parse_yes_no, num_args = 1)]
    secure: bool,

    /// Specify node name for operations (optional - lists all nodes if omitted)
    #[arg(short = 'n', long = "node")]
    node: Option<String>,

    /// Output format: json, csv, or table
    #[arg(short = 'f', long = "format", default_value = "table", value_parser = parse_format)]
    format: models::OutputFormat,

    /// Enable verbose debug logging
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

/// Parse yes/no values for --secure flag
fn parse_yes_no(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => Err(format!("Invalid value '{}'. Expected 'yes' or 'no'", s)),
    }
}

/// Parse format values for --format flag
fn parse_format(s: &str) -> Result<models::OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "json" => Ok(models::OutputFormat::Json),
        "csv" => Ok(models::OutputFormat::Csv),
        "table" => Ok(models::OutputFormat::Table),
        _ => Err(format!("Invalid format '{}'. Expected 'json', 'csv', or 'table'", s)),
    }
}

/// Try to build a working base URL with protocol auto-detection
/// Tries HTTPS first, falls back to HTTP if needed
async fn resolve_base_url(controller: &str, username: &str, password: &str, secure: bool) -> Result<String> {
    // If user already specified protocol, use it as-is
    if controller.starts_with("http://") || controller.starts_with("https://") {
        vlog_debug!("Protocol already specified in controller address: {}", controller);
        return Ok(controller.to_string());
    }

    // Try HTTPS first, assuming that production clusters have SSL certificates
    let https_url = format!("https://{}", controller);
    vlog_info!("Attempting HTTPS connection to {}...", controller);

    if try_connection(&https_url, username, password, secure).await.is_ok() {
        vlog_success!("HTTPS connection established to {}", controller);
        return Ok(https_url);
    }

    // Fall back to HTTP, providing support for homelabs with no public SSL certificates
    vlog_warn!("HTTPS connection failed, attempting HTTP fallback...");
    let http_url = format!("http://{}", controller);

    if try_connection(&http_url, username, password, secure).await.is_ok() {
        vlog_warn!("HTTP connection successful - consider using HTTPS in production!");
        return Ok(http_url);
    }

    vlog_error!("Could not connect to {} via HTTPS or HTTP", controller);
    bail!("Failed to establish connection to Proxmox cluster");
}

/// Quick connection test to check if the endpoint is reachable
async fn try_connection(base_url: &str, username: &str, password: &str, secure: bool) -> Result<()> {
    vlog_debug!("Testing connection to {}", base_url);

    // Build a minimal reqwest client just for testing
    // When secure=true, verify certs; when secure=false, skip verification (danger!)
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(!secure)
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // Try to hit the ticket endpoint
    let url = format!("{}/api2/json/access/ticket", base_url);
    let response = client.post(&url)
        .form(&[
            ("username", username),
            ("password", password),
        ])
        .send()
        .await?;

    if response.status().is_success() {
        vlog_debug!("Connection test successful");
        Ok(())
    } else {
        vlog_debug!("Connection test failed with status: {}", response.status());
        bail!("Connection test failed");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set log level based on verbose flag
    if cli.verbose {
        vlog::set_level(vlog::LogLevel::Debug);
        vlog_debug!("Verbose logging enabled");
    }
    vlog_debug!("--controller: {}", &cli.controller);
    vlog_debug!("--username: {}", &cli.username);
    vlog_debug!("--password: {}", &cli.password);

    vlog_info!("Proxmox VE Node Observability Monitor v{}", env!("CARGO_PKG_VERSION"));

    // Resolve base URL with auto-detection (hidden ugliness under Persian carpets!)
    vlog_info!("Connecting to Proxmox cluster at {}...", cli.controller);
    let base_url = match resolve_base_url(&cli.controller, &cli.username, &cli.password, cli.secure).await {
        Ok(url) => url,
        Err(e) => {
            vlog_error!("Connection failed: {}", e);
            std::process::exit(1);
        }
    };

    // Create Proxmox client and authenticate
    vlog_info!("Authenticating to Proxmox API...");
    let client = match ProxmoxClient::new(&base_url, &cli.username, &cli.password, cli.secure).await {
        Ok(c) => {
            vlog_success!("Authentication successful!");
            c
        },
        Err(e) => {
            vlog_error!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    };

    // Execute the requested command
    let commands = commands::Commands::new(client, cli.format);

    let result = if let Some(node_name) = cli.node {
        // Inspect specific node and list its guests
        vlog_info!("Executing: show info for node '{}' with guests", node_name);
        commands.show_node_info(&node_name).await
    } else {
        // Default behavior: list all nodes
        vlog_debug!("Executing: list all nodes");
        commands.list_nodes().await
    };

    // Handle command execution result
    if let Err(e) = result {
        vlog_error!("Command execution failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}