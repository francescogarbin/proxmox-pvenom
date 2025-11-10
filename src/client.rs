// proxmox-pvenom: inspect and operate your ProxMox clusters from
// the CLI with no API keys.
// Copyright (C) 2025 Francesco Garbin
//
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

//! # client.rs
//!
//! The ProxMox client code.

use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use serde_json::Value;

use crate::models::{AuthTicket, ProxmoxResponse, Node, VM, LXC};
use crate::{vlog_debug, vlog_info, vlog_error};

pub struct ProxmoxClient {
    base_url: String,
    client: Client,
    ticket: String,           // PVEAuthCookie passed in all requests
    csrf_token: String,       // CSRFPreventionToken passed in POST/PUT/DELETE
}

impl ProxmoxClient {
    pub async fn new(base_url: &str, username: &str, password: &str, secure: bool) -> Result<Self> {
        vlog_debug!("Creating Proxmox client for {}", base_url);

        // Build HTTP client
        // When secure=true, verify certs; when secure=false, skip verification
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(!secure)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        // Authenticate and get ticket
        vlog_debug!("Requesting authentication ticket for user: {}", username);
        let ticket_url = format!("{}/api2/json/access/ticket", base_url);

        let response = client
            .post(&ticket_url)
            .form(&[
                ("username", username),
                ("password", password),
            ])
            .send()
            .await
            .context("Failed to send authentication request")?;

        if !response.status().is_success() {
            vlog_error!("Authentication failed with status: {}", response.status());
            anyhow::bail!("Authentication failed: HTTP {}", response.status());
        }

        let auth_response: ProxmoxResponse<AuthTicket> = response
            .json()
            .await
            .context("Failed to parse authentication response")?;

        vlog_debug!("Received authentication ticket for user: {}", auth_response.data.username);

        Ok(Self {
            base_url: base_url.to_string(),
            client,
            ticket: auth_response.data.ticket,
            csrf_token: auth_response.data.csrf_token,
        })
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        vlog_debug!("GET {}", url);

        // Pass the ticket as cookie
        let cookie_header = format!("PVEAuthCookie={}", self.ticket);

        let response = self.client
            .get(&url)
            .header("Cookie", cookie_header)
            .send()
            .await
            .context("Failed to send GET request")?;

        let status = response.status();
        if !status.is_success() {
            vlog_error!("GET {} failed with status: {}", path, status);
            anyhow::bail!("Request failed: HTTP {}", status);
        }

        let json: Value = response.json().await.context("Failed to parse response")?;
        Ok(json)
    }

    /// Get request that doesn't log errors (for optional features like guest agent)
    async fn get_optional(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        vlog_debug!("GET {} (optional)", url);

        let cookie_header = format!("PVEAuthCookie={}", self.ticket);

        let response = self.client
            .get(&url)
            .header("Cookie", cookie_header)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            // Don't log error for expected failures (agent not available)
            vlog_debug!("GET {} returned {}", path, status);
            anyhow::bail!("Request failed: HTTP {}", status);
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    /// Get raw JSON response from an API endpoint (for debugging/dumping)
    pub async fn get_raw_json(&self, path: &str) -> Result<Value> {
        self.get(path).await
    }

    pub async fn get_nodes(&self) -> Result<Vec<Node>> {
        vlog_info!("Fetching cluster nodes...");
        let response = self.get("/api2/json/nodes").await?;

        let nodes: Vec<Node> = serde_json::from_value(response["data"].clone())
            .context("Failed to parse nodes response")?;

        vlog_debug!("Found {} node(s)", nodes.len());
        Ok(nodes)
    }

    pub async fn get_node_status(&self, node: &str) -> Result<Node> {
        vlog_info!("Fetching status for node '{}'...", node);
        let path = format!("/api2/json/nodes/{}/status", node);
        let response = self.get(&path).await?;

        let data = &response["data"];

        // Manually map nested fields from status response to flat Node struct
        let node_status = Node {
            node: node.to_string(),
            status: "online".to_string(), // Status endpoint only returns data for online nodes
            ip: None, // Will be populated later if needed
            cpu: data["cpu"].as_f64(),
            maxcpu: data["cpuinfo"]["cpus"].as_u64().map(|v| v as u32),
            mem: data["memory"]["used"].as_u64(),
            maxmem: data["memory"]["total"].as_u64(),
            disk: data["rootfs"]["used"].as_u64(),
            maxdisk: data["rootfs"]["total"].as_u64(),
            uptime: data["uptime"].as_u64(),
        };

        Ok(node_status)
    }

    pub async fn get_node_ip(&self, node: &str) -> Result<Option<String>> {
        vlog_debug!("Fetching IP for node '{}'...", node);
        let path = format!("/api2/json/nodes/{}/network", node);
        let response = self.get(&path).await?;

        // Parse network interfaces and find the first active interface with an IP
        if let Some(interfaces) = response["data"].as_array() {
            for interface in interfaces {
                // Look for bridge or physical interface with an IP address
                if let Some(address) = interface["address"].as_str() {
                    if !address.is_empty() && address != "127.0.0.1" {
                        vlog_debug!("Found IP {} for node '{}'", address, node);
                        return Ok(Some(address.to_string()));
                    }
                }
            }
        }

        vlog_debug!("No IP found for node '{}'", node);
        Ok(None)
    }

    pub async fn get_guest_ip(&self, node: &str, vmid: u32, guest_type: &str) -> Result<Option<String>> {
        vlog_debug!("Fetching IP for {} {} on node '{}'...", guest_type, vmid, node);

        // Try to get IP from agent network interfaces (optional feature)
        let path = format!("/api2/json/nodes/{}/{}/{}/agent/network-get-interfaces",
                          node, guest_type, vmid);

        match self.get_optional(&path).await {
            Ok(response) => {
                if let Some(result) = response["data"]["result"].as_array() {
                    for interface in result {
                        if let Some(ip_addresses) = interface["ip-addresses"].as_array() {
                            for ip_addr in ip_addresses {
                                if let Some(ip) = ip_addr["ip-address"].as_str() {
                                    // Skip loopback addresses
                                    if !ip.starts_with("127.") && !ip.starts_with("::1") {
                                        vlog_debug!("Found IP {} for {} {}", ip, guest_type, vmid);
                                        return Ok(Some(ip.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                vlog_debug!("No IP found in agent response for {} {}", guest_type, vmid);
                Ok(None)
            }
            Err(_) => {
                // Agent not available or not running (normal for guests without agent)
                vlog_debug!("Agent not available for {} {}", guest_type, vmid);
                Ok(None)
            }
        }
    }

    pub async fn get_vms(&self, node: &str) -> Result<Vec<VM>> {
        vlog_debug!("Fetching VMs for node '{}'...", node);
        let path = format!("/api2/json/nodes/{}/qemu", node);
        let response = self.get(&path).await?;

        let vms: Vec<VM> = serde_json::from_value(response["data"].clone())
            .context("Failed to parse VMs response")?;

        vlog_debug!("Found {} VM(s) on node '{}'", vms.len(), node);
        Ok(vms)
    }

    pub async fn get_lxc(&self, node: &str) -> Result<Vec<LXC>> {
        vlog_debug!("Fetching LXC containers for node '{}'...", node);
        let path = format!("/api2/json/nodes/{}/lxc", node);
        let response = self.get(&path).await?;

        let lxc: Vec<LXC> = serde_json::from_value(response["data"].clone())
            .context("Failed to parse LXC response")?;

        vlog_debug!("Found {} LXC container(s) on node '{}'", lxc.len(), node);
        Ok(lxc)
    }
}