# Task 002: APK Inspector

Status: completed
Priority: medium

## Description
Implement basic APK inspection to extract metadata needed for the runtime.

## Todos
- [x] Add `zip` and XML parsing dependencies to `crates/apk`
- [x] Implement APK file reading
- [x] Extract package name from `AndroidManifest.xml` (placeholder)
- [x] Detect supported ABIs by inspecting `lib/` entries
- [x] Implement a `CompatReport` struct
- [x] Add tests for APK inspection

## Context
- `AGENTS.md` - Agent: APK Installer/Inspector
- `PRODUCT.md` - Requirement: `run-android-app <apk>` inspects APK
