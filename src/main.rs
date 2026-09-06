use clap::Parser;
use dumpr::{Digest, DigestOptions};
use std::io::{self, BufWriter, Write};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to digest
    #[arg(value_name = "DIRECTORY", default_value = ".")]
    directory: String,

    /// Output the directory tree (with neither output flag, both are printed)
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    tree: bool,

    /// Output text file contents (with neither output flag, both are printed)
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    files: bool,

    /// Only output files that match the glob pattern
    /// Example: --include "*.rs" --include "*.toml" to only include Rust and TOML files
    #[arg(short, long, value_name = "GLOB")]
    include: Option<Vec<String>>,

    /// Exclude files whose path matches the glob pattern
    /// Example: --exclude "*.png" --exclude "*.json" excludes all PNG and JSON files
    #[arg(short, long, value_name = "GLOB")]
    exclude: Option<Vec<String>>,
}

impl From<Args> for DigestOptions {
    fn from(value: Args) -> Self {
        DigestOptions {
            directory: value.directory,
            include: value.include,
            exclude: value.exclude,
        }
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    let write_tree = args.tree || (!args.tree && !args.files);
    let write_files = args.files || (!args.tree && !args.files);
    let options = DigestOptions::from(args);

    let mut digest = Digest::new(options)?;
    digest.walk_dirs()?;

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    if write_tree {
        digest.write_tree(&mut stdout)?;
    }
    if write_files {
        digest.write_files(&mut stdout)?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        if error.chain().any(|cause| {
            cause
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
        }) {
            return;
        }
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
