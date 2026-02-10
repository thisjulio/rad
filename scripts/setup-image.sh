#!/usr/bin/env bash
#
# setup-image.sh - Download and extract Waydroid LineageOS images for RAD
#
# Usage: ./scripts/setup-image.sh
#
# This script will:
#   1. Download system.zip and vendor.zip from Waydroid SourceForge
#   2. Extract .img files from the zips
#   3. Convert sparse images to raw ext4 if needed (simg2img)
#   4. Mount and extract contents to ~/.local/share/rad/images/
#   5. Verify required binaries exist
#
# Requirements: wget/curl, unzip, simg2img (android-tools), sudo (for mount)

set -euo pipefail

# --- Configuration ---
RAD_DATA_DIR="${RAD_DATA_DIR:-$HOME/.local/share/rad}"
IMAGES_DIR="$RAD_DATA_DIR/images"
CACHE_DIR="$RAD_DATA_DIR/cache"
SYSTEM_DIR="$IMAGES_DIR/system"
VENDOR_DIR="$IMAGES_DIR/vendor"

SYSTEM_URL="https://sourceforge.net/projects/waydroid/files/images/system/lineage/waydroid_x86_64/lineage-20.0-20250823-VANILLA-waydroid_x86_64-system.zip/download"
VENDOR_URL="https://sourceforge.net/projects/waydroid/files/images/vendor/waydroid_x86_64/lineage-20.0-20250809-MAINLINE-waydroid_x86_64-vendor.zip/download"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
die()   { error "$*"; exit 1; }

# --- Dependency Checks ---
check_deps() {
    local missing=()

    for cmd in unzip simg2img; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if ! command -v wget &>/dev/null && ! command -v curl &>/dev/null; then
        missing+=("wget or curl")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        die "Missing required tools: ${missing[*]}\n  Install with: sudo pacman -S android-tools wget unzip"
    fi

    info "All dependencies satisfied"
}

# --- Download Helper ---
download() {
    local url="$1"
    local output="$2"

    if [[ -f "$output" ]]; then
        info "Already downloaded: $(basename "$output")"
        return 0
    fi

    info "Downloading $(basename "$output")..."
    info "  URL: $url"
    info "  This may take a while (~1 GB total)..."

    if command -v wget &>/dev/null; then
        wget -q --show-progress -O "$output" "$url"
    else
        curl -L --progress-bar -o "$output" "$url"
    fi

    info "Downloaded: $(basename "$output") ($(du -h "$output" | cut -f1))"
}

# --- Check if image is Android sparse ---
is_sparse_image() {
    local img="$1"
    # Android sparse images start with magic bytes 0x3aff26ed
    local magic
    magic=$(xxd -l 4 -p "$img" 2>/dev/null || echo "")
    [[ "$magic" == "ed26ff3a" ]]
}

# --- Extract image contents to directory ---
extract_image() {
    local img_file="$1"
    local target_dir="$2"
    local label="$3"

    if [[ -d "$target_dir" ]] && [[ -f "$target_dir/.extracted" ]]; then
        info "$label already extracted, skipping"
        return 0
    fi

    info "Extracting $label image..."

    local raw_img="$img_file"

    # Convert sparse to raw if needed
    if is_sparse_image "$img_file"; then
        info "  Converting sparse image to raw ext4..."
        raw_img="${img_file%.img}.raw.img"
        simg2img "$img_file" "$raw_img"
        info "  Converted: $(du -h "$raw_img" | cut -f1)"
    else
        info "  Image is already raw ext4"
    fi

    # Create target directory
    rm -rf "$target_dir"
    mkdir -p "$target_dir"

    # Mount and copy
    local mount_point
    mount_point=$(mktemp -d /tmp/rad-mount-XXXXXX)

    info "  Mounting image (requires sudo)..."
    sudo mount -o loop,ro "$raw_img" "$mount_point"

    info "  Copying contents to $target_dir..."
    sudo cp -a "$mount_point"/. "$target_dir"/

    info "  Unmounting..."
    sudo umount "$mount_point"
    rmdir "$mount_point"

    # Fix ownership (files were copied as root)
    info "  Fixing ownership..."
    sudo chown -R "$(id -u):$(id -g)" "$target_dir"

    # Mark as extracted
    date -Iseconds > "$target_dir/.extracted"

    # Clean up raw image if we created one
    if [[ "$raw_img" != "$img_file" ]]; then
        rm -f "$raw_img"
    fi

    info "$label extracted successfully ($(du -sh "$target_dir" | cut -f1))"
}

# --- Verify extracted images ---
verify_system() {
    local dir="$1"
    local ok=true

    info "Verifying system image contents..."

    local required_files=(
        "system/bin/app_process64"
        "system/bin/sh"
        "system/lib64/libc.so"
        "system/lib64/libandroid_runtime.so"
        "system/framework/framework.jar"
    )

    for f in "${required_files[@]}"; do
        if [[ -e "$dir/$f" ]]; then
            info "  OK: $f"
        else
            warn "  MISSING: $f"
            ok=false
        fi
    done

    # Check for init
    if [[ -e "$dir/init" ]] || [[ -e "$dir/system/bin/init" ]]; then
        info "  OK: init binary found"
    else
        warn "  MISSING: init binary"
        ok=false
    fi

    if $ok; then
        info "System image verification PASSED"
    else
        warn "System image verification had warnings (some files missing)"
        warn "This may be OK - Waydroid images have a different layout"
    fi
}

verify_vendor() {
    local dir="$1"

    info "Verifying vendor image contents..."

    if [[ -d "$dir" ]] && [[ "$(ls -A "$dir" 2>/dev/null)" ]]; then
        info "  Vendor directory is populated ($(ls "$dir" | wc -l) top-level entries)"
        info "Vendor image verification PASSED"
    else
        warn "Vendor directory appears empty"
    fi
}

# --- Main ---
main() {
    info "=== RAD Image Setup ==="
    info "Data directory: $RAD_DATA_DIR"
    echo

    check_deps

    # Create directories
    mkdir -p "$CACHE_DIR" "$IMAGES_DIR"

    # Download
    info ""
    info "--- Downloading images ---"
    download "$SYSTEM_URL" "$CACHE_DIR/system.zip"
    download "$VENDOR_URL" "$CACHE_DIR/vendor.zip"

    # Unzip
    info ""
    info "--- Extracting zip files ---"

    if [[ ! -f "$CACHE_DIR/system.img" ]]; then
        info "Unzipping system.zip..."
        unzip -o -q "$CACHE_DIR/system.zip" -d "$CACHE_DIR/"
        info "Unzipped system.img ($(du -h "$CACHE_DIR/system.img" | cut -f1))"
    else
        info "system.img already extracted from zip"
    fi

    if [[ ! -f "$CACHE_DIR/vendor.img" ]]; then
        info "Unzipping vendor.zip..."
        unzip -o -q "$CACHE_DIR/vendor.zip" -d "$CACHE_DIR/"
        info "Unzipped vendor.img ($(du -h "$CACHE_DIR/vendor.img" | cut -f1))"
    else
        info "vendor.img already extracted from zip"
    fi

    # Extract image contents
    info ""
    info "--- Extracting image contents ---"
    extract_image "$CACHE_DIR/system.img" "$SYSTEM_DIR" "System"
    extract_image "$CACHE_DIR/vendor.img" "$VENDOR_DIR" "Vendor"

    # Verify
    info ""
    info "--- Verifying images ---"
    verify_system "$SYSTEM_DIR"
    verify_vendor "$VENDOR_DIR"

    # Summary
    info ""
    info "=== Setup Complete ==="
    info "System image: $SYSTEM_DIR ($(du -sh "$SYSTEM_DIR" | cut -f1))"
    info "Vendor image: $VENDOR_DIR ($(du -sh "$VENDOR_DIR" | cut -f1))"
    info ""
    info "You can now use 'run-android-app run <apk>' to run Android apps."
}

main "$@"
