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
        help = "Optional path to an external Primer repository root"
    )]
    pub primer_root: Option<PathBuf>,

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

    /// Manage repository-local Primer workstreams
    Workstream(WorkstreamArgs),

    /// Show current Primer workspace progress
    Status,

    /// Switch the active workspace track
    #[command(name = "track")]
    Track(TrackArgs),

    /// Run verification for the current milestone
    #[command(name = "verify", visible_alias = "check")]
    Verify,

    /// Advance to the next milestone after verification
    #[command(name = "next-milestone")]
    NextMilestone,

    /// Show the explanation for the current milestone
    Explain,

    /// Show current milestone build guidance
    Build,

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

#[derive(Debug, Clone, Args)]
pub struct WorkstreamArgs {
    #[command(subcommand)]
    pub command: WorkstreamCommands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum WorkstreamCommands {
    /// Initialize a Primer workstream in the current repository
    Init(WorkstreamInitArgs),
}

#[derive(Debug, Clone, Args)]
pub struct WorkstreamInitArgs {
    /// Workstream identifier
    pub workstream_id: String,

    /// Short workstream goal
    #[arg(long, value_name = "TEXT")]
    pub goal: String,

    /// AI tool adapter to generate
    #[arg(long, value_enum)]
    pub tool: Tool,

    /// Initial interaction track
    #[arg(long, value_enum, default_value_t = Track::Learner)]
    pub track: Track,
}

#[derive(Debug, Clone, Args)]
pub struct TrackArgs {
    /// Target interaction track
    #[arg(value_enum, value_name = "TRACK")]
    pub track: Track,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Tool {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Opencode,
}

impl Tool {
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::Codex => "Codex",
            Tool::Claude => "Claude Code",
            Tool::Cursor => "Cursor",
            Tool::Gemini => "Gemini CLI",
            Tool::Opencode => "OpenCode",
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
