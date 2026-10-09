use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use curly_finder::index::{Index, SharedIndex};
use curly_finder::serve;

/// Find files by the {tags} in their paths and names
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Root directory to search
    #[arg(long, env = "CURLY_ROOT")]
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
    /// Answer one JSON request per line on stdin with one JSON line on stdout, until stdin closes
    Serve,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let index = match Index::scan(&cli.root) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("curly: {}: {e}", cli.root.display());
            return ExitCode::FAILURE;
        }
    };
    for rel in &index.skipped {
        eprintln!("curly: could not read: {rel}");
    }
    match cli.command {
        Command::Query { expr } => {
            let found = match index.query(&expr) {
                Ok(found) => found,
                Err(e) => {
                    eprintln!("curly: query: {e}");
                    return ExitCode::FAILURE;
                }
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
        Command::Serve => return run_serve(index),
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

fn run_serve(index: Index) -> ExitCode {
    use std::io::{BufRead, Write};

    let index = SharedIndex::new(index);
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut line = Vec::new();
    loop {
        line.clear();
        match stdin.read_until(b'\n', &mut line) {
            Ok(0) => return ExitCode::SUCCESS,
            Ok(_) => {}
            Err(e) => {
                eprintln!("curly: serve: {e}");
                return ExitCode::FAILURE;
            }
        }
        let response = serve::handle(&index, &String::from_utf8_lossy(&line));
        if writeln!(stdout, "{response}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            return ExitCode::FAILURE;
        }
    }
}

fn print_json(value: &impl serde::Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("serializable")
    );
}
