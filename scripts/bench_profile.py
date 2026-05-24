#!/usr/bin/env python3

from __future__ import annotations

import argparse
import datetime as dt
import os
import shlex
import shutil
import subprocess
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parent.parent
CORPUS_FILE = ROOT / "benches" / "corpus.toml"
REPOS_DIR = ROOT / ".bench" / "repos"
RESULTS_DIR = ROOT / "benches" / "results"
BIN = ROOT / "target" / "profiling" / "dumpr"


MODES = {
    "tree": ["--tree"],
    "files": ["--files"],
    "tree_files": ["--tree", "--files"],
    "rust_only": ["--tree", "--files", "--include", r"\.rs$"],
}


def run(
    cmd: list[str], cwd: Path | None = None, env: dict[str, str] | None = None
) -> None:
    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}")

    result = subprocess.run(cmd, cwd=cwd, env=env)

    if result.returncode != 0:
        raise SystemExit(result.returncode)


def load_corpus() -> list[dict[str, str]]:
    with CORPUS_FILE.open("rb") as f:
        data = tomllib.load(f)

    repos = data.get("repo", [])

    if not isinstance(repos, list):
        raise ValueError("corpus.toml must contain [[repo]] entries")

    return repos


def default_repo_name(repos: list[dict[str, str]]) -> str:
    names = {repo["name"] for repo in repos}

    if "bevy" in names:
        return "bevy"

    if repos:
        return repos[0]["name"]

    raise ValueError("No repos found in corpus.toml")


def find_flamegraph() -> str:
    flamegraph = shutil.which("flamegraph")

    if flamegraph is not None:
        return flamegraph

    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    cargo_flamegraph = cargo_home / "bin" / "flamegraph"

    if cargo_flamegraph.exists():
        return str(cargo_flamegraph)

    raise RuntimeError(
        "flamegraph not found. Install it with `cargo install flamegraph` "
        "and make sure ~/.cargo/bin is in PATH."
    )


def build_profile_binary() -> None:
    env = os.environ.copy()

    existing_rustflags = env.get("RUSTFLAGS", "")
    profiling_rustflags = "-C force-frame-pointers=yes"

    env["RUSTFLAGS"] = (
        f"{existing_rustflags} {profiling_rustflags}".strip()
        if existing_rustflags
        else profiling_rustflags
    )

    run(["cargo", "build", "--profile", "profiling"], cwd=ROOT, env=env)


def main() -> None:
    parser = argparse.ArgumentParser(description="Profile dumpr with flamegraph.")

    parser.add_argument(
        "--repo",
        help="Repo name from corpus.toml. Defaults to bevy if present, otherwise first repo.",
    )
    parser.add_argument(
        "--mode",
        choices=sorted(MODES.keys()),
        default="tree_files",
        help="dumpr mode to profile.",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip cargo build --profile profiling.",
    )
    parser.add_argument(
        "--output",
        help="Custom output SVG path.",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=100,
        help="Run dumpr this many times under perf to collect more samples.",
    )
    parser.add_argument(
        "--freq",
        type=int,
        default=997,
        help="perf sampling frequency passed to flamegraph.",
    )

    args = parser.parse_args()

    flamegraph = find_flamegraph()

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    if not args.no_build:
        build_profile_binary()

    if not BIN.exists():
        raise FileNotFoundError(
            f"{BIN} does not exist. Run cargo build --profile profiling first."
        )

    repos = load_corpus()
    repo_name = args.repo or default_repo_name(repos)
    repo_dir = REPOS_DIR / repo_name

    if not repo_dir.exists():
        raise FileNotFoundError(
            f"{repo_dir} does not exist. Run scripts/bench_setup.py first."
        )

    timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")

    if args.output:
        output = Path(args.output)
    else:
        output = RESULTS_DIR / f"flamegraph-{repo_name}-{args.mode}-{timestamp}.svg"

    output.parent.mkdir(parents=True, exist_ok=True)

    dumpr_args = MODES[args.mode]

    loop_script = """
iterations="$1"
bin="$2"
repo="$3"
shift 3

i=0
while [ "$i" -lt "$iterations" ]; do
    "$bin" --directory "$repo" "$@" > /dev/null
    i=$((i + 1))
done
"""

    cmd = [
        flamegraph,
        "--freq",
        str(args.freq),
        "-o",
        str(output),
        "--",
        "bash",
        "-lc",
        loop_script,
        "dumpr-profile-loop",
        str(args.iterations),
        str(BIN),
        str(repo_dir),
        *dumpr_args,
    ]

    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}")

    result = subprocess.run(cmd, cwd=ROOT)

    if result.returncode != 0:
        print()
        print(f"FAILED: flamegraph exited with code {result.returncode}")
        raise SystemExit(result.returncode)

    latest = RESULTS_DIR / "latest-flamegraph.svg"
    latest.unlink(missing_ok=True)
    latest.symlink_to(output.name)

    print()
    print(f"Saved flamegraph: {output}")
    print(f"Latest symlink:   {latest}")


if __name__ == "__main__":
    main()
