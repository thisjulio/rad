# Task 003: Runtime Orchestrator & Sandbox

Status: in_progress
Priority: high

## Description
Implement the core logic to create a prefix and setup the sandbox using Linux namespaces.

## Todos
- [ ] Define `Prefix` struct in `crates/core`
- [ ] Implement prefix creation (directory structure: `/data`, `/system`, `/dev`, etc.)
- [ ] Implement `sandbox::setup` using `nix` to enter new namespaces (user, mount, pid)
- [ ] Implement basic bind mounting of a "payload" directory into the prefix
- [ ] Add `doctor` check for required capabilities/privileges

## Context
- `AGENTS.md` - Agent: Runtime Orchestrator & Kernel/Sandbox
- `PRODUCT.md` - Requirement: Reprodutibilidade por prefix
