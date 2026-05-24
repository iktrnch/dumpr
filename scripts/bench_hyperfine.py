#!/usr/bin/env python3

from __future__ import annotations

import argparse
import datetime as dt
import shlex
import subprocess
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parent.parent
CORPUS_FILE = ROOT / "benches" / "corpus.toml"
REPOS_DIR = ROOT / ".bench" / "repos"
RESULTS_DIR = ROOT / "benches" / "results"
BIN = ROOT / "target" / "release" / "dumpr"


def run(cmd: list[str], cwd: Path | None = None) -> None:
    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}")
    subprocess.run(cmd, cwd=cwd, check=True)


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


def command_name(repo_name: str, mode_name: str) -> str:
    return f"{repo_name}: {mode_name}"


def dumpr_command(repo_path: Path, args: list[str]) -> str:
    # hyperfine commands are shell strings, so quote paths/args safely.
    parts = [str(BIN), "-d", str(repo_path), *args]
    return " ".join(shlex.quote(part) for part in parts) + " > /dev/null"


def build_commands(repos: list[dict[str, str]]) -> list[tuple[str, str]]:
    commands: list[tuple[str, str]] = []

    for repo in repos:
        name = repo["name"]
        repo_path = REPOS_DIR / name

        if not repo_path.exists():
            raise FileNotFoundError(
                f"{repo_path} does not exist. Run scripts/bench-setup.py first."
            )

        commands.extend(
            [
                (
                    command_name(name, "tree"),
                    dumpr_command(repo_path, ["--tree"]),
                ),
                (
                    command_name(name, "files"),
                    dumpr_command(repo_path, ["--files"]),
                ),
                (
                    command_name(name, "tree+files"),
                    dumpr_command(repo_path, ["--tree", "--files"]),
                ),
                (
                    command_name(name, "rust-only"),
                    dumpr_command(
                        repo_path, ["--tree", "--files", "--include", r"\.rs$"]
                    ),
                ),
                (
                    command_name(name, "exclude-generated"),
                    dumpr_command(
                        repo_path,
                        [
                            "--tree",
                            "--files",
                            "--exclude",
                            r"target/|node_modules/|dist/|build/|\.git/",
                        ],
                    ),
                ),
            ]
        )

    return commands


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark dumpr with hyperfine.")
    parser.add_argument(
        "--warmup",
        type=int,
        default=3,
        help="Number of warmup runs for hyperfine.",
    )
    parser.add_argument(
        "--min-runs",
        type=int,
        default=10,
        help="Minimum number of benchmark runs per command.",
    )
    parser.add_argument(
        "--repo",
        action="append",
        help="Only benchmark a specific repo name. Can be passed multiple times.",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip cargo build --release.",
    )
    args = parser.parse_args()

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    if not args.no_build:
        run(["cargo", "build", "--release"], cwd=ROOT)

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

    if not repos:
        raise ValueError("No repos selected for benchmarking.")

    timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    json_out = RESULTS_DIR / f"hyperfine-{timestamp}.json"
    md_out = RESULTS_DIR / f"hyperfine-{timestamp}.md"

    commands = build_commands(repos)

    hyperfine_cmd = [
        "hyperfine",
        "--warmup",
        str(args.warmup),
        "--min-runs",
        str(args.min_runs),
        "--export-json",
        str(json_out),
        "--export-markdown",
        str(md_out),
    ]

    for name, command in commands:
        hyperfine_cmd.extend(["--command-name", name, command])

    run(hyperfine_cmd, cwd=ROOT)

    latest_json = RESULTS_DIR / "latest.json"
    latest_md = RESULTS_DIR / "latest.md"

    latest_json.unlink(missing_ok=True)
    latest_md.unlink(missing_ok=True)

    latest_json.symlink_to(json_out.name)
    latest_md.symlink_to(md_out.name)

    print()
    print(f"Saved JSON:     {json_out}")
    print(f"Saved Markdown: {md_out}")
    print(f"Latest JSON:    {latest_json}")
    print(f"Latest MD:      {latest_md}")


if __name__ == "__main__":
    main()
