use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use crate::app::{self, ImageAction, ImageSetRequest};
use crate::interactions::Interaction;
use crate::pet::custom_image::AsciiCharset;

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
    /// Poke the current project's pet.
    Poke,
    /// Give the current project's pet a treat.
    Treat,
    /// Call the current project's pet.
    Call,
    /// Ask the current project's pet to nap.
    Nap,
    /// Configure a custom image pet for the current project.
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
    /// Print the current project's pet state.
    Status {
        /// Include debug fields such as database path.
        #[arg(long)]
        debug: bool,
        /// Print machine-readable JSON for agents and scripts.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ImageCommand {
    /// Use an image file as this project's pet.
    Set {
        path: PathBuf,

        /// ASCII output width for the pet image.
        #[arg(long, default_value_t = 24)]
        width: u32,

        /// Terminal cell aspect correction.
        #[arg(long, default_value_t = 0.5)]
        height_scale: f32,

        /// ASCII character set.
        #[arg(long, value_enum, default_value_t = CliAsciiCharset::Dense)]
        charset: CliAsciiCharset,

        /// Reverse brightness mapping.
        #[arg(long)]
        invert: bool,
    },
    /// Restore the built-in pet.
    Clear,
    /// Print the current custom image configuration.
    Status,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliAsciiCharset {
    Dense,
    Simple,
}

impl From<CliAsciiCharset> for AsciiCharset {
    fn from(value: CliAsciiCharset) -> Self {
        match value {
            CliAsciiCharset::Dense => Self::Dense,
            CliAsciiCharset::Simple => Self::Simple,
        }
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Watch => app::watch(),
        Command::Pass => app::mark_test_pass(),
        Command::Fail => app::mark_test_fail(),
        Command::Poke => app::interact(Interaction::Poke),
        Command::Treat => app::interact(Interaction::Treat),
        Command::Call => app::interact(Interaction::Call),
        Command::Nap => app::interact(Interaction::Nap),
        Command::Image { command } => app::image(command.into()),
        Command::Status { debug, json } => app::status(app::StatusOptions { debug, json }),
    }
}

impl From<ImageCommand> for ImageAction {
    fn from(command: ImageCommand) -> Self {
        match command {
            ImageCommand::Set {
                path,
                width,
                height_scale,
                charset,
                invert,
            } => Self::Set(ImageSetRequest {
                path,
                width,
                height_scale,
                charset: charset.into(),
                invert,
            }),
            ImageCommand::Clear => Self::Clear,
            ImageCommand::Status => Self::Status,
        }
    }
}
