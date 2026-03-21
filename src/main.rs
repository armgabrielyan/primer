mod cli;
mod commands;
mod recipe;
mod state;
mod ui;
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
            let primer_root = workspace::resolve_primer_root(&cli.primer_root)?;
            match command {
                Commands::List => commands::list::run(&primer_root),
                Commands::Init(args) => commands::init::run(&primer_root, args),
                Commands::Doctor(args) => commands::doctor::run(&primer_root, args),
                Commands::Status => commands::status::run(&primer_root, &cli.primer_root),
                Commands::Check => commands::check::run(&primer_root, &cli.primer_root),
                Commands::Build => commands::build::run(&primer_root, &cli.primer_root),
                Commands::NextMilestone => {
                    commands::next_milestone::run(&primer_root, &cli.primer_root)
                }
                Commands::Explain => commands::explain::run(&primer_root, &cli.primer_root),
                Commands::Completions { .. } => unreachable!(),
            }
        }
    }
}
