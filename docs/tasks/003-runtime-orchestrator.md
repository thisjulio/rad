# Task 003: Runtime Orchestrator & Sandbox

Status: completed
Priority: high

## Description
Implement the core logic to create a prefix and setup the sandbox using Linux namespaces.

## Todos
- [x] Define `Prefix` struct in `crates/core`
- [x] Implement prefix creation (directory structure: `/data`, `/system`, `/dev`, etc.)
- [x] Implement `sandbox::setup` using `nix` to enter new namespaces (user, mount, pid)
- [x] Implement basic bind mounting of a "payload" directory into the prefix
- [x] Add `doctor` check for required capabilities/privileges (OverlayFS, etc.)

## Context
- `AGENTS.md` - Agent: Runtime Orchestrator & Kernel/Sandbox
- `PRODUCT.md` - Requirement: Reprodutibilidade por prefix
