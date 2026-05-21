mod digest;

use clap::Parser;
use digest::Digest;

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

    /// Only output files that match the regex pattern
    /// Example: --include ".rs" to only include Rust files
    /// Note: This will be applied after the ignore rules, so it will only filter the files that are not ignored
    #[arg(short, long, default_value = "")]
    include: String,

    /// Exlude files which path matches the regex pattern
    /// Example: --exclude ".png" will exlude all PNG files
    /// Note: This will be applied after the ignore rules, so it will only filter the files that are not ignored
    #[arg(short, long, default_value = "^$")]
    exclude: String,
}

fn main() {
    let args = Args::parse();
    let mut digest = Digest::new(&args);
    if digest.walk_dirs(&args.directory).is_err() {
        eprintln!("Failed to parse the directory. Please check the provided path and try again.");
        std::process::exit(1);
    };

    if args.tree {
        digest.print_tree();
    }
    if args.files {
        digest.print_files();
    }
}
