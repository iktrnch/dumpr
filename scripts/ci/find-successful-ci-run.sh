#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
    echo "usage: $0 <workflow-file> <branch>" >&2
    exit 2
}

workflow_file="$1"
branch="$2"
repository="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
output_file="${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"
commit_sha="$(git rev-parse 'HEAD^{commit}')"
runs="$(gh api --method GET \
    "repos/$repository/actions/workflows/$workflow_file/runs" \
    -f head_sha="$commit_sha" \
    -f branch="$branch" \
    -f event=push \
    -f status=success \
    -f per_page=100)"
ci_run_id="$(jq -r --arg sha "$commit_sha" --arg branch "$branch" '
    [
      .workflow_runs[]
      | select(.head_sha == $sha)
      | select(.event == "push")
      | select(.head_branch == $branch)
      | select(.conclusion == "success")
    ]
    | max_by(.id)
    | .id // empty
' <<<"$runs")"

if ! [[ "$ci_run_id" =~ ^[0-9]+$ ]]; then
    echo "::error::No successful CI push run on $branch exists for commit $commit_sha." >&2
    echo "::error::Push this exact commit to $branch and wait for CI to succeed before creating a release tag." >&2
    exit 1
fi

echo "Resolved commit $commit_sha to successful CI run $ci_run_id"
echo "ci_run_id=$ci_run_id" >> "$output_file"
