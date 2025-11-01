/*
src/commands.rs

Output stile `--list-nodes`:
NODE,STATUS,CPU_PERCENT,CPU_CORES,MEM_GB,MEM_MAX_GB,DISK_GB,DISK_MAX_GB,UPTIME_DAYS
tatooine,online,15.3,8,12.45,32.00,45.23,500.00,15.2
hoth,online,8.7,4,6.22,16.00,23.11,250.00,10.5

Output stile `--node tatooine --list`:
tatooine:
├── [100] database-prod (VM) - status:running, cpus:4, ram:8.0GB
├── [101] web-frontend (LXC) - status:running, cpus:2, ram:2.0GB
└── [102] backup-server (VM) - status:stopped, cpus:2, ram:4.0GB
*/

use anyhow::Result;
use crate::client::ProxmoxClient;
use crate::models::Guest;
use crate::{vlog_debug, vlog_success};

pub struct Commands {
    client: ProxmoxClient,
}

impl Commands {
    pub fn new(client: ProxmoxClient) -> Self {
        Self { client }
    }

    pub async fn list_nodes(&self) -> Result<()> {
        vlog_debug!("Fetching cluster nodes...");
        let nodes = self.client.get_nodes().await?;

        // Header - CSV style
        println!("NODE,STATUS,CPU_PERCENT,CPU_CORES,MEM_GB,MEM_MAX_GB,DISK_GB,DISK_MAX_GB,UPTIME_DAYS");

        for node in &nodes {
            let cpu_percent = node.cpu.map(|c| format!("{:.1}", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
            let cpu_cores = node.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());

            let mem_gb = node.mem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
            let maxmem_gb = node.maxmem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());

            let disk_gb = node.disk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
            let maxdisk_gb = node.maxdisk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());

            let uptime_days = node.uptime.map(|u| format!("{:.1}", u as f64 / 86400.0)).unwrap_or_else(|| "N/A".to_string());

            println!("{},{},{},{},{},{},{},{},{}",
                     node.node,
                     node.status,
                     cpu_percent,
                     cpu_cores,
                     mem_gb,
                     maxmem_gb,
                     disk_gb,
                     maxdisk_gb,
                     uptime_days
            );
        }

        vlog_success!("Listed {} node(s)", nodes.len());
        Ok(())
    }

    pub async fn show_node_info(&self, node: &str) -> Result<()> {
        vlog_debug!("Fetching node info for '{}'...", node);
        let node_info = self.client.get_node_status(node).await?;

        // Simple key-value output, parsable
        println!("node: {}", node_info.node);
        println!("status: {}", node_info.status);

        if let Some(cpu) = node_info.cpu {
            println!("cpu_usage: {:.2}%", cpu * 100.0);
        }

        if let Some(maxcpu) = node_info.maxcpu {
            println!("cpu_cores: {}", maxcpu);
        }

        if let Some(mem) = node_info.mem {
            println!("memory: {:.2} GB", mem as f64 / 1024.0 / 1024.0 / 1024.0);
        }

        if let Some(maxmem) = node_info.maxmem {
            println!("memory_max: {:.2} GB", maxmem as f64 / 1024.0 / 1024.0 / 1024.0);
        }

        if let Some(disk) = node_info.disk {
            println!("disk: {:.2} GB", disk as f64 / 1024.0 / 1024.0 / 1024.0);
        }

        if let Some(maxdisk) = node_info.maxdisk {
            println!("disk_max: {:.2} GB", maxdisk as f64 / 1024.0 / 1024.0 / 1024.0);
        }

        if let Some(uptime) = node_info.uptime {
            let days = uptime / 86400;
            let hours = (uptime % 86400) / 3600;
            println!("uptime: {}d {}h", days, hours);
        }

        vlog_success!("Node info displayed");
        Ok(())
    }

    pub async fn list_node_guests(&self, node: &str) -> Result<()> {
        vlog_debug!("Fetching guests for node '{}'...", node);

        // Fetch both VMs and LXC containers
        let vms = self.client.get_vms(node).await?;
        let lxc = self.client.get_lxc(node).await?;

        // Combine into Guest enum and sort by name (quicksort, not bogosort! 😄)
        let mut guests: Vec<Guest> = Vec::new();

        for vm in vms {
            guests.push(Guest::VM(vm));
        }

        for container in lxc {
            guests.push(Guest::LXC(container));
        }

        guests.sort_by(|a, b| a.name().cmp(b.name()));

        // Tree-style output with CSV data
        println!("{}:", node);

        for (i, guest) in guests.iter().enumerate() {
            let is_last = i == guests.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };

            let ram_gb = match guest {
                Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
            }.unwrap_or_else(|| "N/A".to_string());

            let cpus = match guest {
                Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
            }.unwrap_or_else(|| "N/A".to_string());

            println!("{} [{:>3}] {} ({}) - status:{}, cpus:{}, ram:{}GB",
                     prefix,
                     guest.vmid(),
                     guest.name(),
                     guest.guest_type(),
                     guest.status(),
                     cpus,
                     ram_gb
            );
        }

        vlog_success!("Listed {} guest(s) on node '{}'", guests.len(), node);
        Ok(())
    }
}