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
# the branch moved past the head the writers ran on: that push started its
# own run. The push is a fast-forward and never forced. A push made with the
# workflow token starts no workflow, so the gate is dispatched for it here.
# It prints "push: pushed <sha>" when it pushed.
set -euo pipefail
subject=$1
read -r -a paths <<<"$2"
shift 2
branch=${BRANCH:?BRANCH is required}
: "${GH_TOKEN:?GH_TOKEN is required}"
if [[ $branch == main ]] || ! git check-ref-format --branch "$branch" >/dev/null; then
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
now=$(git -c "http.extraheader=AUTHORIZATION: basic $auth" ls-remote "$remote" "refs/heads/$branch" | cut -f1)
if [[ $now != "$head" ]]; then
  echo "push: $branch moved from $head to $now; its own run regenerates"
  exit 0
fi
if ! git -c "http.extraheader=AUTHORIZATION: basic $auth" push "$remote" "HEAD:refs/heads/$branch"; then
  now=$(git -c "http.extraheader=AUTHORIZATION: basic $auth" ls-remote "$remote" "refs/heads/$branch" | cut -f1)
  if [[ $now != "$head" ]]; then
    echo "push: $branch moved while pushing; its own run regenerates"
    exit 0
  fi
  # The branch stands where it was, so the forge refused the push itself
  # (a protected branch, a token without contents: write); git printed why.
  echo "::error::the push to $branch was refused with the branch unmoved" >&2
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
