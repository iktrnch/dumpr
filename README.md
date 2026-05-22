<p align="center">
  <img src="https://raw.githubusercontent.com/iktrnch/dumpr/refs/heads/main/assets/dumpr.svg" alt="Dumpr logo" width="300">
</p>

<p align="center">
  <strong>Turn a project directory into a readable text dump.</strong>
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

---

**dumpr** is a small command-line tool for creating readable text snapshots of
directories.

It can print:

- a directory tree
- the contents of matching text files
- both in one output

It is useful when you want to:

- inspect a small project quickly
- paste relevant source files into an AI/code-review tool
- share a compact project snapshot in an issue, article, or message
- generate a readable dump of a repository without manually opening files

`dumpr` respects ignore rules before applying include or exclude filters, so
ignored files are skipped before any custom matching happens.

<p align="center">
  <img src="https://github.com/iktrnch/dumpr/blob/main/assets/dumpr.gif?raw=true" alt="Dumpr demo">
</p>

## Installation

### crates.io

Requires a working Rust toolchain with Cargo installed.

```sh
cargo install dumpr
```

Make sure Cargo's binary directory is on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Most Rust installations already configure this automatically.

### Arch Linux AUR

Install from the AUR with your preferred helper:

```sh
paru -S dumpr
```

or:

```sh
yay -S dumpr
```

### From source

```sh
git clone https://github.com/iktrnch/dumpr.git
cd dumpr
cargo install --path .
```

You can also run it directly while developing:

```sh
cargo run -- --tree --files
```

## Usage

Print the directory tree for the current directory:

```sh
dumpr --tree
```

Print file contents for the current directory:

```sh
dumpr --files
```

Print both the tree and file contents:

```sh
dumpr --tree --files
```

Dump another directory:

```sh
dumpr --directory path/to/project --tree --files
```

Only include Rust files:

```sh
dumpr --tree --files --include '\.rs$'
```

Exclude generated files or directories:

```sh
dumpr --tree --files --exclude 'target/|\.lock$'
```

Save the output to a file:

```sh
dumpr --tree --files > dump.txt
```

## Filtering

`--include` and `--exclude` are regular expressions matched against file paths.

Filtering order:

1. ignore rules are applied
2. `--include` is applied
3. `--exclude` is applied

Examples:

```sh
# Only dump Rust source files
dumpr --files --include '\.rs$'

# Dump everything except Cargo.lock and target/
dumpr --tree --files --exclude 'Cargo\.lock$|target/'

# Dump Markdown and Rust files only
dumpr --files --include '\.md$|\.rs$'
```

## CLI reference

| Option                        | Description                                                                                   |
| ----------------------------- | --------------------------------------------------------------------------------------------- |
| `-d, --directory <DIRECTORY>` | Directory to read. Defaults to the current directory.                                         |
| `-t, --tree`                  | Print a tree view of matching files.                                                          |
| `-f, --files`                 | Print the contents of matching files.                                                         |
| `-i, --include <INCLUDE>`     | Only include files whose path matches this regex. Defaults to matching all non-ignored paths. |
| `-e, --exclude <EXCLUDE>`     | Exclude files whose path matches this regex. Defaults to matching nothing.                    |
| `-h, --help`                  | Print command-line help.                                                                      |
| `-V, --version`               | Print the installed version.                                                                  |

## Notes

`dumpr` is intended for readable project snapshots, not for archiving or backup.

Before sharing output publicly, check that it does not contain secrets, private
keys, tokens, credentials, or other sensitive files.

## Development

Run the project:

```sh
cargo run -- --tree --files
```

Run tests:

```sh
cargo test
```

Format the code:

```sh
cargo fmt
```

## Acknowledgements

This project was inspired by [git-ingest](https://gitingest.com/), an excellent
tool for turning Git repositories into LLM-friendly text input.

See also: [coderamp-labs/gitingest](https://github.com/coderamp-labs/gitingest)

## License

MIT. See [LICENSE.md](LICENSE.md).
