mod file_tree;

use anyhow::{Context, bail};
use ignore::WalkBuilder;
use ignore::overrides::{Override, OverrideBuilder};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

use crate::file_tree::FileTree;

const TEXT_CHECK_BUFFER_SIZE: usize = 64 * 1024;

pub struct DigestOptions {
    pub directory: String,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

/// A collected directory digest that can render a tree, file contents, or both.
pub struct Digest {
    root: PathBuf,
    file_tree: FileTree,
    overrides: Override,
    stdout_file: OutputFile,
}

impl Digest {
    /// Validates the root and glob filters before any output is produced.
    pub fn new(options: DigestOptions) -> anyhow::Result<Self> {
        let root = PathBuf::from(&options.directory);
        let metadata = fs::symlink_metadata(&root)
            .with_context(|| format!("cannot access directory `{}`", root.display()))?;

        if metadata.file_type().is_symlink() {
            bail!("directory must not be a symlink: `{}`", root.display());
        }
        if !metadata.is_dir() {
            bail!("path is not a directory: `{}`", root.display());
        }

        let mut override_builder = OverrideBuilder::new(&root);

        for glob in options.include.iter().flatten() {
            override_builder
                .add(glob)
                .with_context(|| format!("invalid include glob `{glob}`"))?;
        }

        for glob in options.exclude.iter().flatten() {
            override_builder
                .add(&format!("!{glob}"))
                .with_context(|| format!("invalid exclude glob `{glob}`"))?;
        }

        let overrides = override_builder
            .build()
            .context("failed to build glob filters")?;

        Ok(Digest {
            file_tree: FileTree::new(&root),
            root,
            overrides,
            stdout_file: OutputFile::stdout(),
        })
    }

    /// Walks the configured directory and collects matching regular files.
    pub fn walk_dirs(&mut self) -> anyhow::Result<()> {
        let entries = WalkBuilder::new(&self.root)
            .overrides(self.overrides.clone())
            .build();

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    eprintln!("warning: skipped inaccessible entry: {error}");
                    continue;
                }
            };

            let Some(file_type) = entry.file_type() else {
                eprintln!(
                    "warning: skipped entry with unknown file type: {}",
                    entry.path().display()
                );
                continue;
            };

            if file_type.is_symlink() {
                eprintln!("warning: skipped symlink: {}", entry.path().display());
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            if self.stdout_file.matches(entry.path()) {
                eprintln!(
                    "warning: skipped active output file: {}",
                    entry.path().display()
                );
                continue;
            }

            let relative_path = entry.path().strip_prefix(&self.root).with_context(|| {
                format!(
                    "walked path `{}` is outside root `{}`",
                    entry.path().display(),
                    self.root.display()
                )
            })?;
            self.file_tree.insert(relative_path, entry.path())?;
        }

        Ok(())
    }

    /// Writes a file header and UTF-8 contents to the output.
    fn write_file<W: Write>(out: &mut W, path: &Path) -> anyhow::Result<()> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                eprintln!(
                    "warning: skipped unreadable file {}: {error}",
                    path.display()
                );
                return Ok(());
            }
        };
        if metadata.file_type().is_symlink() {
            eprintln!("warning: skipped symlink: {}", path.display());
            return Ok(());
        }

        let mut file = match open_without_following(path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!(
                    "warning: skipped unreadable file {}: {error}",
                    path.display()
                );
                return Ok(());
            }
        };

        match is_utf8_text(&mut file) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("warning: skipped non-text file: {}", path.display());
                return Ok(());
            }
            Err(error) => {
                eprintln!(
                    "warning: skipped unreadable file {}: {error}",
                    path.display()
                );
                return Ok(());
            }
        }

        file.rewind()
            .with_context(|| format!("failed to rewind `{}`", path.display()))?;

        let display_path = path.to_string_lossy();
        let table_bar = "─".repeat(UnicodeWidthStr::width(display_path.as_ref()));
        writeln!(
            out,
            "\n┌─{}─┐\n│ {} │\n└─{}─┘",
            table_bar, display_path, table_bar
        )?;

        io::copy(&mut file, out)
            .with_context(|| format!("failed while emitting `{}`", path.display()))?;
        Ok(())
    }

    pub fn write_tree<W: Write>(&self, out: &mut W) -> anyhow::Result<()> {
        self.file_tree.write("", out)?;
        writeln!(out)?;
        Ok(())
    }

    pub fn write_files<W: Write>(&self, out: &mut W) -> anyhow::Result<()> {
        self.file_tree
            .traverse(|path| Self::write_file(out, path))?;
        writeln!(out)?;
        Ok(())
    }
}

#[cfg(unix)]
fn open_without_following(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(windows)]
fn open_without_following(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;

    // FILE_FLAG_OPEN_REPARSE_POINT prevents the final path component from
    // being followed if it changes into a symlink after metadata inspection.
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

#[cfg(not(any(unix, windows)))]
fn open_without_following(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(path)
}

/// Validates UTF-8 and rejects NUL bytes using bounded memory. A second pass
/// then streams valid contents without allocating the entire file.
fn is_utf8_text(file: &mut File) -> io::Result<bool> {
    let mut read_buffer = [0_u8; TEXT_CHECK_BUFFER_SIZE];
    let mut pending = Vec::with_capacity(TEXT_CHECK_BUFFER_SIZE + 3);

    loop {
        let read = file.read(&mut read_buffer)?;
        if read == 0 {
            return Ok(pending.is_empty());
        }
        if read_buffer[..read].contains(&0) {
            return Ok(false);
        }

        pending.extend_from_slice(&read_buffer[..read]);
        match std::str::from_utf8(&pending) {
            Ok(_) => pending.clear(),
            Err(error) if error.error_len().is_some() => return Ok(false),
            Err(error) => pending = pending.split_off(error.valid_up_to()),
        }
    }
}

#[cfg(unix)]
#[derive(Clone, Copy)]
struct OutputFile(Option<(u64, u64)>);

#[cfg(unix)]
impl OutputFile {
    fn stdout() -> Self {
        use std::mem::MaybeUninit;

        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `stat` points to writable, aligned storage and fd 1 is only
        // inspected. A failed fstat simply disables output-file detection.
        let result = unsafe { libc::fstat(libc::STDOUT_FILENO, stat.as_mut_ptr()) };
        if result != 0 {
            return OutputFile(None);
        }

        // SAFETY: fstat returned success and initialized the structure.
        let stat = unsafe { stat.assume_init() };
        if stat.st_mode & libc::S_IFMT != libc::S_IFREG {
            return OutputFile(None);
        }

        OutputFile(Some((stat.st_dev, stat.st_ino)))
    }

    fn matches(&self, path: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;

        let Some((device, inode)) = self.0 else {
            return false;
        };

        fs::metadata(path)
            .map(|metadata| metadata.dev() == device && metadata.ino() == inode)
            .unwrap_or(false)
    }
}

#[cfg(not(unix))]
struct OutputFile(Option<same_file::Handle>);

#[cfg(not(unix))]
impl OutputFile {
    fn stdout() -> Self {
        OutputFile(same_file::Handle::stdout().ok())
    }

    fn matches(&self, path: &Path) -> bool {
        let Some(stdout) = &self.0 else {
            return false;
        };
        same_file::Handle::from_path(path)
            .map(|candidate| candidate == *stdout)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "test failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn utf8_validator_handles_code_point_across_buffer_boundary() {
        let path = std::env::temp_dir().join(format!(
            "dumpr_utf8_boundary_{}_{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let mut contents = vec![b'a'; TEXT_CHECK_BUFFER_SIZE - 1];
        contents.extend_from_slice("€".as_bytes());
        fs::write(&path, contents).unwrap();

        let mut file = File::open(&path).unwrap();
        assert!(is_utf8_text(&mut file).unwrap());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn output_writer_failure_is_propagated() {
        let digest = Digest::new(DigestOptions {
            directory: ".".to_string(),
            include: None,
            exclude: None,
        })
        .unwrap();

        let error = digest.write_tree(&mut FailingWriter).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::BrokenPipe
        );
    }

    #[test]
    fn header_uses_terminal_width_for_unicode_path() {
        let path = std::env::temp_dir().join(format!("dumpr-é-{}.txt", std::process::id()));
        fs::write(&path, "text\n").unwrap();

        let mut output = Vec::new();
        Digest::write_file(&mut output, &path).unwrap();
        let output = String::from_utf8(output).unwrap();
        let top_border = output.lines().nth(1).unwrap();
        let expected_dashes = UnicodeWidthStr::width(path.to_string_lossy().as_ref()) + 2;
        assert_eq!(top_border.matches('─').count(), expected_dashes);

        fs::remove_file(path).unwrap();
    }
}
