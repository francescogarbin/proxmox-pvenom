// src/client.rs
use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use serde_json::Value;

use crate::models::{AuthTicket, ProxmoxResponse, Node, VM, LXC};
use crate::{vlog_debug, vlog_info, vlog_error};

pub struct ProxmoxClient {
    base_url: String,
    client: Client,
    ticket: String,           // PVEAuthCookie - usato in TUTTE le richieste
    csrf_token: String,       // CSRFPreventionToken - usato solo in POST/PUT/DELETE
}

impl ProxmoxClient {
    pub async fn new(base_url: &str, username: &str, password: &str, insecure: bool) -> Result<Self> {
        vlog_debug!("Creating Proxmox client for {}", base_url);

        // Build HTTP client
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(insecure)
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

        if !response.status().is_success() {
            vlog_error!("GET {} failed with status: {}", path, response.status());
            anyhow::bail!("Request failed: HTTP {}", response.status());
        }

        let json: Value = response.json().await.context("Failed to parse response")?;
        Ok(json)
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

        let mut node_status: Node = serde_json::from_value(response["data"].clone())
            .context("Failed to parse node status")?;

        // Set the node name (not always in status response)
        node_status.node = node.to_string();

        Ok(node_status)
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