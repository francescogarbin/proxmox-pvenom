# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`pvenom` (Proxmox Virtual Environment Node Observability Monitor) is a Rust CLI tool for inspecting Proxmox clusters without requiring API tokens. It authenticates using username/password credentials and queries cluster/node information directly from the Proxmox controller.

Key characteristics:
- Single binary built with Rust 2021 edition
- Designed for CLI-first usage (TUI is a future goal)
- Authenticates via Proxmox ticket-based auth (username/password)
- Supports both HTTPS and HTTP (with automatic fallback for homelabs)
- Intended to run on controller nodes or via SSH access

## Build and Development Commands

Build the project:
```bash
cargo build --release
# Binary output: target/release/pvenom
```

Run during development:
```bash
cargo run -- --controller <host> --username root@pam --password <pass> --list-nodes
```

Run tests:
```bash
cargo test
```

Check code formatting:
```bash
cargo fmt --check
```

Format code:
```bash
cargo fmt
```

Run clippy linting:
```bash
cargo clippy
```

## Architecture

### Module Structure

The codebase follows a clean separation of concerns across five main modules:

1. **`src/main.rs`** - CLI argument parsing (using clap) and orchestration
   - Defines the `Cli` struct with command-line arguments
   - Implements protocol auto-detection (HTTPS with HTTP fallback)
   - Entry point that coordinates authentication and command execution

2. **`src/client.rs`** - Proxmox API client implementation
   - `ProxmoxClient` handles authentication and maintains session state
   - Stores authentication ticket (`PVEAuthCookie`) and CSRF token
   - Provides methods: `get_nodes()`, `get_node_status()`, `get_vms()`, `get_lxc()`
   - All API calls use cookie-based authentication

3. **`src/commands.rs`** - Command execution and output formatting
   - `Commands` struct executes high-level operations (list nodes, show node info, list guests)
   - Handles three output formats: Default (tree/CSV style), CSV, and Table
   - Uses `comfy-table` crate for formatted table output with colors

4. **`src/models.rs`** - Data structures and API response models
   - Defines Proxmox API response types: `Node`, `VM`, `LXC`, `AuthTicket`
   - `Guest` enum wraps both VMs and LXC containers for unified handling
   - `OutputFormat` enum controls output rendering

5. **`src/vlog.rs`** - Custom logging macros (no third-party log dependencies)
   - Level-based logging: Debug, Info, Warn, Error
   - Macros: `vlog_debug!`, `vlog_info!`, `vlog_warn!`, `vlog_error!`, `vlog_success!`
   - Logging level controlled by `--verbose` flag

### Authentication Flow

1. User provides controller address, username, and password
2. `resolve_base_url()` attempts HTTPS first, falls back to HTTP if needed
3. `ProxmoxClient::new()` sends credentials to `/api2/json/access/ticket`
4. Response contains ticket and CSRF token stored in client
5. All subsequent requests include `PVEAuthCookie` header with ticket
6. CSRF token used for POST/PUT/DELETE operations (not yet implemented)

### API Endpoints Used

- `POST /api2/json/access/ticket` - Authentication
- `GET /api2/json/nodes` - List cluster nodes
- `GET /api2/json/nodes/{node}/status` - Get node status/info
- `GET /api2/json/nodes/{node}/qemu` - List VMs on node
- `GET /api2/json/nodes/{node}/lxc` - List LXC containers on node

## CLI Usage Examples

List all nodes in cluster:
```bash
./pvenom --controller pve.example.com --username root@pam --password SECRET --list-nodes
```

List nodes with table formatting:
```bash
./pvenom -c pve.example.com -u root@pam -p SECRET --list-nodes --as-table
```

Show specific node info:
```bash
./pvenom -c pve.example.com -u root@pam -p SECRET --node pve01
```

List VMs and LXC containers on a node:
```bash
./pvenom -c pve.example.com -u root@pam -p SECRET --node pve01 --list
```

Use environment variable for password:
```bash
export PVENOM_PASSWORD="SECRET"
./pvenom -c pve.example.com -u root@pam --list-nodes
```

Skip SSL verification (for self-signed certs):
```bash
./pvenom -c pve.example.com -u root@pam -p SECRET --insecure --list-nodes
```

Enable verbose debug logging:
```bash
./pvenom -c pve.example.com -u root@pam -p SECRET --verbose --list-nodes
```

## Key Dependencies

- **reqwest** (0.12.24) - HTTP client with cookies and JSON support
- **tokio** (1.x) - Async runtime
- **serde/serde_json** (1.0) - JSON serialization
- **clap** (4.x) - CLI argument parsing with derive macros
- **anyhow** (1.0) - Error handling
- **comfy-table** (7.1) - Terminal table formatting

## Code Style Notes

- Error handling uses `anyhow::Result` with context
- Async/await with tokio runtime (`#[tokio::main]`)
- Logging via custom macros (not env_logger or tracing)
- Memory values converted from bytes to GB for display
- Uptime converted from seconds to days/hours
- Protocol auto-detection prioritizes HTTPS for security
