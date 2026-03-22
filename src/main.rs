mod adapter;
mod bundled;
mod cli;
mod commands;
mod paths;
mod recipe;
mod state;
mod ui;
mod validation;
mod workspace;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use crate::cli::{Cli, Commands};

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
                Commands::Status => commands::status::run(&workspace_hint),
                Commands::Check => commands::check::run(&workspace_hint),
                Commands::Build => commands::build::run(&workspace_hint),
                Commands::NextMilestone => commands::next_milestone::run(&workspace_hint),
                Commands::Explain => commands::explain::run(&workspace_hint),
                Commands::Completions { .. } => unreachable!(),
            }
        }
    }
}
