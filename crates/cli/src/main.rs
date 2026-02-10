use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nix::unistd::Pid;
use tracing_subscriber::EnvFilter;

use apk::ApkInspector;
use core::container::Container;
use core::doctor;
use core::image::{ImagePaths, MountPoints};
use core::prefix::Prefix;

#[derive(Parser)]
#[command(name = "run-android-app")]
#[command(about = "A rootless runner for Android apps on Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check host requirements and system state
    Doctor,
    /// Download and set up Waydroid LineageOS images
    Setup,
    /// Run an Android application (.apk)
    Run {
        /// Path to the APK file
        apk_path: String,
        /// Force execution even if doctor finds issues
        #[arg(long)]
        force: bool,
        /// Wait timeout for Android boot (seconds)
        #[arg(long, default_value = "120")]
        boot_timeout: u64,
    },
    /// Open an interactive shell inside the container
    Shell {
        /// Package name (used for prefix directory)
        package: String,
    },
    /// Reset the environment for a specific package
    Reset {
        /// Package name
        package: String,
    },
    /// Show logs for a specific package
    Logs {
        /// Package name
        package: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    /// Stop a running container
    Stop {
        /// Package name
        package: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
            run_doctor();
        }
        Commands::Setup => {
            run_setup()?;
        }
        Commands::Run {
            apk_path,
            force,
            boot_timeout,
        } => {
            run_app(&apk_path, force, boot_timeout)?;
        }
        Commands::Shell { package } => {
            run_shell(&package)?;
        }
        Commands::Reset { package } => {
            let prefix = get_prefix(&package)?;
            println!("Resetting prefix for {}...", package);
            prefix.reset()?;
            println!("Prefix reset.");
        }
        Commands::Logs {
            package,
            follow: _,
        } => {
            let prefix = get_prefix(&package)?;
            let log_file = prefix.root.join("logs/app.log");
            if !log_file.exists() {
                println!("No logs found for {}.", package);
                return Ok(());
            }
            println!("Showing logs for {}:", package);
            let content = std::fs::read_to_string(log_file)?;
            println!("{}", content);
        }
        Commands::Stop { package } => {
            stop_container(&package)?;
        }
    }

    Ok(())
}

fn run_doctor() {
    println!("Running doctor...");
    let issues = doctor::run_doctor();
    let mut all_ok = true;

    for issue in &issues {
        let mark = if issue.status { "OK" } else { "FAIL" };
        println!("[{}] {}: {}", mark, issue.name, issue.description);
        if !issue.status {
            all_ok = false;
            if let Some(fix) = &issue.fix {
                println!("  Fix: {}", fix);
            }
        }
    }

    // Check for fuse2fs
    let fuse2fs_ok = core::container::check_fuse2fs();
    println!(
        "[{}] fuse2fs: {}",
        if fuse2fs_ok { "OK" } else { "FAIL" },
        if fuse2fs_ok {
            "fuse2fs is available for rootless image mounting."
        } else {
            "fuse2fs is NOT installed. Required for rootless image mounting."
        }
    );
    if !fuse2fs_ok {
        println!("  Fix: Install fuse2fs (e.g., `pacman -S fuse2fs`)");
        all_ok = false;
    }

    // Check for images
    match ImagePaths::default_location() {
        Ok(paths) => match paths.validate() {
            Ok(()) => println!("[OK] Waydroid Images: system.img and vendor.img found"),
            Err(e) => {
                println!("[FAIL] Waydroid Images: {}", e);
                println!("  Fix: Run 'run-android-app setup' to download images");
                all_ok = false;
            }
        },
        Err(e) => {
            println!("[FAIL] Waydroid Images: {}", e);
            all_ok = false;
        }
    }

    if all_ok {
        println!("\nSystem is ready for run-android-app (rootless).");
    } else {
        println!("\nSome issues were found. Please resolve them before running apps.");
    }
}

fn run_setup() -> Result<()> {
    println!("Setting up Waydroid LineageOS images...");
    println!("This will run scripts/setup-image.sh to download ~1 GB of images.");

    let script = find_setup_script()?;

    let status = std::process::Command::new("bash")
        .arg(&script)
        .status()
        .with_context(|| format!("Failed to run setup script: {}", script.display()))?;

    if !status.success() {
        anyhow::bail!("Setup script failed with exit code: {:?}", status.code());
    }

    // Validate images after setup
    let paths = ImagePaths::default_location()?;
    paths.validate()?;

    println!("\nSetup complete. You can now run Android apps.");
    Ok(())
}

fn run_app(apk_path: &str, force: bool, boot_timeout: u64) -> Result<()> {
    // Doctor check
    if !force {
        let issues = doctor::run_doctor();
        if issues.iter().any(|i| !i.status) {
            println!("System has issues. Run 'doctor' or use --force to skip checks.");
            return Ok(());
        }
        if !core::container::check_fuse2fs() {
            println!("fuse2fs is not installed. Install it or use --force.");
            println!("  Fix: pacman -S fuse2fs");
            return Ok(());
        }
    }

    // Inspect APK - use parse_manifest() to get main_activity
    println!("Inspecting APK: {}", apk_path);
    let inspector = ApkInspector::new(apk_path);
    let info = inspector.inspect()?;
    let manifest = inspector.parse_manifest()?;

    println!("APK Metadata:");
    println!("  Package: {}", info.package_name);
    println!(
        "  ABIs: {:?}",
        info.supported_abis
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>()
    );
    if let Some(ref activity) = manifest.main_activity {
        println!("  Main activity: {}", activity);
    }

    // Locate images
    let images = ImagePaths::default_location()?;
    images.validate()?;

    // Set up prefix
    let prefix = get_prefix(&info.package_name)?;
    prefix.initialize()?;
    println!("Prefix initialized at: {}", prefix.root.display());

    // Set up container mount points
    let mounts = MountPoints::for_prefix(&prefix.root);
    let pid_file = prefix.root.join(".container_pid");
    let mut container = Container::new(images, mounts).with_pid_file(pid_file);

    // Start container (rootless)
    println!("\nStarting rootless Android container...");
    container.start()?;

    // Wait for boot
    println!("Waiting for Android system to boot...");
    match container.wait_for_boot(boot_timeout) {
        Ok(()) => println!("Android system booted!"),
        Err(e) => {
            println!("[WARN] Boot wait issue: {}. Continuing anyway...", e);
        }
    }

    // Install APK
    println!("Installing APK...");
    match container.install_apk(Path::new(apk_path)) {
        Ok(()) => println!("APK installed."),
        Err(e) => {
            println!("[WARN] APK install issue: {}", e);
        }
    }

    // Try to launch the main activity
    if let Some(activity) = &manifest.main_activity {
        println!("Launching {}/{}...", info.package_name, activity);
        match container.launch_app(&info.package_name, activity) {
            Ok(()) => println!("App launched!"),
            Err(e) => println!("[WARN] App launch issue: {}", e),
        }
    } else {
        println!("No main activity found in manifest.");
        println!(
            "Container is running. Use 'shell {}' to interact.",
            info.package_name
        );
    }

    // Keep running until Ctrl+C
    println!("\nContainer is running. Press Ctrl+C to stop.");
    wait_for_signal();

    // Stop container
    container.stop()?;
    println!("Container stopped.");

    Ok(())
}

fn run_shell(package: &str) -> Result<()> {
    let prefix = get_prefix(package)?;
    let images = ImagePaths::default_location()?;

    // Check if container is already running (look for init PID file)
    let pid_file = prefix.root.join(".container_pid");
    if pid_file.exists() {
        let pid_str = std::fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let pid_nix = Pid::from_raw(pid as i32);
            if nix::sys::signal::kill(pid_nix, None).is_ok() {
                println!("Entering running container (PID {})...", pid);
                // nsenter into our own user namespace doesn't need root
                let status = std::process::Command::new("nsenter")
                    .arg("-t")
                    .arg(pid.to_string())
                    .arg("--user")
                    .arg("--mount")
                    .arg("--uts")
                    .arg("--ipc")
                    .arg("--pid")
                    .arg("--")
                    .arg("/system/bin/sh")
                    .status()?;
                if !status.success() {
                    println!("[WARN] Shell exited with: {:?}", status.code());
                }
                return Ok(());
            }
        }
    }

    // No running container, start one and enter shell
    println!("Starting container for shell access...");
    images.validate()?;
    prefix.initialize()?;

    let mounts = MountPoints::for_prefix(&prefix.root);
    let mut container =
        Container::new(images, mounts).with_pid_file(prefix.root.join(".container_pid"));
    container.start()?;

    // Give it a moment for basic services
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Enter shell
    println!("Entering shell...");
    if let Some(pid) = container.init_pid {
        let _status = std::process::Command::new("nsenter")
            .arg("-t")
            .arg(pid.to_string())
            .arg("--user")
            .arg("--mount")
            .arg("--uts")
            .arg("--ipc")
            .arg("--pid")
            .arg("--")
            .arg("/system/bin/sh")
            .status()?;
    }

    // Stop container when shell exits
    container.stop()?;

    Ok(())
}

fn stop_container(package: &str) -> Result<()> {
    let prefix = get_prefix(package)?;
    let images = ImagePaths::default_location()?;
    let mounts = MountPoints::for_prefix(&prefix.root);

    let pid_file = prefix.root.join(".container_pid");
    let mut container =
        Container::new(images, mounts).with_pid_file(pid_file.clone());

    if pid_file.exists() {
        let pid_str = std::fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            container.init_pid = Some(pid);
        }
    }

    container.stop()?;
    println!("Container stopped.");
    Ok(())
}

fn get_prefix(package: &str) -> Result<Prefix> {
    let prefix_path = std::env::current_dir()?.join("prefixes").join(package);
    Ok(Prefix::new(prefix_path))
}

fn find_setup_script() -> Result<PathBuf> {
    // Try relative to current directory
    let cwd_script = std::env::current_dir()?.join("scripts/setup-image.sh");
    if cwd_script.exists() {
        return Ok(cwd_script);
    }

    // Try relative to binary
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let script = dir.join("../../scripts/setup-image.sh");
        if script.exists() {
            return Ok(script);
        }
    }

    anyhow::bail!(
        "Setup script not found. Expected at ./scripts/setup-image.sh\n\
         Make sure you're running from the project root directory."
    )
}

/// Wait for SIGINT/SIGTERM using a simple signal flag
fn wait_for_signal() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let _running = AtomicBool::new(true);

    // Register signal handler (best-effort)
    unsafe {
        nix::libc::signal(nix::libc::SIGINT, signal_handler as nix::libc::sighandler_t);
        nix::libc::signal(nix::libc::SIGTERM, signal_handler as nix::libc::sighandler_t);
    }

    // Store global flag pointer for signal handler
    SIGNAL_FLAG.store(true, Ordering::SeqCst);

    while SIGNAL_FLAG.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

static SIGNAL_FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn signal_handler(_sig: i32) {
    SIGNAL_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
}
