use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "primer")]
#[command(author, version, about = "Primer CLI for AI-guided project recipes", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "PRIMER_ROOT",
        value_name = "PATH",
        default_value = ".",
        help = "Path to the Primer repository root"
    )]
    pub primer_root: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available recipes
    List,

    /// Initialize a new Primer workspace
    Init(InitArgs),

    /// Check required local tools for a recipe milestone
    Doctor(DoctorArgs),

    /// Show current Primer workspace progress
    Status,

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// Recipe identifier to initialize
    pub recipe_id: String,

    /// AI tool adapter to generate
    #[arg(long, value_enum)]
    pub tool: Tool,

    /// Target workspace directory
    #[arg(long, value_name = "DIR")]
    pub path: PathBuf,

    /// Initial interaction track
    #[arg(long, value_enum, default_value_t = Track::Learner)]
    pub track: Track,

    /// Initial milestone identifier
    #[arg(long, value_name = "ID")]
    pub milestone: Option<String>,

    /// Allow initialization into a non-empty directory
    #[arg(long)]
    pub force: bool,

    /// Show planned actions without writing files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    /// Recipe identifier to inspect. Defaults to the only recipe if there is one.
    pub recipe_id: Option<String>,

    /// Milestone identifier to check against
    #[arg(long, value_name = "ID")]
    pub milestone: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Tool {
    Codex,
    Claude,
}

impl Tool {
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::Codex => "Codex",
            Tool::Claude => "Claude Code",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Track {
    Learner,
    Builder,
}

impl Track {
    pub fn as_str(self) -> &'static str {
        match self {
            Track::Learner => "learner",
            Track::Builder => "builder",
        }
    }
}
