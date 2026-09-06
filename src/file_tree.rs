use anyhow::{Context, ensure};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct FileTree {
    name: OsString,
    children: BTreeMap<OsString, FileTree>,
    files: BTreeMap<OsString, PathBuf>,
}

impl FileTree {
    pub fn new(root_path: &Path) -> Self {
        FileTree {
            name: root_path.as_os_str().to_os_string(),
            children: BTreeMap::new(),
            files: BTreeMap::new(),
        }
    }

    /// Inserts a path relative to the digest root without assuming a platform separator.
    pub fn insert(&mut self, relative_path: &Path, full_path: &Path) -> anyhow::Result<()> {
        let components = relative_path
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => Some(name.to_os_string()),
                Component::CurDir => None,
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => None,
            })
            .collect::<Vec<_>>();

        ensure!(
            !components.is_empty(),
            "file path has no component relative to the digest root: {}",
            full_path.display()
        );

        let (file_name, directories) = components
            .split_last()
            .context("relative file path unexpectedly had no file name")?;
        let mut node = self;

        for directory in directories {
            node = node
                .children
                .entry(directory.clone())
                .or_insert_with(|| FileTree::from_name(directory));
        }

        node.files
            .insert(file_name.clone(), full_path.to_path_buf());
        Ok(())
    }

    fn from_name(name: &OsStr) -> Self {
        FileTree {
            name: name.to_os_string(),
            children: BTreeMap::new(),
            files: BTreeMap::new(),
        }
    }

    fn write_children<W: Write>(&self, depth: &str, out: &mut W) -> anyhow::Result<()> {
        for (i, child) in self.children.values().enumerate() {
            // Create appropriate indentation
            let mut child_depth = "│   ";
            let mut prefix = "├── ";

            if self.children.len() == i + 1 {
                prefix = "└── ";
                child_depth = "   ";
            }

            let child_depth = format!("{}{} ", depth, child_depth);

            write!(out, "{}{}", depth, prefix)?;
            child.write(&child_depth, out)?;
        }

        Ok(())
    }

    fn write_file_names<W: Write>(&self, depth: &str, out: &mut W) -> anyhow::Result<()> {
        for (i, file_name) in self.files.keys().enumerate() {
            let prefix = if self.files.len() == i + 1 && self.children.is_empty() {
                "└── "
            } else {
                "├── "
            };

            writeln!(out, "{}{}{}", depth, prefix, file_name.to_string_lossy())?;
        }

        Ok(())
    }

    pub fn write<W: Write>(&self, depth: &str, out: &mut W) -> anyhow::Result<()> {
        writeln!(out, "{}", self.name.to_string_lossy())?;
        self.write_file_names(depth, out)?;
        self.write_children(depth, out)?;

        Ok(())
    }

    pub fn traverse(
        &self,
        mut visit: impl FnMut(&Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.traverse_inner(&mut visit)
    }

    fn traverse_inner(
        &self,
        visit: &mut impl FnMut(&Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for child in self.children.values() {
            child.traverse_inner(visit)?;
        }

        for file in self.files.values() {
            visit(file)?;
        }

        Ok(())
    }
}
