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
    /// Print the results as JSON
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the absolute path of each matching file, one per line (with --json, also the tags and name parts)
    Query {
        /// e.g. `opening #solo #3D|#2D -#self-shot`
        expr: String,
    },
    /// Print the tags, most used first, with a breakdown of spelling variants
    Tags,
    /// Print the folders and files whose braces have problems
    Lint,
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
            if cli.json {
                print_json(&found);
            } else {
                for entry in found {
                    println!("{}", entry.path.display());
                }
            }
        }
        Command::Tags => {
            let tags = index.tags();
            if cli.json {
                print_json(&tags);
            } else {
                for tag in tags {
                    print!("{}\t{}", tag.count, tag.tag);
                    if tag.spellings.len() > 1 {
                        let spellings: Vec<String> = tag
                            .spellings
                            .iter()
                            .map(|s| format!("{} ×{}", s.text, s.count))
                            .collect();
                        print!("\t({})", spellings.join(", "));
                    }
                    println!();
                }
            }
        }
        Command::Lint => {
            let items = index.lint();
            if cli.json {
                print_json(&items);
            } else {
                for item in items {
                    let issues: Vec<&str> = item.issues.iter().map(|i| i.describe()).collect();
                    println!("{}\t{}", item.rel, issues.join(", "));
                }
            }
        }
    }
    ExitCode::SUCCESS
}

fn print_json(value: &impl serde::Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("serializable")
    );
}
