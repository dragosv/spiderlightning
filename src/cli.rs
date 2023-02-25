use clap::{Parser, Subcommand};
use slight_core::interface_parser::{InterfaceAtRelease, InterfaceParser};
use slight_core::wasm_parser::{WasmModule, WasmModuleParser};

/// Helper for passing VERSION to opt.
/// If CARGO_VERSION_INFO is set, use it, otherwise use CARGO_PKG_VERSION.
fn version() -> &'static str {
    option_env!("CARGO_VERSION_INFO").unwrap_or(env!("CARGO_PKG_VERSION"))
}

#[derive(Parser, Debug)]
#[clap(author, version = version(), about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run slight providing a config and a module
    Run {
        #[clap(index = 1, value_parser = WasmModuleParser)]
        module: WasmModule,
    },
    /// Add a secret to the application
    Secret {
        #[clap(short, long, value_parser)]
        key: String,
        #[clap(short, long, value_parser)]
        value: String,
    },
    /// Download a SpiderLightning interface
    Add {
        #[clap(index = 1, value_parser = InterfaceParser)]
        interface_at_release: InterfaceAtRelease,
    },

    /// Start a new Slight project
    New {
        #[clap(subcommand)]
        command: Templates,
        #[clap(short, long, value_parser = InterfaceParser)]
        name_at_release: InterfaceAtRelease,
    },
}

#[derive(Debug, Subcommand)]
pub enum Templates {
    /// Start a new C Slight project
    C,
    /// Start a new Rust Slight Project
    Rust,
}

impl std::fmt::Display for Templates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Templates::C => "c",
                Templates::Rust => "rust",
            }
        )
    }
}