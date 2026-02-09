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
}

fn main() -> Result<()> {
    // Initialize logging
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
            
            let prefix_path = std::env::current_dir()?.join("prefixes").join(&info.package_name);
            let prefix = Prefix::new(&prefix_path);
            prefix.initialize()?;
            println!("✅ Prefix initialized at: {}", prefix_path.display());

            println!("Installing APK to prefix...");
            prefix.install_apk(std::path::Path::new(&apk_path), &info)?;

            let payload_path = std::env::current_dir()?.join("payload");
            if !payload_path.exists() {
                eprintln!("❌ Payload directory not found. Please ensure 'payload/' exists.");
                std::process::exit(1);
            }

            println!("\n🚀 Launching sandbox...");
            if let Err(e) = prefix.run_in_sandbox(&payload_path, "init") {
                eprintln!("❌ Sandbox failure: {}", e);
                eprintln!("   Note: This often requires User Namespaces. Multi-threading can also block unshare.");
            } else {
                println!("✨ Sandbox session finished.");
            }
        }
    }

    Ok(())
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
