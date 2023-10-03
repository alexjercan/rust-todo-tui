use clap::Parser;

/// Simple TUI TODO Application for daily tasks.
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Name of the config file to use
    #[arg(short, long)]
    pub config: Option<String>,
}
