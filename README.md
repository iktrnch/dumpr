# dumpr

dumpr is a small command line tool that turns a directory into a readable text
dump. It can print the directory tree, the contents of matching text files, or
both.

The output is useful when you want to review a project quickly, paste relevant
files into another tool, or create a compact snapshot of a codebase. dumpr uses
the same ignore-aware directory walking behaviour as common developer tools, so
files excluded by ignore rules are skipped before any include or exclude filters
are applied.

## Install

### AUR

The package is available in the AUR, install it with your preferred AUR
helper:

```sh
paru -S dumpr
```

or:

```sh
yay -S dumpr
```

### From git

You need a working Rust toolchain with Cargo installed.

Clone the repository, build the binary, and install it into Cargo's bin
directory:

```sh
git clone https://github.com/iktrnch/dumpr.git
cd dumpr
cargo install --path .
```

After installation, make sure Cargo's bin directory is on your `PATH`. It is
usually `~/.cargo/bin`.

You can also run dumpr directly from the repository while developing:

```sh
cargo run -- --tree --files
```

## Usage

Print the tree for the current directory:

```sh
dumpr --tree
```

Print the contents of files in the current directory:

```sh
dumpr --files
```

Print both the tree and file contents for another directory:

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

The `--include` and `--exclude` values are regular expressions matched against
file paths. Ignore rules are applied first, then the include filter, then the
exclude filter.

### Arguments

`-d, --directory <DIRECTORY>`

Directory to read. Defaults to the current directory.

`-t, --tree`

Print a tree view of the matching files.

`-f, --files`

Print the contents of the matching files. Each file is separated by a header
containing its path.

`-i, --include <INCLUDE>`

Only include files whose path matches this regular expression. Defaults to an
empty pattern, which matches every path that was not ignored.

`-e, --exclude <EXCLUDE>`

Exclude files whose path matches this regular expression. Defaults to `^$`,
which does not match normal file paths.

`-h, --help`

Print command line help.

`-V, --version`

Print the installed dumpr version.
