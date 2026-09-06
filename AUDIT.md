# Bug and Performance Audit

Audit date: 2026-09-06

Scope: `src/`, the CLI integration tests, public README examples, and the
benchmark driver. The audit combined source review, adversarial CLI probes, the
existing test suite, Clippy, and the project's benchmark workflow. Items below
are the concrete issues found in this checkout; "fixed" means a regression test
or another named verification now covers the behavior.

## Correctness and safety

| ID | Severity | Finding | Resolution | Status |
| --- | --- | --- | --- | --- |
| C-01 | High | `Path::to_str().unwrap()` crashed on non-UTF-8 paths, and splitting paths on literal `/` was not platform-safe. Absolute paths also appeared as a duplicated directory chain below the root. | Keep paths as `Path`/`PathBuf`/`OsString`, derive entries with `strip_prefix`, and render names lossily only at the output boundary. | Fixed |
| C-02 | High | `path.is_file()` followed file symlinks. A symlink inside the requested tree could therefore disclose a readable file outside that tree. | Inspect the walker's non-following file type and skip every yielded symlink with a path warning. A symlink supplied as the root is rejected. | Fixed |
| C-03 | High | Redirecting stdout into the walked directory caused dumpr to ingest the file it was actively producing, duplicating partial output into itself. | Capture stdout's file identity and omit a matching entry with a warning. Unix compares device/inode metadata without opening every candidate. | Fixed |
| C-04 | High | `write_files` discarded every per-file error, including output-writer failures, so truncated output could still exit successfully. | Only expected input problems are warning-and-skip cases. Header/body/output failures now propagate and cause a non-zero exit. | Fixed |
| C-05 | Medium | Missing roots and other root-walk failures printed an empty tree and exited successfully. | Validate with `symlink_metadata` before walking; missing and non-directory roots fail before stdout is locked or written. | Fixed |
| C-06 | Medium | Malformed include/exclude globs were ignored, potentially turning a narrow command into an unexpectedly broad dump. | Build and validate all overrides during `Digest::new`; malformed patterns fail before stdout. | Fixed |
| C-07 | Medium | Binary, invalid-UTF-8, and unreadable files disappeared from the content dump without an explanation. | Validate text explicitly and warn to stderr with the skipped path. Open/read failures use the same warning-and-skip policy. | Fixed |
| C-08 | Medium | Running `dumpr` without `--tree` or `--files` was a silent no-op. | Neither flag now selects both, exactly like `dumpr . --tree --files`; one explicit flag still selects only its section. | Fixed |
| C-09 | Low | Tree order depended on filesystem traversal order, making otherwise identical dumps unstable between filesystems or runs. | Store children and file names in `BTreeMap`s for deterministic lexical order. | Fixed |
| C-10 | Low | Header borders used UTF-8 byte length, so paths containing multibyte characters produced visibly overlong borders. | Size borders using terminal display width. | Fixed |
| C-11 | Low | Closing a downstream pipe early (for example, `dumpr --tree | head`) was reported as an application error. | Treat `BrokenPipe` as normal Unix pipeline termination while continuing to propagate other output errors. | Fixed |

## Performance and resource use

| ID | Impact | Finding | Resolution | Status |
| --- | --- | --- | --- | --- |
| P-01 | Memory | `fs::read_to_string` allocated the full contents of each file. Peak memory grew with the largest matching file. | Validate UTF-8/NULs in bounded 64 KiB chunks, rewind, then stream valid files with `io::copy`. Peak content-buffer memory is constant. | Fixed |
| P-02 | CPU | Inserting a file scanned every sibling directory linearly. Wide trees could approach quadratic insertion work. | Use ordered-map lookup for `O(log siblings)` insertion while also guaranteeing output order. | Fixed |
| P-03 | Syscalls | Writes went directly through `StdoutLock`; the many small tree/header writes were not explicitly buffered. | Wrap stdout in `BufWriter`, then flush once at the end. | Fixed |
| P-04 | Build size/time | The direct `regex` dependency was unused; filtering was already implemented by `ignore`/`globset`. | Remove the unused dependency and document glob syntax consistently. | Fixed |
| P-05 | Avoidable work | `Args` was cloned only to retain flags after constructing `DigestOptions`. | Resolve output selection first, then move `Args` into the options. | Fixed |

Text validation intentionally reads a valid file twice: once to ensure no
partial binary body reaches stdout, and once to stream it. This trades extra
page-cache-friendly reads for bounded memory and all-or-nothing binary
skipping. A file concurrently modified between those passes can still produce
a changed body; dumpr is a snapshot formatter, not a filesystem-consistency or
backup tool.

### Benchmark snapshot

The repaired Hyperfine workflow completed against the pinned ripgrep corpus.
Mean times were 1.3 ms for tree-only, 2.1 ms for files-only and tree-plus-files,
1.5 ms for Rust-only, and 2.2 ms with generated-directory exclusions. A local
before/after comparison measured tree-only at 1.3/1.2 ms and tree-plus-files at
1.9/2.0 ms respectively. Every command was below Hyperfine's 5 ms precision
threshold, so these figures support "no material latency regression," not a
speedup claim. The substantive performance wins are bounded peak file memory,
better wide-tree scaling, and buffered output.

## Documentation and tooling

| ID | Finding | Resolution | Status |
| --- | --- | --- | --- |
| D-01 | README advertised regex filters while the CLI and tests implemented gitignore-style globs. | Rewrite filtering prose and examples around globs and repeated flags. | Fixed |
| D-02 | README documented `-d/--directory`, but the CLI intentionally accepts only a positional directory. | Document `[DIRECTORY]` and use it as the first argument in every example. | Fixed |
| D-03 | The benchmark smoke, hyperfine, and flamegraph commands all used the nonexistent `--directory` flag and regex filters, so the workflow failed before measuring dumpr. | Use the positional root and equivalent glob lists throughout `scripts/bench.py`. | Fixed |
| D-04 | Public documentation did not explain skipped files, symlinks, invalid-input output guarantees, or redirected-output protection. | Add the agreed behavior to README Notes and CLI reference. | Fixed |

## Verification map

`tests/cli_args.rs` covers the public CLI contract: positional-only roots,
implicit `-tf`, multiple glob filters, invalid-root/glob empty-stdout failures,
binary warnings, symlink warnings, redirected-output exclusion, stable order,
correct absolute-root structure, non-UTF-8 names, and closed-pipe behavior. The
library tests exercise writer-error propagation, Unicode header width, and a
multibyte UTF-8 code point split across validation-buffer boundaries.
