//! # main.rs
//!
//! Command line parsing and tool logic.

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

    /// List all nodes in the cluster
    #[arg(long = "list-nodes", conflicts_with = "node")]
    list_nodes: bool,

    /// Specify node name for operations
    #[arg(short = 'n', long = "node")]
    node: Option<String>,

    /// List VMs and LXCs on the specified node
    #[arg(short = 'l', long = "list", requires = "node")]
    list: bool,

    /// Skip SSL certificate verification (use with caution!)
    #[arg(long = "insecure")]
    insecure: bool,

    /// Enable verbose debug logging
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Output format as CSV
    #[arg(long = "as-csv", conflicts_with = "as_table")]
    as_csv: bool,

    /// Output format as table with borders
    #[arg(long = "as-table", conflicts_with = "as_csv")]
    as_table: bool,
}

/// Try to build a working base URL with protocol auto-detection
/// Tries HTTPS first, falls back to HTTP if needed
async fn resolve_base_url(controller: &str, username: &str, password: &str, insecure: bool) -> Result<String> {
    // If user already specified protocol, use it as-is
    if controller.starts_with("http://") || controller.starts_with("https://") {
        vlog_debug!("Protocol already specified in controller address: {}", controller);
        return Ok(controller.to_string());
    }

    // Try HTTPS first, assuming that production clusters have SSL certificates
    let https_url = format!("https://{}", controller);
    vlog_info!("Attempting HTTPS connection to {}...", controller);

    if try_connection(&https_url, username, password, insecure).await.is_ok() {
        vlog_success!("HTTPS connection established to {}", controller);
        return Ok(https_url);
    }

    // Fall back to HTTPm providing support for homelabs with no public SSL certificates
    vlog_warn!("HTTPS connection failed, attempting HTTP fallback...");
    let http_url = format!("http://{}", controller);

    if try_connection(&http_url, username, password, false).await.is_ok() {
        vlog_warn!("HTTP connection successful - consider using HTTPS in production!");
        return Ok(http_url);
    }

    vlog_error!("Could not connect to {} via HTTPS or HTTP", controller);
    bail!("Failed to establish connection to Proxmox cluster");
}

/// Quick connection test to check if the endpoint is reachable
async fn try_connection(base_url: &str, username: &str, password: &str, insecure: bool) -> Result<()> {
    vlog_debug!("Testing connection to {}", base_url);

    // Build a minimal reqwest client just for testing
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(insecure)
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
    let base_url = match resolve_base_url(&cli.controller, &cli.username, &cli.password, cli.insecure).await {
        Ok(url) => url,
        Err(e) => {
            vlog_error!("Connection failed: {}", e);
            std::process::exit(1);
        }
    };

    // Create Proxmox client and authenticate
    vlog_info!("Authenticating to Proxmox API...");
    let client = match ProxmoxClient::new(&base_url, &cli.username, &cli.password, cli.insecure).await {
        Ok(c) => {
            vlog_success!("Authentication successful!");
            c
        },
        Err(e) => {
            vlog_error!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    };

    // Determine output format
    let output_format = if cli.as_csv {
        models::OutputFormat::Csv
    } else if cli.as_table {
        models::OutputFormat::Table
    } else {
        models::OutputFormat::Default
    };

    // Execute the requested command
    let commands = commands::Commands::new(client, output_format);

    let result = if cli.list_nodes {
        vlog_debug!("Executing: list all nodes");
        commands.list_nodes().await
    } else if let Some(node_name) = cli.node {
        if cli.list {
            vlog_info!("Executing: list guests on node '{}'", node_name);
            commands.list_node_guests(&node_name).await
        } else {
            vlog_info!("Executing: show info for node '{}'", node_name);
            commands.show_node_info(&node_name).await
        }
    } else {
        // No command specified, show help
        vlog_error!("No command specified. Use --help for usage information.");
        std::process::exit(1);
    };

    // Handle command execution result
    if let Err(e) = result {
        vlog_error!("Command execution failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}