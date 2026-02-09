# Task 005: Basic Management (Stop/Reset/Logs)

Status: pending
Priority: medium

## Description
Implement basic app management commands: reset (clean prefix) and logs (view collected output).

## Todos
- [ ] Implement `Prefix::reset(&self)` to clean the data directory
- [ ] Implement `run-android-app reset <package>` CLI command
- [ ] Redirect sandboxed process output to a log file in `<prefix>/logs/`
- [ ] Implement `run-android-app logs <package>` to tail the log file
- [ ] Implement a basic `stop` command (placeholder or PID based)

## Context
- `AGENTS.md` - Agent: Debug/ADB Bridge
- `PRODUCT.md` - Requirement: `logcat`, `stop`, `reset`
