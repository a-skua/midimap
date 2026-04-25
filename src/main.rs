mod action;
mod command;
mod config;
mod keys;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Map MIDI events to keyboard shortcuts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available MIDI input ports
    List,
    /// Run the MIDI mapper with a config file
    Run {
        #[arg(default_value = "midimap.toml")]
        config: String,
        /// Print debug information for each triggered event
        #[arg(short, long)]
        debug: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::List => command::cmd_list(),
        Command::Run { config, debug } => command::cmd_run(&config, debug),
    }
}
