mod adapter;
mod bundled;
mod cli;
mod commands;
mod paths;
mod recipe;
mod retry_guidance;
mod state;
mod ui;
mod validation;
mod verification_history;
mod workflow;
mod workspace;
mod workstream;
mod workstream_resume;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use crate::cli::{Cli, Commands, WorkstreamCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "primer", &mut std::io::stdout());
            Ok(())
        }
        command => {
            let source = recipe::source(cli.primer_root.as_deref())?;
            let workspace_hint = std::env::current_dir()?;
            match command {
                Commands::List => commands::list::run(&source),
                Commands::Init(args) => commands::init::run(&source, args),
                Commands::Doctor(args) => commands::doctor::run(&source, args),
                Commands::Workstream(args) => match args.command {
                    WorkstreamCommands::List(args) => {
                        commands::workstream::list(&workspace_hint, args)
                    }
                    WorkstreamCommands::Init(args) => {
                        commands::workstream::init(&workspace_hint, args)
                    }
                    WorkstreamCommands::Switch(args) => {
                        commands::workstream::switch(&workspace_hint, args)
                    }
                },
                Commands::Status(args) => commands::status::run(&workspace_hint, args),
                Commands::Track(args) => commands::track::run(&workspace_hint, args),
                Commands::Verify(args) => commands::verify::run(&workspace_hint, args),
                Commands::Build => commands::build::run(&workspace_hint),
                Commands::NextMilestone(args) => {
                    commands::next_milestone::run(&workspace_hint, args)
                }
                Commands::Explain => commands::explain::run(&workspace_hint),
                Commands::Completions { .. } => unreachable!(),
            }
        }
    }
}
