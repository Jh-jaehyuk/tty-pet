use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::app;

#[derive(Debug, Parser)]
#[command(name = "tty-pet")]
#[command(about = "A tiny terminal companion that reacts to your project state.")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Watch the current project in a small TUI.
    Watch,
    /// Mark the current project's tests as passed.
    Pass,
    /// Mark the current project's tests as failed.
    Fail,
    /// Print the current project's pet state.
    Status {
        /// Include debug fields such as database path.
        #[arg(long)]
        debug: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Watch => app::watch(),
        Command::Pass => app::mark_test_pass(),
        Command::Fail => app::mark_test_fail(),
        Command::Status { debug } => app::status(debug),
    }
}
