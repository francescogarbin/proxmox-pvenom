//! # commands.rs
//!
//! CLI commands to inspect and manipulate ProxMox cluster nodes via VE.
//!
//! Output stile `--list-nodes`:
//! 
//! NODE,STATUS,CPU_PERCENT,CPU_CORES,MEM_GB,MEM_MAX_GB,DISK_GB,DISK_MAX_GB,UPTIME_DAYS
//! tatooine,online,15.3,8,12.45,32.00,45.23,500.00,15.2
//! hoth,online,8.7,4,6.22,16.00,23.11,250.00,10.5
//!
//! Output stile `--node mynode --list`:
//! 
//! mynode:
//! ├── [100] database-prod (VM) - status:running, cpus:4, ram:8.0GB
//! ├── [101] web-frontend (LXC) - status:running, cpus:2, ram:2.0GB
//! └── [102] backup-server (VM) - status:stopped, cpus:2, ram:4.0GB

use anyhow::Result;
use crate::client::ProxmoxClient;
use crate::models::{Guest, OutputFormat};
use crate::{vlog_debug, vlog_success};
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement, presets::UTF8_FULL};

pub struct Commands {
    client: ProxmoxClient,
    output_format: OutputFormat,
}

impl Commands {
    pub fn new(client: ProxmoxClient, output_format: OutputFormat) -> Self {
        Self { client, output_format }
    }

    pub async fn list_nodes(&self) -> Result<()> {
        vlog_debug!("Fetching cluster nodes...");
        let nodes = self.client.get_nodes().await?;

        match self.output_format {
            OutputFormat::Csv => {
                // CSV format with header
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
            }
            OutputFormat::Table => {
                // Table format with borders
                let mut table = Table::new();
                table.load_preset(UTF8_FULL)
                     .set_content_arrangement(ContentArrangement::Dynamic);

                // Add header
                table.set_header(vec![
                    Cell::new("Node").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("CPU %").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("CPU Cores").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Mem (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Mem Max (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Disk (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Disk Max (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Uptime (days)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                // Add rows
                for node in &nodes {
                    let cpu_percent = node.cpu.map(|c| format!("{:.1}", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
                    let cpu_cores = node.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());
                    let mem_gb = node.mem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                    let maxmem_gb = node.maxmem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                    let disk_gb = node.disk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                    let maxdisk_gb = node.maxdisk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                    let uptime_days = node.uptime.map(|u| format!("{:.1}", u as f64 / 86400.0)).unwrap_or_else(|| "N/A".to_string());

                    let status_cell = if node.status == "online" {
                        Cell::new(&node.status).fg(Color::Green)
                    } else {
                        Cell::new(&node.status).fg(Color::Red)
                    };

                    table.add_row(vec![
                        Cell::new(&node.node),
                        status_cell,
                        Cell::new(&cpu_percent),
                        Cell::new(&cpu_cores),
                        Cell::new(&mem_gb),
                        Cell::new(&maxmem_gb),
                        Cell::new(&disk_gb),
                        Cell::new(&maxdisk_gb),
                        Cell::new(&uptime_days),
                    ]);
                }

                println!("{}", table);
            }
            OutputFormat::Default => {
                // Default CSV-style output (backward compatible)
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
            }
        }

        vlog_success!("Listed {} node(s)", nodes.len());
        Ok(())
    }

    pub async fn show_node_info(&self, node: &str) -> Result<()> {
        vlog_debug!("Fetching node info for '{}'...", node);
        let node_info = self.client.get_node_status(node).await?;

        match self.output_format {
            OutputFormat::Csv => {
                // CSV format with header
                println!("NODE,STATUS,CPU_PERCENT,CPU_CORES,MEM_GB,MEM_MAX_GB,DISK_GB,DISK_MAX_GB,UPTIME_DAYS");

                let cpu_percent = node_info.cpu.map(|c| format!("{:.2}", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
                let cpu_cores = node_info.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());
                let mem_gb = node_info.mem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                let maxmem_gb = node_info.maxmem.map(|m| format!("{:.2}", m as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                let disk_gb = node_info.disk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                let maxdisk_gb = node_info.maxdisk.map(|d| format!("{:.2}", d as f64 / 1024.0 / 1024.0 / 1024.0)).unwrap_or_else(|| "N/A".to_string());
                let uptime_days = node_info.uptime.map(|u| format!("{:.1}", u as f64 / 86400.0)).unwrap_or_else(|| "N/A".to_string());

                println!("{},{},{},{},{},{},{},{},{}",
                         node_info.node,
                         node_info.status,
                         cpu_percent,
                         cpu_cores,
                         mem_gb,
                         maxmem_gb,
                         disk_gb,
                         maxdisk_gb,
                         uptime_days
                );
            }
            OutputFormat::Table => {
                // Table format with borders (vertical key-value layout)
                let mut table = Table::new();
                table.load_preset(UTF8_FULL)
                     .set_content_arrangement(ContentArrangement::Dynamic);

                table.set_header(vec![
                    Cell::new("Property").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Value").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                table.add_row(vec!["Node", &node_info.node]);

                let status_cell = if node_info.status == "online" {
                    Cell::new(&node_info.status).fg(Color::Green)
                } else {
                    Cell::new(&node_info.status).fg(Color::Red)
                };
                table.add_row(vec![Cell::new("Status"), status_cell]);

                if let Some(cpu) = node_info.cpu {
                    table.add_row(vec!["CPU Usage", &format!("{:.2}%", cpu * 100.0)]);
                }

                if let Some(maxcpu) = node_info.maxcpu {
                    table.add_row(vec!["CPU Cores", &maxcpu.to_string()]);
                }

                if let Some(mem) = node_info.mem {
                    table.add_row(vec!["Memory", &format!("{:.2} GB", mem as f64 / 1024.0 / 1024.0 / 1024.0)]);
                }

                if let Some(maxmem) = node_info.maxmem {
                    table.add_row(vec!["Memory Max", &format!("{:.2} GB", maxmem as f64 / 1024.0 / 1024.0 / 1024.0)]);
                }

                if let Some(disk) = node_info.disk {
                    table.add_row(vec!["Disk", &format!("{:.2} GB", disk as f64 / 1024.0 / 1024.0 / 1024.0)]);
                }

                if let Some(maxdisk) = node_info.maxdisk {
                    table.add_row(vec!["Disk Max", &format!("{:.2} GB", maxdisk as f64 / 1024.0 / 1024.0 / 1024.0)]);
                }

                if let Some(uptime) = node_info.uptime {
                    let days = uptime / 86400;
                    let hours = (uptime % 86400) / 3600;
                    table.add_row(vec!["Uptime", &format!("{}d {}h", days, hours)]);
                }

                println!("{}", table);
            }
            OutputFormat::Default => {
                // Default key-value output
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
            }
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

        match self.output_format {
            OutputFormat::Csv => {
                // CSV format with header
                println!("NODE,VMID,NAME,TYPE,STATUS,CPUS,RAM_GB");

                for guest in &guests {
                    let ram_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let cpus = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    println!("{},{},{},{},{},{},{}",
                             node,
                             guest.vmid(),
                             guest.name(),
                             guest.guest_type(),
                             guest.status(),
                             cpus,
                             ram_gb
                    );
                }
            }
            OutputFormat::Table => {
                // Table format with borders
                let mut table = Table::new();
                table.load_preset(UTF8_FULL)
                     .set_content_arrangement(ContentArrangement::Dynamic);

                table.set_header(vec![
                    Cell::new("VMID").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("CPUs").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("RAM (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                for guest in &guests {
                    let ram_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let cpus = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let status_cell = if guest.status() == "running" {
                        Cell::new(guest.status()).fg(Color::Green)
                    } else if guest.status() == "stopped" {
                        Cell::new(guest.status()).fg(Color::Red)
                    } else {
                        Cell::new(guest.status()).fg(Color::Yellow)
                    };

                    let type_cell = match guest.guest_type() {
                        "VM" => Cell::new("VM").fg(Color::Blue),
                        "LXC" => Cell::new("LXC").fg(Color::Magenta),
                        _ => Cell::new(guest.guest_type()),
                    };

                    table.add_row(vec![
                        Cell::new(&guest.vmid().to_string()),
                        Cell::new(guest.name()),
                        type_cell,
                        status_cell,
                        Cell::new(&cpus),
                        Cell::new(&ram_gb),
                    ]);
                }

                println!("Node: {}", node);
                println!("{}", table);
            }
            OutputFormat::Default => {
                // Default tree-style output
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
            }
        }

        vlog_success!("Listed {} guest(s) on node '{}'", guests.len(), node);
        Ok(())
    }
}