mod digest;

use clap::Parser;
use digest::Digest;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder to digest
    #[arg(short, long, default_value = ".")]
    directory: String,

    /// Output file tree structure
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    tree: bool,

    /// Output file contents
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    files: bool,

    /// Only output files that match the glob pattern
    /// Example: --include "*.rs" --include "*.toml" to only include Rust and TOML files
    #[arg(short, long, default_value = "", value_name = "GLOB")]
    include: Option<Vec<String>>,

    /// Exlude files which path matches the regex pattern
    /// Example: --exclude "*.png" --exclude "*.json" will exlude all PNG and JSON files
    #[arg(short, long, value_name = "GLOB")]
    exclude: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    let mut digest = Digest::new(&args);
    if digest.walk_dirs(&args.directory).is_err() {
        eprintln!("Failed to parse the directory. Please check the provided path and try again.");
        std::process::exit(1);
    };

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if args.tree {
        if digest.write_tree(&mut stdout).is_err() {
            eprintln!("Failed to write the file tree.");
            std::process::exit(1);
        }
    }
    if args.files {
        if digest.write_files(&mut stdout).is_err() {
            eprintln!("Failed to write file contents.");
            std::process::exit(1);
        }
    }

    if stdout.flush().is_err() {
        eprintln!("Failed to flush stdout.");
        std::process::exit(1);
    }
}
