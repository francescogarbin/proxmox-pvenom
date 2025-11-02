# pvenom
### Proxmox Virtual Environment Node Observability Monitor

![pvenom cli](https://github.com/francescogarbin/proxmox-pvenom/docs/pvenom-as-table.png)

> <b>Inspect ProxMox clusters without requesting API-keys to your sysadmin</b>
>
> *A little terminal friend for nosy sysadmins, ethical hackers, curious, and slightly caffeinated devops engineers.*  
>
> 
> Written in Rust. CLI-first today, TUI-dreams for tomorrow.

**What is this?**  
`pvenom` is a tiny Linux command to inspect Proxmox clusters from the shell — no API tokens required, just user credentials to your Proxmox root/controller. It’s an early, rough-cut version: fast, scrappy, and useful for quick cluster pokes.

---

### Quick highlights
- Single binary (Rust) you can build with `cargo`.
- Connects to a Proxmox root/controller using credentials (username/password).
- Query cluster/node info from the terminal, driven by CLI args.
- Intended to be run on the controller node (or wherever you normally SSH into).
- Licensed under **LGPL-2.1** (see repository).

---

### Install / Build

Clone the repo and build with cargo:

```bash
git clone https://github.com/francescogarbin/proxmox-pvenom.git
cd proxmox-pvenom
cargo build --release
# resulting binary: target/release/pvenom
```

(Or `cargo install --path .` if you prefer it installed to your cargo bin dir.)

Tips:
- Building with `--release` produces the small, optimized binary you want on servers.
- Run it where you already have SSH/console access to the Proxmox controller for easiest credential entry.

---

### Usage (examples)

`pvenom` is intentionally CLI-driven. The syntax and flags are simple and pragmatic (this is a first draft — expect rough edges):

```bash
# show help
./pvenom -h

# inspect cluster summary on controller `pve.example.local`
./pvenom --host pve.controller --user root@pam --password •••• --list-nodes --as-table

# query a node
./pvenom --host pve.controller --user root@pam --password •••• --node pve.node

```

> ⚠️ Never commit plaintext passwords to repos. Use your environment, secrets manager, or run interactively.

---

### Design goals & philosophy

- **Minimal friction.** No API keys or fancy auth flow — just what you need to check things quickly.
- **Practical over pretty.** This is a tool for people who live in terminals.
- **Ethical by default.** Use with permission. Don’t be the villain in someone else’s datacenter. 😏😇

---

### Future roadmap (my TODOs — contributions welcome)
- [ ] Polish CLI syntax and add proper subcommands & exit codes
- [ ] Add an interactive TUI (ncurses / tui-rs) to run directly on controller nodes — keyboard-friendly dashboards 🎛️
- [ ] Support token-based auth and optional vault integration for safer credential handling
- [ ] Add richer output formats: JSON / table / prometheus metrics export
- [ ] Tests, CI, and packaging (deb/rpm)

---

### Contributing
Love it? Hate it? Want to make it better? PRs and issues are warmly invited. This is intentionally an early-stage project — patches, ideas, and fun hacker energy are welcome.

When opening an issue or PR:
- Include the command you ran and the output you saw.
- If you propose new flags or behavior, show an example CLI invocation.

---

### Security & privacy
- `pvenom` talks to your Proxmox controller using the credentials you provide. Keep those credentials safe.
- Consider running the tool locally on a jump host or the controller itself rather than on random workstations.
- Don’t use this on systems you don’t own or administer.

---

### License
This repository is published under **LGPL-2.1**. See the repository license file for details.

---

### A word from the author
The plan: ship something useful, then make it pretty with a TUI — or not. Either way — more nerd toys for sysadmins. :)
