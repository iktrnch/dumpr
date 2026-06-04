use std::io::Write;

#[derive(Debug)]
struct FileEntry {
    name: String,
    path: String,
}

#[derive(Debug)]
pub struct FileTree {
    /// Path to the directory
    root: String,
    /// Vector of paths of child directories
    children: Vec<FileTree>,
    /// Vector of file paths in current directory
    files: Vec<FileEntry>,
}

impl FileTree {
    pub fn new(root_path: &str) -> Self {
        FileTree {
            root: root_path.to_string(),
            children: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Inserts an entry into the tree
    /// Recursively inserts directories before the file
    pub fn insert(&mut self, path: &str) -> anyhow::Result<()> {
        self.insert_inner(path, path)
    }

    fn insert_inner(&mut self, path: &str, full_path: &str) -> anyhow::Result<()> {
        let path = match path.split_once("/") {
            None => {
                // We reached the file - insert
                self.append_file(path, full_path);
                return Ok(());
            }
            Some(p) => p,
        };
        // Check if file is in root dir
        if self.root == path.0 {
            self.insert_inner(path.1, full_path)?;
            return Ok(());
        }

        if let Some(child) = self.children.last_mut()
            && child.root == path.0
        {
            child.insert_inner(path.1, full_path)?;
            return Ok(());
        }

        // Find the dir to insert to
        for child in &mut self.children {
            if child.root == path.0 {
                child.insert_inner(path.1, full_path)?;
                return Ok(());
            }
        }
        // If the directory doesnt exist - create it
        let mut new_child = FileTree::new(path.0);
        new_child.insert_inner(path.1, full_path)?;
        self.append_child(new_child);

        Ok(())
    }

    /// Appends a child directory to the data structure
    fn append_child(&mut self, child: FileTree) {
        self.children.push(child);
    }

    /// Appends files in the directory to the data structure.
    fn append_file(&mut self, name: &str, path: &str) {
        self.files.push(FileEntry {
            name: name.to_string(),
            path: path.to_string(),
        });
    }

    fn write_children<W: Write>(&self, depth: &str, out: &mut W) -> anyhow::Result<()> {
        for (i, child) in self.children.iter().enumerate() {
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
        for (i, file) in self.files.iter().enumerate() {
            let prefix = if self.files.len() == i + 1 && self.children.is_empty() {
                "└── "
            } else {
                "├── "
            };

            writeln!(out, "{}{}{}", depth, prefix, file.name)?;
        }

        Ok(())
    }

    pub fn write<W: Write>(&self, depth: &str, out: &mut W) -> anyhow::Result<()> {
        writeln!(out, "{}", self.root)?;
        self.write_file_names(depth, out)?;
        self.write_children(depth, out)?;

        Ok(())
    }

    pub fn traverse(
        &self,
        mut visit: impl FnMut(&str) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.traverse_inner(&mut visit)
    }

    fn traverse_inner(
        &self,
        visit: &mut impl FnMut(&str) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for child in &self.children {
            child.traverse_inner(visit)?;
        }

        for file in &self.files {
            visit(&file.path)?;
        }

        Ok(())
    }
}
