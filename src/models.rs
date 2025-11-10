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

//! # models.rs
//!
//! Models uses throughout the project.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Csv,
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