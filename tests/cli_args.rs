use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn dumpr_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dumpr"))
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("dumpr_{name}_{unique}"));
    fs::create_dir_all(&dir).expect("test temp directory should be created");
    dir
}

fn write_file(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test fixture parent directory should be created");
    }
    fs::write(path, contents).expect("test fixture file should be written");
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be valid utf-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be valid utf-8")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success, got status {:?}\nstderr:\n{}",
        output.status.code(),
        stderr(output)
    );
}

fn assert_stdout_contains(output: &Output, expected: &str) {
    let stdout = stdout(output);
    assert!(
        stdout.contains(expected),
        "expected stdout to contain {expected:?}\nstdout:\n{stdout}"
    );
}

fn assert_stdout_not_contains(output: &Output, unexpected: &str) {
    let stdout = stdout(output);
    assert!(
        !stdout.contains(unexpected),
        "expected stdout not to contain {unexpected:?}\nstdout:\n{stdout}"
    );
}

#[test]
fn help_lists_every_cli_argument() {
    let output = dumpr_cmd()
        .arg("--help")
        .output()
        .expect("help command should run");

    assert_success(&output);
    let stdout = stdout(&output);
    for arg in [
        "-d, --directory",
        "-t, --tree",
        "-f, --files",
        "-i, --include",
        "-e, --exclude",
    ] {
        assert!(
            stdout.contains(arg),
            "expected help output to list {arg:?}\nstdout:\n{stdout}"
        );
    }
}

#[test]
fn tree_flag_uses_current_directory_by_default() {
    let dir = temp_dir("default_directory_tree");
    write_file(dir.join("root.txt"), "root file");

    let output = dumpr_cmd()
        .current_dir(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "root.txt");
}

#[test]
fn short_directory_argument_selects_directory_to_dump() {
    let dir = temp_dir("short_directory");
    let selected = dir.join("selected");
    let sibling = dir.join("sibling");
    write_file(selected.join("selected.txt"), "selected");
    write_file(sibling.join("sibling.txt"), "sibling");

    let output = dumpr_cmd()
        .arg("-d")
        .arg(&selected)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "selected.txt");
    assert_stdout_not_contains(&output, "sibling.txt");
}

#[test]
fn long_directory_argument_selects_directory_to_dump() {
    let dir = temp_dir("long_directory");
    let selected = dir.join("selected");
    let sibling = dir.join("sibling");
    write_file(selected.join("selected.txt"), "selected");
    write_file(sibling.join("sibling.txt"), "sibling");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&selected)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "selected.txt");
    assert_stdout_not_contains(&output, "sibling.txt");
}

#[test]
fn short_tree_argument_prints_file_tree() {
    let dir = temp_dir("short_tree");
    write_file(dir.join("nested").join("leaf.txt"), "leaf");

    let output = dumpr_cmd()
        .arg("-d")
        .arg(&dir)
        .arg("-t")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "nested");
    assert_stdout_contains(&output, "leaf.txt");
}

#[test]
fn long_tree_argument_prints_file_tree() {
    let dir = temp_dir("long_tree");
    write_file(dir.join("nested").join("leaf.txt"), "leaf");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "nested");
    assert_stdout_contains(&output, "leaf.txt");
}

#[test]
fn short_files_argument_prints_file_contents() {
    let dir = temp_dir("short_files");
    write_file(dir.join("content.txt"), "short files body");

    let output = dumpr_cmd()
        .arg("-d")
        .arg(&dir)
        .arg("-f")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "content.txt");
    assert_stdout_contains(&output, "short files body");
}

#[test]
fn long_files_argument_prints_file_contents() {
    let dir = temp_dir("long_files");
    write_file(dir.join("content.txt"), "long files body");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--files")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "content.txt");
    assert_stdout_contains(&output, "long files body");
}

#[test]
fn tree_is_printed_before_file_contents() {
    let dir = temp_dir("tree_before_files");
    write_file(dir.join("content.txt"), "file body");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--tree")
        .arg("--files")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    let tree_pos = stdout
        .find("content.txt")
        .expect("tree should contain file name");
    let body_pos = stdout
        .find("file body")
        .expect("file contents should be printed");

    assert!(
        tree_pos < body_pos,
        "expected tree to be printed before file contents\nstdout:\n{stdout}"
    );
}

#[test]
fn files_are_printed_in_post_order_dfs() {
    let dir = temp_dir("post_order_files");
    write_file(dir.join("root.txt"), "root body");
    write_file(dir.join("nested").join("leaf.txt"), "leaf body");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--files")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    let leaf_pos = stdout
        .find("leaf body")
        .expect("nested file contents should be printed");
    let root_pos = stdout
        .find("root body")
        .expect("root file contents should be printed");

    assert!(
        leaf_pos < root_pos,
        "expected nested file to be printed before root file\nstdout:\n{stdout}"
    );
}

#[test]
fn short_include_argument_keeps_only_matching_paths() {
    let dir = temp_dir("short_include");
    write_file(dir.join("main.rs"), "fn main() {}");
    write_file(dir.join("notes.txt"), "notes");

    let output = dumpr_cmd()
        .arg("-d")
        .arg(&dir)
        .arg("-t")
        .arg("-i")
        .arg(r"\.rs$")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "main.rs");
    assert_stdout_not_contains(&output, "notes.txt");
}

#[test]
fn long_include_argument_keeps_only_matching_paths() {
    let dir = temp_dir("long_include");
    write_file(dir.join("main.rs"), "fn main() {}");
    write_file(dir.join("notes.txt"), "notes");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--tree")
        .arg("--include")
        .arg(r"\.rs$")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "main.rs");
    assert_stdout_not_contains(&output, "notes.txt");
}

#[test]
fn short_exclude_argument_removes_matching_paths() {
    let dir = temp_dir("short_exclude");
    write_file(dir.join("keep.rs"), "keep");
    write_file(dir.join("skip.log"), "skip");

    let output = dumpr_cmd()
        .arg("-d")
        .arg(&dir)
        .arg("-t")
        .arg("-e")
        .arg(r"\.log$")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "keep.rs");
    assert_stdout_not_contains(&output, "skip.log");
}

#[test]
fn long_exclude_argument_removes_matching_paths() {
    let dir = temp_dir("long_exclude");
    write_file(dir.join("keep.rs"), "keep");
    write_file(dir.join("skip.log"), "skip");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--tree")
        .arg("--exclude")
        .arg(r"\.log$")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_stdout_contains(&output, "keep.rs");
    assert_stdout_not_contains(&output, "skip.log");
}

#[test]
fn invalid_include_regex_exits_with_error() {
    let dir = temp_dir("invalid_include");
    write_file(dir.join("file.txt"), "body");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--include")
        .arg("[")
        .output()
        .expect("dumpr command should run");

    assert!(
        !output.status.success(),
        "expected invalid include regex to fail"
    );
    assert!(
        stderr(&output).contains("Failed to read match pattern"),
        "expected include regex error message\nstderr:\n{}",
        stderr(&output)
    );
}

#[test]
fn invalid_exclude_regex_exits_with_error() {
    let dir = temp_dir("invalid_exclude");
    write_file(dir.join("file.txt"), "body");

    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--exclude")
        .arg("[")
        .output()
        .expect("dumpr command should run");

    assert!(
        !output.status.success(),
        "expected invalid exclude regex to fail"
    );
    assert!(
        stderr(&output).contains("Failed to read exclude pattern"),
        "expected exclude regex error message\nstderr:\n{}",
        stderr(&output)
    );
}
