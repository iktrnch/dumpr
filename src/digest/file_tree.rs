#[derive(Debug)]
pub struct FileTree {
    /// Path to the directory
    root: String,
    /// Vector of paths of child directories
    children: Vec<FileTree>,
    /// Vector of file paths in current directory
    files: Vec<String>,
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
        let path = match path.split_once("/") {
            None => {
                // We reached the file - insert
                self.append_file(path);
                return Ok(());
            }
            Some(p) => p,
        };
        // Check if file is in root dir
        if self.root == path.0 {
            self.insert(path.1)?;
            return Ok(());
        }

        // Find the dir to insert to
        for child in &mut self.children {
            if child.root == path.0 {
                child.insert(path.1)?;
                return Ok(());
            }
        }
        // If the directory doesnt exist - create it
        let mut new_child = FileTree::new(path.0);
        new_child.insert(path.1)?;
        self.append_child(new_child);

        Ok(())
    }

    /// Appends a child directory to the data structure
    fn append_child(&mut self, child: FileTree) {
        self.children.push(child);
    }

    /// Appends files in the directory to the datastructure
    fn append_file(&mut self, file: &str) {
        self.files.push(file.to_string());
    }

    fn print_children(&self, depth: &str, out: &mut String) {
        for (i, child) in self.children.iter().enumerate() {
            // Create appropriate indentation
            let mut child_depth = "│   ";
            let mut prefix = "├── ";

            if self.children.len() == i + 1 {
                prefix = "└── ";
                child_depth = "   ";
            }

            let child_depth = format!("{}{} ", depth, child_depth);

            out.push_str(&format!("{}{}{}", depth, prefix, child.print(&child_depth)));
        }
    }

    fn print_files(&self, depth: &str, out: &mut String) {
        for (i, file) in self.files.iter().enumerate() {
            let prefix = if self.files.len() == i + 1 && self.children.len() == 0 {
                "└── "
            } else {
                "├── "
            };

            out.push_str(&format!("{}{}{}\n", depth, prefix, file));
        }
    }

    pub fn print(&self, depth: &str) -> String {
        let mut out: String = String::new();

        out.push_str(&format!("{}\n", self.root));
        self.print_files(depth, &mut out);
        self.print_children(depth, &mut out);

        out
    }
}
