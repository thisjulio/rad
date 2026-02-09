# Task 004: APK Installer (Runtime)

Status: pending
Priority: medium

## Description
Implement the logic to "install" an APK into a prefix during the orchestration flow.

## Todos
- [ ] Implement `Prefix::install_apk(&self, apk_path: &Path, info: &ApkInfo)`
- [ ] Copy APK to `<prefix>/data/app/<package>/base.apk`
- [ ] Extract native libraries from APK to `<prefix>/data/app/<package>/lib/<abi>/`
- [ ] Create basic app-specific data directory in `<prefix>/data/data/<package>/`
- [ ] Update CLI to call `install_apk` before launching the sandbox

## Context
- `AGENTS.md` - Agent: APK Installer/Inspector
- `PRODUCT.md` - Requirement: `run-android-app <apk>` installs and executes
