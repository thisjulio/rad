# Task 004: APK Installer (Runtime)

Status: completed

## Todos
- [x] Implement `Prefix::install_apk(&self, apk_path: &Path, info: &ApkInfo)`
- [x] Copy APK to `<prefix>/data/app/<package>/base.apk`
- [x] Extract native libraries from APK to `<prefix>/data/app/<package>/lib/<abi>/`
- [x] Create basic app-specific data directory in `<prefix>/data/data/<package>/`
- [x] Update CLI to call `install_apk` before launching the sandbox


## Context
- `AGENTS.md` - Agent: APK Installer/Inspector
- `PRODUCT.md` - Requirement: `run-android-app <apk>` installs and executes
