//! # models.rs
//!
//! Data structures used across the project

use serde::{Deserialize, Serialize};

/// Output format for command results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format (prettified)
    Json,
    /// CSV format with headers
    Csv,
    /// Table format with borders (default)
    Table,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProxmoxResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthTicket {
    pub ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_token: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub node: String,
    pub status: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub cpu: Option<f64>,
    #[serde(default)]
    pub maxcpu: Option<u32>,
    #[serde(default)]
    pub mem: Option<u64>,
    #[serde(default)]
    pub maxmem: Option<u64>,
    #[serde(default)]
    pub disk: Option<u64>,
    #[serde(default)]
    pub maxdisk: Option<u64>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VM {
    pub vmid: u32,
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub cpus: Option<u32>,
    #[serde(default)]
    pub maxmem: Option<u64>,
    #[serde(default)]
    pub maxdisk: Option<u64>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LXC {
    pub vmid: u32,
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub cpus: Option<u32>,
    #[serde(default)]
    pub maxmem: Option<u64>,
    #[serde(default)]
    pub maxdisk: Option<u64>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum Guest {
    VM(VM),
    LXC(LXC),
}

impl Guest {
    pub fn vmid(&self) -> u32 {
        match self {
            Guest::VM(vm) => vm.vmid,
            Guest::LXC(lxc) => lxc.vmid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Guest::VM(vm) => &vm.name,
            Guest::LXC(lxc) => &lxc.name,
        }
    }

    pub fn status(&self) -> &str {
        match self {
            Guest::VM(vm) => &vm.status,
            Guest::LXC(lxc) => &lxc.status,
        }
    }

    pub fn guest_type(&self) -> &str {
        match self {
            Guest::VM(_) => "VM",
            Guest::LXC(_) => "LXC",
        }
    }
}

// ============================================================================
// Custom JSON output structures (for --format json)
// ============================================================================

/// JSON output structure for listing all nodes
#[derive(Debug, Serialize)]
pub struct NodeListOutput {
    pub root_controller: String,
    pub proxmox_version: String,
    pub nodes: Vec<NodeJsonInfo>,
}

/// Node information in JSON format
#[derive(Debug, Serialize)]
pub struct NodeJsonInfo {
    pub name: String,
    pub cpu: String,
    pub memory_gb: String,
    pub storage_gb: String,
    pub ipv4: String,
    pub status: String,
}

/// JSON output structure for inspecting a single node with guests
#[derive(Debug, Serialize)]
pub struct NodeDetailOutput {
    pub name: String,
    pub cpu: String,
    pub memory_gb: String,
    pub storage_gb: String,
    pub ipv4: String,
    pub status: String,
    pub is_root_controller: String,
    pub guests: Vec<GuestJsonInfo>,
}

/// Guest information in JSON format
#[derive(Debug, Serialize)]
pub struct GuestJsonInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub guest_type: String,
    pub cpu: String,
    pub memory_gb: String,
    pub storage_gb: String,
    pub ipv4: String,
    pub status: String,
}