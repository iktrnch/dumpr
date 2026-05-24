#!/usr/bin/env python3

from __future__ import annotations

import argparse
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPTS = ROOT / "scripts"


def find_script(*names: str) -> Path:
    for name in names:
        path = SCRIPTS / name
        if path.exists():
            return path

    joined = ", ".join(names)
    raise FileNotFoundError(f"Could not find any of: {joined}")


def run(cmd: list[str]) -> None:
    print()
    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}")
    subprocess.run(cmd, cwd=ROOT, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run full dumpr benchmark workflow.")

    parser.add_argument(
        "--repo",
        action="append",
        help="Repo name from corpus.toml. Passed to validate and hyperfine.",
    )

    parser.add_argument("--skip-setup", action="store_true")
    parser.add_argument("--skip-validate", action="store_true")
    parser.add_argument("--skip-hyperfine", action="store_true")
    parser.add_argument("--skip-profile", action="store_true")

    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Do not run cargo build --release in the orchestrator.",
    )

    parser.add_argument(
        "--warmup",
        type=int,
        default=3,
        help="Warmup count passed to bench_hyperfine.py.",
    )

    parser.add_argument(
        "--min-runs",
        type=int,
        default=10,
        help="Minimum run count passed to bench_hyperfine.py.",
    )

    parser.add_argument(
        "--profile-repo",
        default="bevy",
        help="Repo to use for flamegraph profiling.",
    )

    parser.add_argument(
        "--profile-mode",
        default="tree_files",
        choices=["tree", "files", "tree_files", "rust_only"],
        help="Mode to use for flamegraph profiling.",
    )

    args = parser.parse_args()

    setup_script = find_script("bench_setup.py")
    validate_script = find_script("bench_validate.py")
    hyperfine_script = find_script("bench_hyperfine.py", "bench_hyprfine.py")
    profile_script = find_script("bench_profile.py")

    python = sys.executable

    repo_args: list[str] = []
    if args.repo:
        for repo in args.repo:
            repo_args.extend(["--repo", repo])

    if not args.skip_setup:
        run([python, str(setup_script)])

    if not args.no_build:
        run(["cargo", "build", "--release"])

    no_build_arg = ["--no-build"]

    if not args.skip_validate:
        run(
            [
                python,
                str(validate_script),
                *no_build_arg,
                *repo_args,
            ]
        )

    if not args.skip_hyperfine:
        run(
            [
                python,
                str(hyperfine_script),
                *no_build_arg,
                "--warmup",
                str(args.warmup),
                "--min-runs",
                str(args.min_runs),
                *repo_args,
            ]
        )

    if not args.skip_profile:
        run(
            [
                python,
                str(profile_script),
                *no_build_arg,
                "--repo",
                args.profile_repo,
                "--mode",
                args.profile_mode,
            ]
        )


if __name__ == "__main__":
    main()
