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

        let mut nodes = self.client.get_nodes().await?;

        // Fetch IP addresses for all nodes
        for node in &mut nodes {
            node.ip = self.client.get_node_ip(&node.node).await?;
        }

        match self.output_format {
            OutputFormat::Json => {
                // JSON format with custom structure
                use crate::models::{NodeListOutput, NodeJsonInfo};

                // TODO: Fetch actual root_controller and proxmox_version from API
                // For now, use placeholder values
                let root_controller = nodes.first()
                    .map(|n| n.node.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                let proxmox_version = "unknown".to_string();

                let nodes_json: Vec<NodeJsonInfo> = nodes.iter().map(|node| {
                    let cpu_cores = node.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());

                    let memory_gb = match (node.mem, node.maxmem) {
                        (Some(m), Some(mm)) => format!("{}/{}",
                            (m as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (mm as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    let storage_gb = match (node.disk, node.maxdisk) {
                        (Some(d), Some(md)) => format!("{}/{}",
                            (d as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (md as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    NodeJsonInfo {
                        name: node.node.clone(),
                        cpu: cpu_cores,
                        memory_gb,
                        storage_gb,
                        ipv4: node.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                        status: node.status.clone(),
                    }
                }).collect();

                let output = NodeListOutput {
                    root_controller,
                    proxmox_version,
                    nodes: nodes_json,
                };

                let json_pretty = serde_json::to_string_pretty(&output)?;
                println!("{}", json_pretty);
            }
            OutputFormat::Csv => {
                // CSV format with header
                println!("NODE,IP,STATUS,CPU_PERCENT,CPU_CORES,RAM_GB,HDD_GB,UPTIME_DAYS");

                for node in &nodes {
                    let ip = node.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A");
                    let cpu_percent = node.cpu.map(|c| format!("{:.1}", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
                    let cpu_cores = node.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());
                    let uptime_days = node.uptime.map(|u| format!("{:.1}", u as f64 / 86400.0)).unwrap_or_else(|| "N/A".to_string());

                    // Format RAM as "allocated/total" with ceiling, no decimals, no unit (unit in header)
                    let ram_gb = match (node.mem, node.maxmem) {
                        (Some(m), Some(mm)) => format!("{}/{}",
                            (m as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (mm as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    // Format HDD as "used/total" with ceiling, no decimals, no unit (unit in header)
                    let hdd_gb = match (node.disk, node.maxdisk) {
                        (Some(d), Some(md)) => format!("{}/{}",
                            (d as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (md as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    println!("{},{},{},{},{},{},{},{}",
                             node.node,
                             ip,
                             node.status,
                             cpu_percent,
                             cpu_cores,
                             ram_gb,
                             hdd_gb,
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
                    Cell::new("RAM (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("HDD (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Uptime (days)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                // Add rows
                for node in &nodes {
                    let cpu_percent = node.cpu.map(|c| format!("{:.1}", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
                    let cpu_cores = node.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());
                    let uptime_days = node.uptime.map(|u| format!("{:.1}", u as f64 / 86400.0)).unwrap_or_else(|| "N/A".to_string());

                    // Format RAM as "allocated/total" with ceiling, no decimals (unit in header)
                    let ram = match (node.mem, node.maxmem) {
                        (Some(m), Some(mm)) => format!("{}/{}",
                            (m as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (mm as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    // Format HDD as "used/total" with ceiling, no decimals (unit in header)
                    let hdd = match (node.disk, node.maxdisk) {
                        (Some(d), Some(md)) => format!("{}/{}",
                            (d as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                            (md as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                        _ => "N/A".to_string(),
                    };

                    // Format node name with IP on second line
                    let node_name_with_ip = if let Some(ip) = &node.ip {
                        format!("{}\n{}", node.node, ip)
                    } else {
                        node.node.clone()
                    };

                    let status_cell = if node.status == "online" {
                        Cell::new(&node.status).fg(Color::Green)
                    } else {
                        Cell::new(&node.status).fg(Color::Red)
                    };

                    table.add_row(vec![
                        Cell::new(&node_name_with_ip),
                        status_cell,
                        Cell::new(&cpu_percent),
                        Cell::new(&cpu_cores),
                        Cell::new(&ram),
                        Cell::new(&hdd),
                        Cell::new(&uptime_days),
                    ]);
                }

                println!("{}", table);
            }
        }

        vlog_success!("Listed {} node(s)", nodes.len());
        Ok(())
    }

    pub async fn show_node_info(&self, node: &str) -> Result<()> {
        vlog_debug!("Fetching node info and guests for '{}'...", node);

        // Fetch node information
        let mut node_info = self.client.get_node_status(node).await?;
        node_info.ip = self.client.get_node_ip(node).await?;

        // Fetch guests (VMs and LXCs) for this node
        let mut vms = self.client.get_vms(node).await?;
        let mut lxc = self.client.get_lxc(node).await?;

        // Fetch IP addresses for VMs
        for vm in &mut vms {
            vm.ip = self.client.get_guest_ip(node, vm.vmid, "qemu").await?;
        }

        // Fetch IP addresses for LXC containers
        for container in &mut lxc {
            container.ip = self.client.get_guest_ip(node, container.vmid, "lxc").await?;
        }

        // Combine into Guest enum and sort by name
        let mut guests: Vec<Guest> = Vec::new();
        for vm in vms {
            guests.push(Guest::VM(vm));
        }
        for container in lxc {
            guests.push(Guest::LXC(container));
        }
        guests.sort_by(|a, b| a.name().cmp(b.name()));

        match self.output_format {
            OutputFormat::Json => {
                // JSON format with custom structure (node info + guests)
                use crate::models::{NodeDetailOutput, GuestJsonInfo};

                let cpu_cores = node_info.maxcpu.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());

                let memory_gb = match (node_info.mem, node_info.maxmem) {
                    (Some(m), Some(mm)) => format!("{}/{}",
                        (m as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                        (mm as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                    _ => "N/A".to_string(),
                };

                let storage_gb = match (node_info.disk, node_info.maxdisk) {
                    (Some(d), Some(md)) => format!("{}/{}",
                        (d as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                        (md as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64),
                    _ => "N/A".to_string(),
                };

                // TODO: Determine if this node is the root controller
                let is_root_controller = "NO".to_string();

                let guests_json: Vec<GuestJsonInfo> = guests.iter().map(|guest| {
                    let ip = match guest {
                        Guest::VM(vm) => vm.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                        Guest::LXC(lxc) => lxc.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                    };

                    let cpu_cores = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let memory_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let storage_gb = match guest {
                        Guest::VM(vm) => vm.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    GuestJsonInfo {
                        name: guest.name().to_string(),
                        guest_type: guest.guest_type().to_string(),
                        cpu: cpu_cores,
                        memory_gb,
                        storage_gb,
                        ipv4: ip,
                        status: guest.status().to_string(),
                    }
                }).collect();

                let output = NodeDetailOutput {
                    name: node_info.node.clone(),
                    cpu: cpu_cores,
                    memory_gb,
                    storage_gb,
                    ipv4: node_info.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                    status: node_info.status.clone(),
                    is_root_controller,
                    guests: guests_json,
                };

                let json_pretty = serde_json::to_string_pretty(&output)?;
                println!("{}", json_pretty);
            }
            OutputFormat::Csv => {
                // CSV format: print ONLY guests (not node info) to keep CSV consistent
                println!("NAME,STATUS,CPU,RAM_GB,HDD_GB,IPv4");

                for guest in &guests {
                    let ip = match guest {
                        Guest::VM(vm) => vm.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                        Guest::LXC(lxc) => lxc.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                    };

                    let ram_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let hdd_gb = match guest {
                        Guest::VM(vm) => vm.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let cpus = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    println!("{},{},{},{},{},{}",
                             guest.name(),
                             guest.status(),
                             cpus,
                             ram_gb,
                             hdd_gb,
                             ip
                    );
                }
            }
            OutputFormat::Table => {
                // Table format: show node info in one table, then guests in another
                println!("\n=== Node Information ===\n");

                let mut node_table = Table::new();
                node_table.load_preset(UTF8_FULL)
                     .set_content_arrangement(ContentArrangement::Dynamic);

                node_table.set_header(vec![
                    Cell::new("Property").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Value").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                node_table.add_row(vec!["Node", &node_info.node]);

                if let Some(ip) = &node_info.ip {
                    node_table.add_row(vec!["IP", ip]);
                }

                let status_cell = if node_info.status == "online" {
                    Cell::new(&node_info.status).fg(Color::Green)
                } else {
                    Cell::new(&node_info.status).fg(Color::Red)
                };
                node_table.add_row(vec![Cell::new("Status"), status_cell]);

                if let Some(maxcpu) = node_info.maxcpu {
                    node_table.add_row(vec!["CPU Cores", &maxcpu.to_string()]);
                }

                if let (Some(mem), Some(maxmem)) = (node_info.mem, node_info.maxmem) {
                    let ram_gb = format!("{}/{} GB",
                        (mem as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                        (maxmem as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64);
                    node_table.add_row(vec!["RAM", &ram_gb]);
                }

                if let (Some(disk), Some(maxdisk)) = (node_info.disk, node_info.maxdisk) {
                    let hdd_gb = format!("{}/{} GB",
                        (disk as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64,
                        (maxdisk as f64 / 1024.0 / 1024.0 / 1024.0).ceil() as u64);
                    node_table.add_row(vec!["HDD", &hdd_gb]);
                }

                if let Some(uptime) = node_info.uptime {
                    let days = uptime / 86400;
                    let hours = (uptime % 86400) / 3600;
                    node_table.add_row(vec!["Uptime", &format!("{}d {}h", days, hours)]);
                }

                println!("{}", node_table);

                // Now show guests in a separate table
                if !guests.is_empty() {
                    println!("\n=== Guests ({}) ===\n", guests.len());

                    let mut guests_table = Table::new();
                    guests_table.load_preset(UTF8_FULL)
                         .set_content_arrangement(ContentArrangement::Dynamic);

                    guests_table.set_header(vec![
                        Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("IP").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("CPUs").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("RAM (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    ]);

                    for guest in &guests {
                        let ip = match guest {
                            Guest::VM(vm) => vm.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                            Guest::LXC(lxc) => lxc.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                        };

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
                        } else {
                            Cell::new(guest.status()).fg(Color::Red)
                        };

                        let type_cell = if guest.guest_type() == "VM" {
                            Cell::new("VM").fg(Color::Blue)
                        } else {
                            Cell::new("LXC").fg(Color::Magenta)
                        };

                        guests_table.add_row(vec![
                            Cell::new(guest.name()),
                            Cell::new(ip),
                            type_cell,
                            status_cell,
                            Cell::new(&cpus),
                            Cell::new(&ram_gb),
                        ]);
                    }

                    println!("{}", guests_table);
                } else {
                    println!("\nNo guests on this node.\n");
                }
            }
        }

        vlog_success!("Node info and {} guest(s) displayed", guests.len());
        Ok(())
    }

    pub async fn list_node_guests(&self, node: &str) -> Result<()> {
        vlog_debug!("Fetching guests for node '{}'...", node);

        // Fetch both VMs and LXC containers
        let mut vms = self.client.get_vms(node).await?;
        let mut lxc = self.client.get_lxc(node).await?;

        // Fetch IP addresses for VMs
        for vm in &mut vms {
            vm.ip = self.client.get_guest_ip(node, vm.vmid, "qemu").await?;
        }

        // Fetch IP addresses for LXC containers
        for container in &mut lxc {
            container.ip = self.client.get_guest_ip(node, container.vmid, "lxc").await?;
        }

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
                println!("NODE,VMID,NAME,IP,TYPE,STATUS,CPUS,RAM_GB");

                for guest in &guests {
                    let ip = match guest {
                        Guest::VM(vm) => vm.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                        Guest::LXC(lxc) => lxc.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                    };

                    let ram_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let cpus = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    println!("{},{},{},{},{},{},{},{}",
                             node,
                             guest.vmid(),
                             guest.name(),
                             ip,
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
                    Cell::new("IP").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("CPUs").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("RAM (GB)").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                for guest in &guests {
                    let ip = match guest {
                        Guest::VM(vm) => vm.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                        Guest::LXC(lxc) => lxc.ip.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                    };

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
                        Cell::new(ip),
                        type_cell,
                        status_cell,
                        Cell::new(&cpus),
                        Cell::new(&ram_gb),
                    ]);
                }

                println!("Node: {}", node);
                println!("{}", table);
            }
            OutputFormat::Json => {
                // JSON format: list of guests
                use crate::models::GuestJsonInfo;

                let guests_json: Vec<GuestJsonInfo> = guests.iter().map(|guest| {
                    let ip = match guest {
                        Guest::VM(vm) => vm.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                        Guest::LXC(lxc) => lxc.ip.clone().unwrap_or_else(|| "N/A".to_string()),
                    };

                    let cpu_cores = match guest {
                        Guest::VM(vm) => vm.cpus.map(|c| c.to_string()),
                        Guest::LXC(lxc) => lxc.cpus.map(|c| c.to_string()),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let memory_gb = match guest {
                        Guest::VM(vm) => vm.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxmem.map(|m| format!("{:.1}", m as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    let storage_gb = match guest {
                        Guest::VM(vm) => vm.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                        Guest::LXC(lxc) => lxc.maxdisk.map(|d| format!("{:.1}", d as f64 / 1024.0 / 1024.0 / 1024.0)),
                    }.unwrap_or_else(|| "N/A".to_string());

                    GuestJsonInfo {
                        name: guest.name().to_string(),
                        guest_type: guest.guest_type().to_string(),
                        cpu: cpu_cores,
                        memory_gb,
                        storage_gb,
                        ipv4: ip,
                        status: guest.status().to_string(),
                    }
                }).collect();

                let json_pretty = serde_json::to_string_pretty(&guests_json)?;
                println!("{}", json_pretty);
            }
        }

        vlog_success!("Listed {} guest(s) on node '{}'", guests.len(), node);
        Ok(())
    }
}