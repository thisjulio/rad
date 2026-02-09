use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber::EnvFilter;
use core::doctor;
use apk::ApkInspector;

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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
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
        Commands::Run { apk_path } => {
            println!("Inspecting APK: {}", apk_path);
            let inspector = ApkInspector::new(&apk_path);
            
            match inspector.inspect() {
                Ok(info) => {
                    println!("✅ APK Metadata:");
                    println!("   📦 Package: {}", info.package_name);
                    println!("   🏗️  ABIs: {:?}", info.supported_abis.iter().map(|a| a.as_str()).collect::<Vec<_>>());
                    
                    if info.supported_abis.is_empty() {
                        println!("   ℹ️  No native libs found (likely Java/Kotlin only).");
                    }
                    
                    println!("\nStarting runtime orchestrator (placeholder)...");
                }
                Err(e) => {
                    eprintln!("❌ Failed to inspect APK: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
