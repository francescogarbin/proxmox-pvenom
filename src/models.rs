// src/models.rs
use serde::{Deserialize, Serialize};

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