<p align="center">
  <img src="https://raw.githubusercontent.com/iktrnch/dumpr/refs/heads/main/assets/dumpr.svg" alt="dumpr" width="280">
</p>

<p align="center">
  <strong>Turn a project directory into a readable text snapshot.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/dumpr">
    <img src="https://img.shields.io/crates/v/dumpr?label=crates.io" alt="crates.io version">
  </a>
  <a href="https://aur.archlinux.org/packages/dumpr">
    <img src="https://img.shields.io/aur/version/dumpr?label=AUR" alt="AUR version">
  </a>
  <a href="LICENSE.md">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license">
  </a>
</p>

<p align="center">
  <a href="#installation">Installation</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#filtering">Filtering</a> ·
  <a href="#cli-reference">CLI reference</a> ·
  <a href="#development">Development</a>
</p>

**dumpr** is a small command-line tool written in Rust that combines a directory
tree and text-file contents into one output. Use it to prepare context for a code
review or AI tool, share a project snapshot, or inspect source files from your
terminal.

- Print a directory tree, file contents, or both.
- Select relevant files with repeatable include and exclude globs.
- Write plain text to stdout for redirection and shell pipelines.

<p align="center">
  <img src="https://raw.githubusercontent.com/iktrnch/dumpr/refs/heads/main/assets/dumpr.gif" alt="Terminal demonstration of dumpr printing a directory tree and file contents" width="900">
</p>

## Installation

### Cargo

With a Rust toolchain installed:

```sh
cargo install dumpr
```

Make sure Cargo's binary directory, usually `~/.cargo/bin`, is on your `PATH`.

### npm packages

Run either wrapper package with Bun:

```sh
bunx @iktrnch/dumpr --version
# Or use the unscoped package:
bunx dumpr-cli --version
```

Both packages provide the `dumpr` command and use the same native binaries.
Supported platforms are Linux (x86_64 and ARM64), macOS (Apple Silicon), and
Windows (x86_64).

### Arch Linux

Install from the AUR with your preferred helper, such as `paru` or `yay`:

```sh
paru -S dumpr
```

For the prebuilt binary, install `dumpr-bin` instead.

### Prebuilt releases

Download native archives from [GitHub Releases](https://github.com/iktrnch/dumpr/releases).
They cover the same platforms as the npm packages and include SHA-256 checksums.

Linux x86_64 releases also include `.deb` packages for Debian/Ubuntu and `.rpm`
packages for Fedora/RHEL. Install the downloaded package with your system's
package manager. There is no dumpr APT or DNF repository.

### From source

```sh
git clone https://github.com/iktrnch/dumpr.git
cd dumpr
cargo install --path .
```

## Usage

Run `dumpr` in a project directory to print its tree followed by its text-file
contents:

```sh
dumpr
```

The directory defaults to `.`. With neither output flag supplied, both sections
are printed, so `dumpr` is equivalent to `dumpr . --tree --files`.

```sh
# Inspect another project
dumpr path/to/project

# Print only the directory tree
dumpr --tree

# Print only text-file contents
dumpr --files

# Save a snapshot
dumpr > dump.txt
```

File contents are labeled with their paths. Warnings and errors go to stderr,
keeping redirected stdout focused on the snapshot. If the output file is inside
the selected directory, dumpr skips it to avoid reading its own output.

> [!TIP]
> Quote glob patterns so your shell passes them to dumpr without expanding them.

## Filtering

Use `--include` and `--exclude` to select paths relative to the chosen directory.
Patterns use gitignore-style syntax: `*.rs` matches names at any depth, while
`src/**` targets paths under `src/`. Repeat either option for multiple patterns.

```sh
# Include Rust and Markdown files
dumpr --include '*.rs' --include '*.md'

# Exclude build artifacts and lockfiles
dumpr --exclude 'target/**' --exclude '*.lock'

# Print Rust source contents, excluding generated files
dumpr --files --include '*.rs' --exclude 'generated.rs'
```

By default, dumpr skips hidden entries and respects `.ignore` files and Git ignore
rules. Git-specific rules, including `.gitignore`, apply when the directory is
inside a Git repository.

When includes are supplied, files must match an include pattern. Exclusions take
precedence over includes. A directory excluded during traversal is not searched.

> [!NOTE]
> Explicit include matches can override ignore rules and include hidden files.
> Files inside directories that remain ignored are still skipped.

## CLI reference

```text
dumpr [OPTIONS] [DIRECTORY]
```

| Argument or option | Description |
| --- | --- |
| `[DIRECTORY]` | Directory to read. Defaults to `.`. |
| `-t, --tree` | Print the directory tree. |
| `-f, --files` | Print matching text-file contents. |
| `-i, --include <GLOB>` | Include matching paths. Repeat for multiple patterns. |
| `-e, --exclude <GLOB>` | Exclude matching paths. Repeat for multiple patterns. |
| `-h, --help` | Show command-line help. |
| `-V, --version` | Show the installed version. |

With neither `--tree` nor `--files`, both sections are printed.

## Output behavior

- Symlinks are skipped and never followed; a symlink cannot be used as the root.
- File contents must be valid UTF-8 without NUL bytes. Non-text and unreadable
  files are omitted from the contents section with a warning identifying the path.
- Invalid roots and malformed globs produce a nonzero exit status, write an error
  to stderr, and leave stdout empty.

dumpr creates readable snapshots; it is not an archive or backup tool.

> [!WARNING]
> Review snapshots before sharing them. Ignore rules and file filters do not
> guarantee that secrets, credentials, or other sensitive data are excluded.

## Development

Clone the repository and use Cargo to run and check the project:

```sh
cargo run -- assets/demo
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

CI also builds native binaries and validates distribution packages. Workflow
helpers live in [`scripts/ci/`](scripts/ci/).

<details>
<summary>Release automation and maintainer setup</summary>

The [CI workflow](.github/workflows/ci.yml) builds, tests, and packages release
artifacts. The [release workflow](.github/workflows/release.yml) requires a
successful `main` CI run for the exact commit and publishes its tested binary
artifacts without rebuilding them. The Rust crate is published separately with
`cargo publish`.

`Cargo.toml` is the version source. A pushed `vX.Y.Z` tag must match that version.
Check a proposed tag locally, substituting the intended version:

```sh
scripts/ci/check-release-version.sh vX.Y.Z
```

Install [actionlint](https://github.com/rhysd/actionlint) to validate the workflows:

```sh
scripts/ci/validate-workflows.sh
```

Manually dispatching **Actions → Release → Run workflow** checks for a successful
CI run for the selected commit without publishing. Builds and package validation
run in CI.

Publishing requires crates.io Trusted Publishing, npm Trusted Publishing for both
wrappers and each native platform package, and the `AUR_SSH_PRIVATE_KEY`
repository secret with write access to both AUR package repositories. The
`dumpr-cli` package requires an initial manual publish before configuring its
Trusted Publisher for `.github/workflows/release.yml`.

</details>

## Acknowledgements

Inspired by [Gitingest](https://github.com/coderamp-labs/gitingest), a tool for
turning Git repositories into LLM-friendly text input.
