# Task 006: Interactive Shell

Status: completed
Priority: medium

## Description
Implement an interactive shell command to explore the sandbox environment of a package.

## Todos
- [x] Implement `Prefix::enter_shell(&self, payload_path: &Path)`
- [x] Enter namespaces without redirecting logs to file (interactive)
- [x] Mount the runtime
- [x] Exec `/bin/sh` (or host shell as fallback) inside the sandbox
- [x] Add `run-android-app shell <package>` command

## Context
- `AGENTS.md` - Agent: Debug/ADB Bridge
- `PRODUCT.md` - Requirement: `shell`
