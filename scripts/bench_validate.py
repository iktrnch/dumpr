#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import shlex
import subprocess
import tempfile
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parent.parent
CORPUS_FILE = ROOT / "benches" / "corpus.toml"
REPOS_DIR = ROOT / ".bench" / "repos"
RESULTS_DIR = ROOT / "benches" / "results"
BIN = ROOT / "target" / "release" / "dumpr"

SEPARATOR = "========================================"


CASES = [
    {
        "name": "files",
        "dumpr_args": ["--files"],
        "include": "",
        "exclude": r"^$",
    },
    {
        "name": "rust_only",
        "dumpr_args": ["--files", "--include", r"\.rs$"],
        "include": r"\.rs$",
        "exclude": r"^$",
    },
    {
        "name": "exclude_lock",
        "dumpr_args": ["--files", "--exclude", r"\.lock$"],
        "include": "",
        "exclude": r"\.lock$",
    },
]


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}")
    return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)


def load_corpus() -> list[dict[str, str]]:
    with CORPUS_FILE.open("rb") as f:
        data = tomllib.load(f)

    repos = data.get("repo", [])

    if not isinstance(repos, list):
        raise ValueError("corpus.toml must contain [[repo]] entries")

    for repo in repos:
        for key in ["name", "url", "rev"]:
            if key not in repo:
                raise ValueError(f"repo entry is missing `{key}`: {repo}")

    return repos


def build_release() -> None:
    result = run(["cargo", "build", "--release"], cwd=ROOT)

    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr)
        raise SystemExit(result.returncode)


def gitignored_paths(repo_dir: Path, paths: list[Path]) -> set[Path]:
    if not paths:
        return set()

    rel_paths = [path.relative_to(repo_dir).as_posix() for path in paths]
    stdin = "\0".join(rel_paths) + "\0"
    result = subprocess.run(
        ["git", "check-ignore", "--stdin", "-z", "--no-index"],
        cwd=repo_dir,
        input=stdin,
        text=True,
        capture_output=True,
        check=False,
    )

    # git check-ignore exits with 1 when no paths match. Other codes indicate a
    # real failure in applying the repository's ignore rules.
    if result.returncode not in {0, 1}:
        raise RuntimeError(result.stderr)

    return {Path(raw_path) for raw_path in result.stdout.split("\0") if raw_path}


def is_hidden_relative_path(path: Path) -> bool:
    return any(part.startswith(".") for part in path.parts)


def symlink_file_candidates(repo_dir: Path) -> list[Path]:
    paths: list[Path] = []

    for path in repo_dir.rglob("*"):
        rel = path.relative_to(repo_dir)

        if ".git" in rel.parts:
            continue

        if is_hidden_relative_path(rel):
            continue

        if path.is_symlink() and path.is_file():
            paths.append(path)

    ignored = gitignored_paths(repo_dir, paths) if (repo_dir / ".git").exists() else set()
    return [path for path in paths if path.relative_to(repo_dir) not in ignored]


def rg_file_candidates(repo_dir: Path) -> list[Path] | None:
    try:
        result = subprocess.run(
            ["rg", "--files", "-0"],
            cwd=repo_dir,
            text=True,
            capture_output=True,
            check=False,
        )
    except FileNotFoundError:
        return None

    if result.returncode not in {0, 1}:
        raise RuntimeError(result.stderr)

    paths = [repo_dir / raw_path for raw_path in result.stdout.split("\0") if raw_path]
    paths.extend(path for path in symlink_file_candidates(repo_dir) if path not in paths)
    return paths


def find_file_candidates(repo_dir: Path) -> list[Path]:
    paths: list[Path] = []

    for path in repo_dir.rglob("*"):
        if not path.is_file():
            continue

        rel = path.relative_to(repo_dir)

        if ".git" in rel.parts:
            continue

        paths.append(path)

    return paths


def is_utf8_readable(path: Path) -> bool:
    try:
        path.read_text(encoding="utf-8")
        return True
    except UnicodeDecodeError:
        return False
    except OSError:
        return False


def expected_files(
    repo_dir: Path, include_pattern: str, exclude_pattern: str
) -> set[str]:
    include_re = re.compile(include_pattern)
    exclude_re = re.compile(exclude_pattern)

    candidates = rg_file_candidates(repo_dir)
    ignored: set[Path] = set()

    if candidates is None:
        candidates = find_file_candidates(repo_dir)
        if (repo_dir / ".git").exists():
            ignored = gitignored_paths(repo_dir, candidates)

    expected: set[str] = set()

    for path in candidates:
        if not path.is_file():
            continue

        if path.relative_to(repo_dir) in ignored:
            continue

        path_str = str(path)

        if not include_re.search(path_str):
            continue

        if exclude_re.search(path_str):
            continue

        if not is_utf8_readable(path):
            continue

        expected.add(path_str)

    return expected


def extract_dumpr_file_headers(output_file: Path) -> tuple[set[str], int]:
    headers: set[str] = set()
    malformed_blocks = 0

    state = 0
    candidate: str | None = None

    with output_file.open("r", encoding="utf-8", errors="replace") as f:
        for raw_line in f:
            line = raw_line.rstrip("\n")

            if state == 0:
                if line == SEPARATOR:
                    state = 1
                continue

            if state == 1:
                candidate = line
                state = 2
                continue

            if state == 2:
                if line == SEPARATOR and candidate is not None:
                    headers.add(candidate)
                else:
                    malformed_blocks += 1

                candidate = None
                state = 0

    return headers, malformed_blocks


def validate_case(
    repo_name: str,
    repo_dir: Path,
    case: dict[str, object],
    keep_output: bool,
    limit: int,
    min_ratio: float,
    strict: bool,
) -> bool:
    case_name = str(case["name"])
    dumpr_args = list(case["dumpr_args"])  # type: ignore[arg-type]
    include = str(case["include"])
    exclude = str(case["exclude"])

    expected = expected_files(repo_dir, include, exclude)

    if keep_output:
        output_path = RESULTS_DIR / f"validate-{repo_name}-{case_name}.txt"
        output_file = output_path.open("w", encoding="utf-8")
    else:
        temp = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            delete=False,
            prefix=f"dumpr-{repo_name}-{case_name}-",
            suffix=".txt",
        )
        output_path = Path(temp.name)
        output_file = temp

    cmd = [str(BIN), "--directory", str(repo_dir), *dumpr_args]

    print(f"+ {' '.join(shlex.quote(part) for part in cmd)} > {output_path}")

    try:
        result = subprocess.run(
            cmd,
            cwd=ROOT,
            text=True,
            stdout=output_file,
            stderr=subprocess.PIPE,
            check=False,
        )
    finally:
        output_file.close()

    if result.returncode != 0:
        print(f"FAIL {repo_name}:{case_name}")
        print(result.stderr)
        return False

    actual, malformed_blocks = extract_dumpr_file_headers(output_path)

    if not keep_output:
        output_path.unlink(missing_ok=True)

    missing = sorted(expected - actual)
    extra = sorted(actual - expected)

    ratio = 1.0 if not expected else len(actual & expected) / len(expected)

    fail = False

    if malformed_blocks > 0:
        fail = True

    if not actual:
        fail = True

    if extra:
        fail = True

    if ratio < min_ratio:
        fail = True

    if strict and missing:
        fail = True

    status = "PASS" if not fail else "FAIL"

    print(f"{status} {repo_name}:{case_name}")
    print(f"  expected readable files: {len(expected)}")
    print(f"  actual dumped files:     {len(actual)}")
    print(f"  match ratio:             {ratio:.2%}")
    print(f"  malformed blocks:        {malformed_blocks}")

    if missing:
        label = "missing from dumpr output"
        print(f"  {label}: {len(missing)}")
        for path in missing[:limit]:
            print(f"    - {path}")

    if extra:
        print(f"  extra in dumpr output: {len(extra)}")
        for path in extra[:limit]:
            print(f"    + {path}")

    return not fail


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Validate dumpr output against repo file lists."
    )

    parser.add_argument("--repo", action="append")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--keep-output", action="store_true")
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--min-ratio", type=float, default=0.95)
    parser.add_argument("--strict", action="store_true")

    args = parser.parse_args()

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    if not args.no_build:
        build_release()

    if not BIN.exists():
        raise FileNotFoundError(
            f"{BIN} does not exist. Run cargo build --release first."
        )

    repos = load_corpus()

    if args.repo:
        wanted = set(args.repo)
        repos = [repo for repo in repos if repo["name"] in wanted]

        missing = wanted - {repo["name"] for repo in repos}
        if missing:
            raise ValueError(
                f"Unknown repo(s) in corpus.toml: {', '.join(sorted(missing))}"
            )

    all_ok = True

    for repo in repos:
        repo_name = repo["name"]
        repo_dir = REPOS_DIR / repo_name

        if not repo_dir.exists():
            raise FileNotFoundError(
                f"{repo_dir} does not exist. Run scripts/bench_setup.py first."
            )

        for case in CASES:
            ok = validate_case(
                repo_name=repo_name,
                repo_dir=repo_dir,
                case=case,
                keep_output=args.keep_output,
                limit=args.limit,
                min_ratio=args.min_ratio,
                strict=args.strict,
            )
            all_ok = all_ok and ok

    if not all_ok:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
