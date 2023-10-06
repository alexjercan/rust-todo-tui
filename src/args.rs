use clap::{Parser, Subcommand};

/// Simple TUI TODO Application for daily tasks.
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Name of the config file to use
    #[arg(short, long)]
    pub config: Option<String>,

    /// Subcommands; By default, the app will run in TUI mode
    #[command(subcommand)]
    pub subcmd: Option<SubCommand>,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Show the current status of the TODO list (short)
    Status,
    /// Show the current status of the TODO list (long)
    Details,
}
