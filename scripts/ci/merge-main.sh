#!/usr/bin/env bash
# Put this checkout where a pull request's writers run: main's head with the
# branch merged in, not yet committed.
#
#     BRANCH=<head branch> scripts/ci/merge-main.sh
#
# A pull request is gated on the merge of its head into main
# (refs/pull/N/merge, whose first parent is main), and the translator's
# ratchet (UE-18) compares a commit with its first parent. So what a writer
# pushes to a pull request's branch is that merge, committed, with the
# writers' outputs in it (scripts/ci/push-branch.sh commits it): its first
# parent is main, its second the branch's head, and pushing it to the branch
# is a fast-forward. The gate dispatched on it then reads the generated
# files of the very tree it gates and compares the ratchet with main, as the
# pull request's own gate would.
#
# A conflict inside a file only the writers write (below) is taken as main
# has it, the whole unit at once, since the writers run next and rewrite it;
# any other conflict refuses, naming the files: a person merges main into the
# branch. A merge that would bring main's workflow files onto the branch
# refuses too: the workflow token cannot push a change to a workflow file.
# Exit status: 0 merged (the state is in $(git rev-parse --git-dir)/branch-merge,
# "MAIN HEAD"), 2 when the branch holds nothing main lacks, 1 on a refusal.
set -euo pipefail
branch=${BRANCH:?BRANCH is required}
: "${GH_TOKEN:?GH_TOKEN is required}"
if [[ $branch == main ]] || ! git check-ref-format --branch "$branch" >/dev/null; then
  printf 'merge-main: not a pull request branch: %s\n' "$branch" >&2
  exit 1
fi
root=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root"
auth=$(printf 'x-access-token:%s' "$GH_TOKEN" | base64 -w0)
remote="https://github.com/${GITHUB_REPOSITORY:?}.git"
git -c "http.extraheader=AUTHORIZATION: basic $auth" fetch -q "$remote" \
  "+refs/heads/main:refs/remotes/upstream/main" "+refs/heads/$branch:refs/remotes/upstream/branch"
main=$(git rev-parse upstream/main)
head=$(git rev-parse upstream/branch)
state="$(git rev-parse --git-dir)/branch-merge"
rm -f "$state"
git merge --abort 2>/dev/null || true
git reset -q --hard
git checkout -q --detach "$main"
if git merge-base --is-ancestor "$head" "$main"; then
  echo "merge-main: $branch ($head) holds nothing main ($main) lacks"
  exit 2
fi
# The writers' units: every file in each is written by a writer of `just
# generate` or `just certified`, so main's version is as good a start as
# the branch's.
units=(
  frontier.toml
  lift-verdicts.toml
  CONFORMANCE.md
  crates/uorust/tests/fixtures/mir-path/lift-verdicts.toml
  crates/uorust/tests/fixtures/thermostat-prism
  "crates/certified crates/uorust/src/certified.rs"
  "app/src app/lexlean.lock"
)
git -c user.name='github-actions[bot]' -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
  merge -q --no-ff --no-commit "$head" >/dev/null 2>&1 || true
if [[ ! -f $(git rev-parse --git-dir)/MERGE_HEAD ]]; then
  echo "merge-main: git did not start the merge of $head onto $main" >&2
  exit 1
fi
taken=()
for unit in "${units[@]}"; do
  read -r -a members <<<"$unit"
  if [[ -n $(git diff --name-only --diff-filter=U -- "${members[@]}") ]]; then
    git rm -r -q --cached --ignore-unmatch -- "${members[@]}"
    rm -rf -- "${members[@]}"
    for member in "${members[@]}"; do
      if git cat-file -e "$main:$member" 2>/dev/null; then
        git checkout -q "$main" -- "$member"
      fi
    done
    taken+=("$unit")
  fi
done
if ((${#taken[@]})); then
  echo "merge-main: taken as main has them, for the writers to rewrite: ${taken[*]}"
fi
conflicts=$(git diff --name-only --diff-filter=U)
if [[ -n $conflicts ]]; then
  git merge --abort
  printf '::error::%s conflicts with main outside the generated files; merge main into it by hand:\n%s\n' "$branch" "$conflicts" >&2
  exit 1
fi
workflows=$(git diff --cached --name-only "$head" -- .github/workflows)
if [[ -n $workflows ]]; then
  git merge --abort
  printf '::error::main changed workflow files %s lacks, which the workflow token cannot push; merge main into it by hand:\n%s\n' "$branch" "$workflows" >&2
  exit 1
fi
printf '%s %s\n' "$main" "$head" >"$state"
echo "merge-main: $branch at $head merged onto main at $main, not committed"
