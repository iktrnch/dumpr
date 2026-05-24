mod file_tree;
mod matcher;

use std::{fs, io::Write};

use ignore::WalkBuilder;

use crate::Args;

use file_tree::FileTree;
use matcher::Matcher;

/// Wrapper struct for file walker
pub struct Digest {
    /// Stores the in-memory representation of matching paths.
    file_tree: FileTree,
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
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Could not access file\n{}", e);
                }
            }
        }

        Ok(())
    }

    /// Writes a file header and contents to the output.
    fn write_file<W: Write>(out: &mut W, path: &str) -> anyhow::Result<()> {
        // Pretty print the header
        writeln!(
            out,
            "\n\n========================================\n{}\n========================================\n",
            path
        )?;

        // Get the file contents
        let contents = fs::read_to_string(path)?;

        out.write_all(contents.as_bytes())?;

        Ok(())
    }

    pub fn write_tree<W: Write>(&self, out: &mut W) -> anyhow::Result<()> {
        self.file_tree.write("", out)?;
        writeln!(out)?;

        Ok(())
    }

    pub fn write_files<W: Write>(&self, out: &mut W) -> anyhow::Result<()> {
        self.file_tree
            .traverse(|path| match Self::write_file(out, path) {
                Ok(()) => Ok(()),
                Err(_) => Ok(()),
            })?;

        writeln!(out)?;

        Ok(())
    }
}
