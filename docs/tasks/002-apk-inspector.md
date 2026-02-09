# Task 002: APK Inspector

Status: in_progress
Priority: medium

## Description
Implement basic APK inspection to extract metadata needed for the runtime.

## Todos
- [ ] Add `zip` and XML parsing dependencies to `crates/apk`
- [ ] Implement APK file reading
- [ ] Extract package name from `AndroidManifest.xml`
- [ ] Detect supported ABIs by inspecting `lib/` entries
- [ ] Implement a `CompatReport` struct
- [ ] Add tests for APK inspection

## Context
- `AGENTS.md` - Agent: APK Installer/Inspector
- `PRODUCT.md` - Requirement: `run-android-app <apk>` inspects APK
