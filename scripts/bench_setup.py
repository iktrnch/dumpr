#!/usr/bin/env python3

from __future__ import annotations

import subprocess
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parent.parent
CORPUS_FILE = ROOT / "benches" / "corpus.toml"
REPOS_DIR = ROOT / ".bench" / "repos"


def run(cmd: list[str], cwd: Path | None = None) -> None:
    print(f"+ {' '.join(cmd)}")
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


def setup_repo(name: str, url: str, rev: str) -> None:
    repo_dir = REPOS_DIR / name

    if not repo_dir.exists():
        run(["git", "clone", url, str(repo_dir)])
    else:
        if not (repo_dir / ".git").exists():
            raise RuntimeError(f"{repo_dir} exists but is not a git repo")

    run(["git", "fetch", "--all", "--tags", "--prune"], cwd=repo_dir)

    # Checkout exact pinned commit.
    run(["git", "checkout", "--detach", rev], cwd=repo_dir)

    # Make sure the working tree exactly matches the pinned commit.
    run(["git", "reset", "--hard", rev], cwd=repo_dir)

    # Remove untracked/generated files from previous benchmark runs.
    run(["git", "clean", "-xfd"], cwd=repo_dir)

    print(f"ready: {name} @ {rev}")


def main() -> None:
    REPOS_DIR.mkdir(parents=True, exist_ok=True)

    repos = load_corpus()

    for repo in repos:
        setup_repo(
            name=repo["name"],
            url=repo["url"],
            rev=repo["rev"],
        )


if __name__ == "__main__":
    main()
