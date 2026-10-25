#!/bin/sh
set -eu

remote="${META_REMOTE:-origin}"
here=$(dirname "$0")

git fetch -q "$remote" "+refs/heads/meta/*:refs/remotes/$remote/meta/*"
old=$(git rev-parse --verify "refs/remotes/$remote/meta/log")
if git rev-parse --verify --quiet "refs/remotes/$remote/meta/log-v0" >/dev/null; then
  echo "meta/log-v0 exists: already archived" >&2
  exit 1
fi
git push -q "$remote" "$old:refs/heads/meta/log-v0"

empty=$(git hash-object -w --stdin </dev/null)
tree=$(printf '100644 blob %s\tevents.jsonl\n' "$empty" | git mktree)
start=$(GIT_AUTHOR_NAME=reseed GIT_AUTHOR_EMAIL=reseed@meta.invalid \
  GIT_COMMITTER_NAME=reseed GIT_COMMITTER_EMAIL=reseed@meta.invalid \
  git commit-tree "$tree" -p "$old" -m "reseed: meta/log restarts as events.jsonl; previous log archived as meta/log-v0")
git push -q "$remote" "$start:refs/heads/meta/log"

sh "$here/import.sh"
sh "$here/carried-over.sh"
