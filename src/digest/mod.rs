mod file_tree;
mod matcher;

use std::fs;

use ignore::WalkBuilder;

use crate::Args;

use file_tree::FileTree;
use matcher::Matcher;

/// Wrapper struct for file walker
pub struct Digest {
    /// Stores creates an in-memory representation of the directory
    file_tree: FileTree,
    file_buf: String,
    matcher: Matcher,
}

impl Digest {
    pub fn new(args: &Args) -> Self {
        // Get the directory for the root of the tree
        let initial_directory = match args.directory.split_once("/") {
            Some(path) => path.0,
            None => args.directory.as_str(),
        };

        Digest {
            file_tree: FileTree::new(initial_directory),
            file_buf: String::new(),
            matcher: Matcher::new(&args.include, &args.exclude),
        }
    }

    /// Recursively walks through every directory and file starting from the root path
    /// And applies ignore patterns and building the file tree structure.
    /// The directory tree is traversed using BFS
    pub fn walk_dirs(&mut self, path: &str) -> anyhow::Result<()> {
        let entries = WalkBuilder::new(path).build();
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_str().unwrap();
                    if entry.path().is_file() && self.matcher.is_match(path) {
                        self.file_tree.insert(path)?;
                        match self.read_file(entry.path().to_str().unwrap()) {
                            Ok(()) => {
                                self.file_tree.insert(path)?;
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Could not access file\n{}", e);
                }
            }
        }

        Ok(())
    }

    /// Reads the contents of the file into the buffer
    /// Appends the header in a form of the path
    fn read_file(&mut self, path: &str) -> anyhow::Result<()> {
        // Pretty print the header
        let header = format!(
            "\n\n========================================\n{}\n========================================\n\n",
            path
        );

        // Get the file contents
        let contents = fs::read_to_string(path)?;

        // Concatinate the header and the file contents
        let mut file = header;
        file.push_str(contents.as_str());

        self.file_buf.push_str(file.as_str());

        Ok(())
    }

    pub fn print_tree(&self) {
        println!("{}", self.file_tree.print(""));
    }

    pub fn print_files(&self) {
        println!("{}", self.file_buf);
    }
}
