# Task 064: Waydroid Container Runtime

Status: in_progress
Priority: critical
Type: feature
Parent: 059-aosp-payload-app-process

## Description
Replace the placeholder payload with real Waydroid LineageOS images and build a proper
container runtime that can boot Android init, install APKs, and launch apps.

## Strategy
Use Waydroid's pre-built LineageOS 20 (Android 13) container images:
- System image (~830 MB) - contains /system with app_process64, ART, framework
- Vendor image (~181 MB) - contains /vendor with HALs

These images are container-ready (modified init for namespace environments) with
Wayland SurfaceFlinger support built-in.

## Phases

### Phase 1: Image Setup
- [x] Install simg2img (android-tools package)
- [ ] Create scripts/setup-image.sh
- [ ] Download and extract Waydroid system + vendor images
- [ ] Verify key binaries exist (app_process64, init, etc.)

### Phase 2: Rust Image Management
- [ ] Create crates/core/src/image.rs - paths, validation
- [ ] Add `setup` CLI command

### Phase 3: Enhanced Sandbox
- [ ] PID namespace (CLONE_NEWPID) for /proc and PID 1 init
- [ ] IPC namespace (CLONE_NEWIPC) for binder isolation
- [ ] UTS namespace (CLONE_NEWUTS)
- [ ] Full UID/GID mapping (0-65535 via subuid/subgid)
- [ ] OverlayFS rootfs (system as lower, prefix overlay as upper)
- [ ] Proper /proc, /sys, /dev mounting
- [ ] Binderfs inside container

### Phase 4: Container Lifecycle
- [ ] Container start: boot Android init as PID 1
- [ ] nsenter for executing commands inside running container
- [ ] Boot detection (poll for system ready)
- [ ] APK install via `pm install`
- [ ] App launch via `am start`
- [ ] Container stop

### Phase 5: CLI Integration
- [ ] `run-android-app setup` - download/extract images
- [ ] Rewrite `run` to use container approach
- [ ] Update `shell` to nsenter into running container

### Phase 6: End-to-end Test
- [ ] Test with a real APK from F-Droid

## Image URLs
- System: https://sourceforge.net/projects/waydroid/files/images/system/lineage/waydroid_x86_64/lineage-20.0-20250823-VANILLA-waydroid_x86_64-system.zip/download
- Vendor: https://sourceforge.net/projects/waydroid/files/images/vendor/waydroid_x86_64/lineage-20.0-20250809-MAINLINE-waydroid_x86_64-vendor.zip/download

## System Requirements (Verified)
- Kernel 6.12.68 with CONFIG_ANDROID_BINDER_IPC=y, CONFIG_ANDROID_BINDERFS=y
- User namespaces enabled (unprivileged)
- OverlayFS available
- Wayland running
- subuid/subgid: thisjulio:100000:65536
- simg2img from android-tools

## Acceptance Criteria
- `run-android-app setup` downloads and extracts images
- `run-android-app doctor` passes all checks
- `run-android-app run some.apk` boots Android init and attempts to launch the app
- Container can boot to the point where system services start
