#!/usr/bin/env bash
# Commit what the writers changed on a pull request's branch and push it to
# that branch, never to main, then dispatch the named workflows on it.
#
#     BRANCH=<head branch> scripts/ci/push-branch.sh "<commit subject>" "<path> ..." [workflow.yml ...]
#
# The checkout is the uncommitted merge scripts/ci/merge-main.sh left: the
# commit made here is that merge (first parent main, second the branch's
# head) with the writers' outputs under the named paths, so the gate on it
# compares with main, as the pull request's own gate does. Nothing is
# pushed when the merge's tree is the head's own and the head already holds
# main: the pull request's own gate reads that tree. Nothing is pushed when
# the branch is no longer the head the writers ran on (moved on, or reset by
# a person): that push started its own run. The push is a fast-forward of
# that head, and the forge checks the branch is still that head as it
# updates it (--force-with-lease names the head), so a branch reset to an
# ancestor meanwhile is refused, never fast-forwarded back. A push made with
# the workflow token starts no workflow, so the gate is dispatched for it
# here.
# Exit status: 0 and "push: pushed <sha>" when it pushed, 0 and "push:
# nothing changed ..." when nothing was to push, 3 and "push: <branch> moved
# ..." when the branch moved, 1 on a refusal.
set -euo pipefail
subject=$1
read -r -a paths <<<"$2"
shift 2
branch=${BRANCH:?BRANCH is required}
: "${GH_TOKEN:?GH_TOKEN is required}"
if [[ $branch == main || $branch == refs/* ]] || ! git check-ref-format --branch "$branch" >/dev/null; then
  printf 'push: not a pull request branch: %s\n' "$branch" >&2
  exit 1
fi
state="$(git rev-parse --git-dir)/branch-merge"
if [[ ! -f $state || ! -f $(git rev-parse --git-dir)/MERGE_HEAD ]]; then
  echo "push: no merge of $branch onto main is in progress (scripts/ci/merge-main.sh)" >&2
  exit 1
fi
read -r main head <"$state"
if [[ $(git rev-parse HEAD) != "$main" || $(git rev-parse MERGE_HEAD) != "$head" ]]; then
  echo "push: the checkout is not the merge merge-main.sh recorded" >&2
  exit 1
fi
git add -- "${paths[@]}"
if [[ $(git write-tree) == $(git rev-parse "$head^{tree}") ]] && git merge-base --is-ancestor "$main" "$head"; then
  echo "push: nothing changed under ${paths[*]} on $branch"
  exit 0
fi
git diff --cached --stat "$head"
git -c user.name='github-actions[bot]' \
  -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
  commit -q -m "$subject" -m "Written by the ${GITHUB_WORKFLOW:-local} workflow on $branch at ${head}, merged onto main at ${main}."
auth=$(printf 'x-access-token:%s' "$GH_TOKEN" | base64 -w0)
remote="https://github.com/${GITHUB_REPOSITORY:?}.git"
# The branch's tip on the forge: the exact ref, never a longer one ending in it.
tip() {
  git -c "http.extraheader=AUTHORIZATION: basic $auth" ls-remote "$remote" "refs/heads/$branch" |
    awk -v ref="refs/heads/$branch" '$2 == ref { print $1 }'
}
if ! git -c "http.extraheader=AUTHORIZATION: basic $auth" push --force-with-lease="refs/heads/$branch:$head" \
  "$remote" "HEAD:refs/heads/$branch"; then
  now=$(tip)
  if [[ $now != "$head" ]]; then
    echo "push: $branch moved from $head to ${now:-nothing}; its own run regenerates"
    exit 3
  fi
  # The branch stands where it was, so the forge refused the push itself
  # (a protected branch, a token without contents: write, or a workflow file
  # the token may not push); git printed why.
  echo "::error::the push to $branch was refused with the branch unmoved" >&2
  if [[ -n $(git diff --name-only "$main" HEAD -- .github/workflows) ]]; then
    echo "::error::$branch itself changes workflow files, which this merge carries against main, its first parent: if git printed 'without \`workflows\` permission', the forge judges a merge by its first parent, and the workflow changes must reach main first" >&2
  fi
  exit 1
fi
rm -f "$state"
for workflow in "$@"; do
  gh workflow run "$workflow" --repo "$GITHUB_REPOSITORY" --ref "$branch"
done
echo "push: pushed $(git rev-parse HEAD)"
{
  echo "## $subject"
  echo
  echo "Pushed \`$(git rev-parse HEAD)\` to $branch (main at \`$main\` merged with \`$head\`) and dispatched on it: $*"
} >>"${GITHUB_STEP_SUMMARY:-/dev/null}"
