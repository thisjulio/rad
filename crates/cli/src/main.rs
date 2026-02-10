use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber::EnvFilter;
use core::doctor;
use apk::ApkInspector;
use core::prefix::Prefix;

#[derive(Parser)]
#[command(name = "run-android-app")]
#[command(about = "A runner for Android apps on Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check host requirements and system state
    Doctor,
    /// Run an Android application (.apk)
    Run {
        /// Path to the APK file
        apk_path: String,
        /// Force execution even if doctor finds issues
        #[arg(long)]
        force: bool,
    },
    /// Open an interactive shell inside the app's sandbox
    Shell {
        /// Package name
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
        Commands::Run { apk_path, force } => {
            if !force {
                let issues = doctor::run_doctor();
                if issues.iter().any(|i| !i.status) {
                    println!("⚠️ System has issues. Run 'doctor' or use --force.");
                }
            }

            println!("Inspecting APK: {}", apk_path);
            let inspector = ApkInspector::new(&apk_path);
            
            let info = inspector.inspect()?;
            println!("✅ APK Metadata:");
            println!("   📦 Package: {}", info.package_name);
            println!("   🏗️  ABIs: {:?}", info.supported_abis.iter().map(|a| a.as_str()).collect::<Vec<_>>());
            
            let prefix = get_prefix(&info.package_name)?;
            prefix.initialize()?;
            println!("✅ Prefix initialized at: {}", prefix.root.display());

            println!("Installing APK to prefix...");
            prefix.install_apk(std::path::Path::new(&apk_path), &info)?;

            let payload_path = std::env::current_dir()?.join("payload");
            if !payload_path.exists() {
                eprintln!("❌ Payload directory not found. Please ensure 'payload/' exists.");
                std::process::exit(1);
            }

            println!("\n🚀 Launching sandbox...");
            // We run /system/bin/init inside the chroot
            if let Err(e) = prefix.run_in_sandbox(&payload_path, "/system/bin/init", &[], true) {
                eprintln!("❌ Sandbox failure: {}", e);
            } else {
                println!("✨ Sandbox session finished.");
            }
        }
        Commands::Shell { package } => {
            let prefix = get_prefix(&package)?;
            let payload_path = std::env::current_dir()?.join("payload");
            
            println!("🐚 Opening shell in {} sandbox...", package);
            // Run busybox explicitly with 'sh' applet
            if let Err(e) = prefix.run_in_sandbox(&payload_path, "/system/bin/busybox", &["sh".to_string()], false) {
                eprintln!("❌ Shell failure: {}", e);
            }
        }
        Commands::Reset { package } => {
            let prefix = get_prefix(&package)?;
            println!("Resetting prefix for {}...", package);
            prefix.reset()?;
            println!("✅ Prefix reset.");
        }
        Commands::Logs { package, follow: _ } => {
            let prefix = get_prefix(&package)?;
            let log_file = prefix.root.join("logs/app.log");
            if !log_file.exists() {
                println!("ℹ️ No logs found for {}.", package);
                return Ok(());
            }
            println!("Showing logs for {}:", package);
            let content = std::fs::read_to_string(log_file)?;
            println!("{}", content);
        }
    }

    Ok(())
}

fn get_prefix(package: &str) -> Result<Prefix> {
    let prefix_path = std::env::current_dir()?.join("prefixes").join(package);
    Ok(Prefix::new(prefix_path))
}

fn run_doctor() {
    println!("Running doctor...");
    let issues = doctor::run_doctor();
    let mut all_ok = true;

    for issue in issues {
        let mark = if issue.status { "✅" } else { "❌" };
        println!("{} {}: {}", mark, issue.name, issue.description);
        if !issue.status {
            all_ok = false;
            if let Some(fix) = issue.fix {
                println!("   💡 Fix: {}", fix);
            }
        }
    }

    if all_ok {
        println!("\n✨ System is ready for run-android-app.");
    } else {
        println!("\n⚠️ Some issues were found. Please resolve them before running apps.");
    }
}
