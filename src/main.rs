use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use vjfind::index::Index;

/// Find files by the {tags} in their paths and names
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Root directory to search
    #[arg(long, env = "VJFIND_ROOT")]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the absolute path of each matching file, one per line
    Query {
        /// e.g. `opening #solo #3D|#2D -#self-shot`
        expr: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let index = match Index::scan(&cli.root) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("vjfind: {}: {e}", cli.root.display());
            return ExitCode::FAILURE;
        }
    };
    match cli.command {
        Command::Query { expr } => {
            let Ok(found) = index.query(&expr) else {
                return ExitCode::FAILURE;
            };
            for entry in found {
                println!("{}", entry.path.display());
            }
        }
    }
    ExitCode::SUCCESS
}
