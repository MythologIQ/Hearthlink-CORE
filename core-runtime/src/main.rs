//! Veritas SDR Runtime entry point.
//!
//! Bootstraps the sandboxed inference engine with:
//! - Configuration loading
//! - IPC listener setup
//! - Signal handling for graceful shutdown
//!
//! ## CLI Subcommands
//!
//! - `veritas-sdr` or `veritas-sdr serve` - Run IPC server (default)
//! - `veritas-sdr health` - Full health check (exit 0/1)
//! - `veritas-sdr live` - Liveness probe (exit 0/1)
//! - `veritas-sdr ready` - Readiness probe (exit 0/1)

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use veritas_sdr::cli::{get_socket_path, run_health, run_liveness, run_readiness};
use veritas_sdr::shutdown::ShutdownResult;
use veritas_sdr::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("serve");

    match command {
        "serve" | "" => {
            let config = load_config();
            let runtime = Runtime::new(config);
            match run_ipc_server(runtime).await {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("Server error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        "health" => {
            let socket_path = get_socket_path();
            let code = run_health(&socket_path).await;
            ExitCode::from(code as u8)
        }
        "live" | "liveness" => {
            let socket_path = get_socket_path();
            let code = run_liveness(&socket_path).await;
            ExitCode::from(code as u8)
        }
        "ready" | "readiness" => {
            let socket_path = get_socket_path();
            let code = run_readiness(&socket_path).await;
            ExitCode::from(code as u8)
        }
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "version" | "--version" | "-V" => {
            println!("veritas-sdr {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: veritas-sdr [COMMAND]

Commands:
  serve       Run the IPC server (default)
  health      Full health check (exit 0 if healthy)
  live        Liveness probe (exit 0 if alive)
  ready       Readiness probe (exit 0 if ready)
  version     Show version
  help        Show this help

Environment:
  VERITAS_SOCKET_PATH  IPC socket path (default: platform-specific)
  CORE_AUTH_TOKEN      Authentication token for server mode
"
    );
}

fn load_config() -> RuntimeConfig {
    // In production, load from environment or config file
    // For now, use secure defaults
    RuntimeConfig {
        base_path: PathBuf::from("."),
        auth_token: std::env::var("CORE_AUTH_TOKEN").unwrap_or_default(),
        session_timeout: Duration::from_secs(3600),
        max_context_length: 4096,
        ..Default::default()
    }
}

async fn run_ipc_server(runtime: Runtime) -> Result<(), Box<dyn std::error::Error>> {
    // IPC server loop would go here
    // Using interprocess crate for named pipes/Unix sockets
    //
    // Placeholder: actual IPC binding requires platform-specific setup
    // The handler is ready: runtime.ipc_handler.process(bytes, session)

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("Shutdown signal received, draining...");

                let result = runtime.shutdown.initiate(
                    runtime.config.shutdown_timeout
                ).await;

                match result {
                    ShutdownResult::Complete => {
                        eprintln!("Shutdown complete");
                    }
                    ShutdownResult::Timeout { remaining } => {
                        eprintln!("Shutdown timeout, {} requests remaining", remaining);
                    }
                }
                break;
            }
        }
    }

    Ok(())
}
