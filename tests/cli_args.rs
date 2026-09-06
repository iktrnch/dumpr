use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
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

fn sample_project(name: &str) -> PathBuf {
    let dir = temp_dir(name);

    write_file(dir.join("README.md"), "project readme\n");
    write_file(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fixture-package\"\n",
    );
    write_file(dir.join("src").join("main.rs"), "fn main() {}\n");
    write_file(dir.join("src").join("lib.rs"), "pub fn sample() {}\n");
    write_file(
        dir.join("src").join("generated.rs"),
        "pub const GENERATED: bool = true;\n",
    );
    write_file(dir.join("src").join("data.json"), "{\"ok\":true}\n");
    write_file(dir.join("notes").join("todo.txt"), "ship tests\n");
    write_file(dir.join("notes").join("draft.md"), "draft notes\n");
    write_file(dir.join("target").join("debug.log"), "debug output\n");

    dir
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

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected output to contain {needle:?}\noutput:\n{haystack}"
    );
}

fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "expected output not to contain {needle:?}\noutput:\n{haystack}"
    );
}

#[test]
fn help_output_lists_supported_arguments() {
    let output = dumpr_cmd()
        .arg("--help")
        .output()
        .expect("help command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    for expected in [
        "Usage: dumpr [OPTIONS] [DIRECTORY]",
        "-t, --tree",
        "-f, --files",
        "-i, --include <GLOB>",
        "-e, --exclude <GLOB>",
    ] {
        assert_contains(&stdout, expected);
    }
}

#[test]
fn tree_flag_outputs_project_tree() {
    let dir = sample_project("tree");

    let output = dumpr_cmd()
        .arg("-t")
        .arg(&dir)
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    assert_contains(&stdout, dir.to_string_lossy().as_ref());
    assert_contains(&stdout, "src");
    assert_contains(&stdout, "main.rs");
    assert_contains(&stdout, "notes");
    assert_contains(&stdout, "todo.txt");
    assert_not_contains(&stdout, "fn main() {}");
    assert_not_contains(&stdout, "ship tests");
}

#[test]
fn files_flag_outputs_file_contents() {
    let dir = sample_project("files");

    let output = dumpr_cmd()
        .arg("-f")
        .arg(&dir)
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    assert_contains(&stdout, "main.rs");
    assert_contains(&stdout, "fn main() {}");
    assert_contains(&stdout, "todo.txt");
    assert_contains(&stdout, "ship tests");
    assert_not_contains(&stdout, "├── main.rs");
    assert_not_contains(&stdout, "└── todo.txt");
}

#[test]
fn tree_and_files_flags_output_tree_before_contents() {
    let dir = sample_project("tree_and_files");

    let output = dumpr_cmd()
        .arg("-t")
        .arg("-f")
        .arg(&dir)
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    let tree_file_name = stdout
        .find("├── README.md")
        .expect("tree output should include README.md");
    let file_body = stdout
        .find("project readme")
        .expect("file output should include README.md contents");

    assert!(
        tree_file_name < file_body,
        "expected tree output before file contents\nstdout:\n{stdout}"
    );
    assert_contains(&stdout, "main.rs");
    assert_contains(&stdout, "fn main() {}");
}

#[test]
fn tree_and_files_respect_multiple_include_and_exclude_globs() {
    let dir = sample_project("glob_constraints");

    let output = dumpr_cmd()
        .arg("-t")
        .arg("-f")
        .arg("-i")
        .arg("*.rs")
        .arg("-i")
        .arg("*.md")
        .arg("-e")
        .arg("generated.rs")
        .arg("-e")
        .arg("notes/*")
        .arg(&dir)
        .output()
        .expect("dumpr command should run");

    assert_success(&output);

    let stdout = stdout(&output);
    assert_contains(&stdout, "README.md");
    assert_contains(&stdout, "project readme");
    assert_contains(&stdout, "main.rs");
    assert_contains(&stdout, "fn main() {}");
    assert_contains(&stdout, "lib.rs");
    assert_contains(&stdout, "pub fn sample() {}");

    assert_not_contains(&stdout, "generated.rs");
    assert_not_contains(&stdout, "GENERATED");
    assert_not_contains(&stdout, "draft.md");
    assert_not_contains(&stdout, "draft notes");
    assert_not_contains(&stdout, "todo.txt");
    assert_not_contains(&stdout, "ship tests");
    assert_not_contains(&stdout, "Cargo.toml");
    assert_not_contains(&stdout, "fixture-package");
    assert_not_contains(&stdout, "data.json");
    assert_not_contains(&stdout, "debug.log");
}

#[test]
fn no_output_flags_defaults_to_tree_and_files() {
    let dir = sample_project("default_output");

    let implicit = dumpr_cmd()
        .arg(&dir)
        .output()
        .expect("dumpr command should run");
    let explicit = dumpr_cmd()
        .arg(&dir)
        .arg("-t")
        .arg("-f")
        .output()
        .expect("dumpr command should run");

    assert_success(&implicit);
    assert_success(&explicit);
    assert_eq!(implicit.stdout, explicit.stdout);
    assert_contains(&stdout(&implicit), "fn main() {}");
}

#[test]
fn directory_remains_positional_only() {
    let dir = sample_project("positional_directory");
    let output = dumpr_cmd()
        .arg("--directory")
        .arg(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_contains(&stderr(&output), "unexpected argument '--directory'");
}

#[test]
fn invalid_root_fails_without_stdout() {
    let missing = temp_dir("missing_root").join("does-not-exist");
    let output = dumpr_cmd()
        .arg(&missing)
        .output()
        .expect("dumpr command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_contains(&stderr(&output), "cannot access directory");
    assert_contains(&stderr(&output), missing.to_string_lossy().as_ref());
}

#[test]
fn malformed_glob_fails_without_stdout() {
    let dir = sample_project("malformed_glob");
    let output = dumpr_cmd()
        .arg(&dir)
        .arg("--include")
        .arg("[")
        .output()
        .expect("dumpr command should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_contains(&stderr(&output), "invalid include glob");
    assert_contains(&stderr(&output), "[");
}

#[test]
fn non_text_file_is_skipped_with_path_warning() {
    let dir = temp_dir("binary_file");
    let binary = dir.join("binary.dat");
    fs::write(&binary, [b'a', 0, b'b']).expect("binary fixture should be written");
    write_file(dir.join("text.txt"), "readable\n");

    let output = dumpr_cmd()
        .arg(&dir)
        .arg("--files")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_contains(&stdout(&output), "readable");
    assert_not_contains(&stdout(&output), "binary.dat");
    assert_contains(&stderr(&output), "warning: skipped non-text file");
    assert_contains(&stderr(&output), binary.to_string_lossy().as_ref());
}

#[cfg(unix)]
#[test]
fn symlink_is_skipped_with_path_warning() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink");
    let target = dir.join("target.txt");
    let link = dir.join("linked.txt");
    write_file(&target, "target contents\n");
    symlink(&target, &link).expect("symlink fixture should be created");

    let output = dumpr_cmd()
        .arg(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_contains(&stdout(&output), "target.txt");
    assert_not_contains(&stdout(&output), "linked.txt");
    assert_contains(&stderr(&output), "warning: skipped symlink");
    assert_contains(&stderr(&output), link.to_string_lossy().as_ref());
}

#[test]
fn redirected_output_inside_root_is_excluded_with_warning() {
    let dir = sample_project("redirected_output");
    let output_path = dir.join("dump.txt");
    let output_file = fs::File::create(&output_path).expect("output fixture should be created");

    let output = dumpr_cmd()
        .arg(&dir)
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::piped())
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    let dumped = fs::read_to_string(&output_path).expect("redirected output should be readable");
    assert_not_contains(&dumped, "dump.txt");
    assert_contains(&stderr(&output), "warning: skipped active output file");
    assert_contains(&stderr(&output), output_path.to_string_lossy().as_ref());
}

#[test]
fn tree_is_sorted_and_does_not_repeat_absolute_root_components() {
    let dir = temp_dir("sorted_absolute_tree");
    write_file(dir.join("zeta").join("z.txt"), "z\n");
    write_file(dir.join("alpha").join("a.txt"), "a\n");

    let output = dumpr_cmd()
        .arg(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    let stdout = stdout(&output);
    assert_eq!(stdout.matches(dir.to_string_lossy().as_ref()).count(), 1);
    assert!(
        stdout.find("alpha").unwrap() < stdout.find("zeta").unwrap(),
        "tree should be deterministic and sorted:\n{stdout}"
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_file_name_does_not_crash_tree_output() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = temp_dir("non_utf8_name");
    let name = OsString::from_vec(b"invalid-\xff.txt".to_vec());
    write_file(dir.join(name), "readable\n");

    let output = dumpr_cmd()
        .arg(&dir)
        .arg("--tree")
        .output()
        .expect("dumpr command should run");

    assert_success(&output);
    assert_contains(&stdout(&output), "invalid-�.txt");
}

#[test]
fn closed_output_pipe_exits_cleanly() {
    let dir = sample_project("closed_pipe");
    let mut child = dumpr_cmd()
        .arg(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("dumpr command should start");

    drop(child.stdout.take());
    let output = child
        .wait_with_output()
        .expect("dumpr command should finish");

    assert_success(&output);
    assert_not_contains(&stderr(&output), "Broken pipe");
}
