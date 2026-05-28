mod file_tree;

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::{fs, io::Write};

use crate::Args;
use crate::digest::file_tree::FileTree;

/// Wrapper struct for file walker
pub struct Digest {
    /// Stores the in-memory representation of matching paths.
    file_tree: FileTree,
    overrides: OverrideBuilder,
}

impl Digest {
    pub fn new(args: &Args) -> Self {
        let mut override_builder = OverrideBuilder::new(&args.directory);

        if let Some(include_args) = &args.include {
            for glob in include_args {
                match override_builder.add(&glob) {
                    Ok(_) => {}
                    Err(_) => eprintln!("Failed to parse glob: {}\nContinuing anyway.", glob),
                };
            }
        }

        if let Some(exclude_args) = &args.exclude {
            for glob in exclude_args {
                match override_builder.add(&format!("!{}", glob)) {
                    Ok(_) => {}
                    Err(_) => eprintln!("Failed to parse glob: {}\nContinuing anyway.", glob),
                };
            }
        }

        Digest {
            file_tree: FileTree::new(&args.directory),
            overrides: override_builder,
        }
    }

    /// Recursively walks through every directory and file starting from the root path
    /// And applies ignore patterns and building the file tree structure.
    /// The directory tree is traversed using BFS
    pub fn walk_dirs(&mut self, path: &str) -> anyhow::Result<()> {
        let overrides = self.overrides.build()?;
        let entries = WalkBuilder::new(path).overrides(overrides).build();
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_str().unwrap();
                    if entry.path().is_file() {
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
        let table_bar = "─".repeat(path.len());
        writeln!(out, "\n┌─{}─┐\n│ {} │\n└─{}─┘", table_bar, path, table_bar)?;

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
