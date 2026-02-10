# Task 006: Interactive Shell

Status: pending
Priority: medium

## Description
Implement an interactive shell command to explore the sandbox environment of a package.

## Todos
- [ ] Implement `Prefix::enter_shell(&self, payload_path: &Path)`
- [ ] Enter namespaces without redirecting logs to file (interactive)
- [ ] Mount the runtime
- [ ] Exec `/bin/sh` (or host shell as fallback) inside the sandbox
- [ ] Add `run-android-app shell <package>` command

## Context
- `AGENTS.md` - Agent: Debug/ADB Bridge
- `PRODUCT.md` - Requirement: `shell`
