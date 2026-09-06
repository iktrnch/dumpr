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
RELEASE_BIN = ROOT / "target" / "release" / "dumpr"
PROFILE_BIN = ROOT / "target" / "profiling" / "dumpr"

HF_MODES = {
    "tree": ["--tree"],
    "files": ["--files"],
    "tree+files": ["--tree", "--files"],
    "rust-only": ["--tree", "--files", "--include", "*.rs"],
    "exclude-generated": [
        "--tree",
        "--files",
        "--exclude",
        "target/**",
        "--exclude",
        "node_modules/**",
        "--exclude",
        "dist/**",
        "--exclude",
        "build/**",
        "--exclude",
        ".git/**",
    ],
}

FG_MODES = {
    "tree": ["--tree"],
    "files": ["--files"],
    "tree_files": ["--tree", "--files"],
    "rust_only": ["--tree", "--files", "--include", "*.rs"],
}


def run(
    cmd: list[str],
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    stdout: int | None = None,
) -> None:
    print(f"+ {' '.join(shlex.quote(part) for part in cmd)}", flush=True)
    subprocess.run(cmd, cwd=cwd, env=env, stdout=stdout, check=True)


def load_repos(names: list[str] | None) -> list[dict[str, str]]:
    with CORPUS_FILE.open("rb") as f:
        repos = tomllib.load(f).get("repo", [])

    if not isinstance(repos, list):
        raise ValueError("corpus.toml must contain [[repo]] entries")

    for repo in repos:
        for key in ("name", "url", "rev"):
            if key not in repo:
                raise ValueError(f"repo entry is missing `{key}`: {repo}")

    if not names:
        return repos

    wanted = set(names)
    selected = [repo for repo in repos if repo["name"] in wanted]
    missing = wanted - {repo["name"] for repo in selected}

    if missing:
        raise ValueError(f"unknown repo(s): {', '.join(sorted(missing))}")

    return selected


def clone_repos(repos: list[dict[str, str]]) -> None:
    REPOS_DIR.mkdir(parents=True, exist_ok=True)

    for repo in repos:
        repo_dir = REPOS_DIR / repo["name"]

        if repo_dir.exists():
            shutil.rmtree(repo_dir)

        run(["git", "clone", repo["url"], str(repo_dir)])
        run(["git", "checkout", "--detach", repo["rev"]], cwd=repo_dir)
        run(["git", "reset", "--hard", repo["rev"]], cwd=repo_dir)
        run(["git", "clean", "-xfd"], cwd=repo_dir)
        print(f"ready: {repo['name']} @ {repo['rev']}")


def ensure_repos_exist(repos: list[dict[str, str]]) -> None:
    missing = [
        repo["name"] for repo in repos if not (REPOS_DIR / repo["name"]).exists()
    ]

    if missing:
        raise FileNotFoundError(
            "missing benchmark repo(s): "
            f"{', '.join(missing)}. Run `./bench.py --clone` first."
        )


def build_release() -> None:
    run(["cargo", "build", "--release"], cwd=ROOT)

    if not RELEASE_BIN.exists():
        raise FileNotFoundError(f"{RELEASE_BIN} was not built")


def smoke_test(repos: list[dict[str, str]]) -> None:
    cases = [
        ["--tree"],
        [
            "--files",
            "--include",
            "README*",
            "--include",
            "Cargo.toml",
            "--include",
            "package.json",
        ],
        [
            "--tree",
            "--files",
            "--include",
            "README*",
            "--include",
            "Cargo.toml",
            "--include",
            "package.json",
        ],
    ]

    for repo in repos:
        repo_dir = REPOS_DIR / repo["name"]

        for args in cases:
            run(
                [str(RELEASE_BIN), str(repo_dir), *args],
                cwd=ROOT,
                stdout=subprocess.DEVNULL,
            )

        print(f"smoke ok: {repo['name']}")


def dumpr_command(repo_dir: Path, args: list[str]) -> str:
    parts = [str(RELEASE_BIN), str(repo_dir), *args]
    return " ".join(shlex.quote(part) for part in parts) + " > /dev/null"


def run_hyperfine(repos: list[dict[str, str]], warmup: int, min_runs: int) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    json_out = RESULTS_DIR / f"hyperfine-{timestamp}.json"
    md_out = RESULTS_DIR / f"hyperfine-{timestamp}.md"
    cmd = [
        "hyperfine",
        "--warmup",
        str(warmup),
        "--min-runs",
        str(min_runs),
        "--export-json",
        str(json_out),
        "--export-markdown",
        str(md_out),
    ]

    for repo in repos:
        repo_dir = REPOS_DIR / repo["name"]

        for mode, args in HF_MODES.items():
            cmd.extend(["--command-name", f"{repo['name']}: {mode}"])
            cmd.append(dumpr_command(repo_dir, args))

    run(cmd, cwd=ROOT)
    link_latest("latest.json", json_out)
    link_latest("latest.md", md_out)
    print(f"hyperfine json: {json_out}")
    print(f"hyperfine md:   {md_out}")


def find_flamegraph() -> str:
    flamegraph = shutil.which("flamegraph")

    if flamegraph:
        return flamegraph

    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    flamegraph = cargo_home / "bin" / "flamegraph"

    if flamegraph.exists():
        return str(flamegraph)

    raise RuntimeError("flamegraph not found; install with `cargo install flamegraph`")


def build_profile() -> None:
    env = os.environ.copy()
    rustflags = env.get("RUSTFLAGS", "")
    env["RUSTFLAGS"] = f"{rustflags} -C force-frame-pointers=yes".strip()
    run(["cargo", "build", "--profile", "profiling"], cwd=ROOT, env=env)

    if not PROFILE_BIN.exists():
        raise FileNotFoundError(f"{PROFILE_BIN} was not built")


def default_fg_repo(repos: list[dict[str, str]]) -> str:
    names = {repo["name"] for repo in repos}

    if "bevy" in names:
        return "bevy"

    if repos:
        return repos[0]["name"]

    raise ValueError("no repos selected")


def run_flamegraph(
    repos: list[dict[str, str]],
    repo_name: str | None,
    mode: str,
    iterations: int,
    freq: int,
) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    flamegraph = find_flamegraph()
    build_profile()

    selected = repo_name or default_fg_repo(repos)
    repo_dir = REPOS_DIR / selected

    if not repo_dir.exists():
        raise FileNotFoundError(f"{repo_dir} does not exist")

    loop = (
        'i=0; while [ "$i" -lt "$1" ]; do '
        '"$2" "$3" "${@:4}" > /dev/null; '
        "i=$((i + 1)); done"
    )
    timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    output = RESULTS_DIR / f"flamegraph-{selected}-{mode}-{timestamp}.svg"
    cmd = [
        flamegraph,
        "--freq",
        str(freq),
        "-o",
        str(output),
        "--",
        "bash",
        "-c",
        loop,
        "dumpr-profile-loop",
        str(iterations),
        str(PROFILE_BIN),
        str(repo_dir),
        *FG_MODES[mode],
    ]

    run(cmd, cwd=ROOT)
    link_latest("latest-flamegraph.svg", output)
    print(f"flamegraph svg: {output}")


def link_latest(name: str, target: Path) -> None:
    latest = RESULTS_DIR / name
    latest.unlink(missing_ok=True)
    latest.symlink_to(target.name)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run dumpr benchmark workflow.")
    parser.add_argument("--clone", action="store_true", help="Reclone repos and exit.")
    parser.add_argument("--no-smoke", action="store_true", help="Skip smoke tests.")
    parser.add_argument("--no-hf", action="store_true", help="Skip hyperfine.")
    parser.add_argument("--no-fg", action="store_true", help="Skip flamegraph.")
    parser.add_argument("--repo", action="append", help="Limit work to a corpus repo.")
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument("--min-runs", type=int, default=10)
    parser.add_argument("--fg-repo", help="Repo to profile; defaults to bevy or first.")
    parser.add_argument("--fg-mode", choices=sorted(FG_MODES), default="tree_files")
    parser.add_argument("--fg-iterations", type=int, default=100)
    parser.add_argument("--fg-freq", type=int, default=997)
    args = parser.parse_args()

    repos = load_repos(args.repo)

    if args.clone:
        clone_repos(repos)
        return

    ensure_repos_exist(repos)
    build_release()

    if not args.no_smoke:
        smoke_test(repos)

    if not args.no_hf:
        run_hyperfine(repos, args.warmup, args.min_runs)

    if not args.no_fg:
        run_flamegraph(
            repos,
            args.fg_repo,
            args.fg_mode,
            args.fg_iterations,
            args.fg_freq,
        )


if __name__ == "__main__":
    main()
