use std::fmt;

#[derive(Debug)]
pub struct Usage;

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "usage: meta <command> [options] [--] [words]\n\
             \x20 concern <text> --division <path> [--source <text>] [--tag acceptance-candidate] [--severity <level>]\n\
             \x20 issue <text> --division <path> [--severity <level>]\n\
             \x20 rate <concern-id|issue-id> <level> [--scenario <text>]   (levels: the division's scale in tooling/policy.toml; raising needs the reproduction or failure scenario)\n\
             \x20 move <concern-id|issue-id> --division <path> [--severity <level> [--scenario <text>]]   (division: a top division or one in tooling/policy.toml; levels compare where they roll up: the level is kept only if it rolls up the same, else give --severity; a lower roll-up is free, a higher one needs --scenario)\n\
             \x20 open <text> --division <path> [--addresses <concern-id|issue-id>]... [--effort <tokens>] [--due <rfc3339>]\n\
             \x20 claim <task-id> [--from <actor>] [--worktree]   (assigns you; several may hold a task; --worktree: new worktree on branch <task-id>, prints its path)\n\
             \x20 worktrees   (this clone's worktrees: task status, missing, unrecorded)\n\
             \x20 gc [--dry-run]   (frees recorded worktrees whose pr is merged or closed or whose task stalled, once all work is on the remote and nothing has them open)\n\
             \x20 gc restore <quarantine>   (puts a stranded quarantine's file modes back, unlocks it and moves it home)\n\
             \x20 release <task-id>   (unassigns you)\n\
             \x20 done|reopen <task-id>\n\
             \x20 wait <task-id> --on <issue-id|task-id|review|ci|merge> | resume <task-id>\n\
             \x20 link <task-id> <pr-number>\n\
             \x20 retire <id> <reason>   (won't do; keeps the entry)\n\
             \x20 correct <id> <text>   (replaces the entry's text)\n\
             \x20 list [tasks|issues|concerns] [--all]\n\
             \x20 status [--flags]   (divisions down to tasks, live PR and CI; --flags: flags only, exit 1 if any)\n\
             \x20 heartbeat tick|flags|plist   (tick: one timed run of status, writes the local snapshot, prints new flags, exit 1 if any needs judgement | flags: latest snapshot and age, stale is a flag | plist: launchd job for this clone, printed only)\n\
             \x20 monitor tick|report   (tick: reports this project's processes, obvious and watch tiers, into the local record, exit 1 if anything is obvious; stops nothing unless meta.local.toml [monitor] stop lists a class; also runs on heartbeat tick | report: the latest report)\n\
             \x20 trace last|chrome [<command>]   (latest recorded run, or the latest of <command>: last: critical path by span kind and untraced time | chrome: Chrome trace JSON; every run records to .git/meta-local/traces, META_TRACE=off skips it, meta.local.toml [trace] files and bytes cap what stays)\n\
             \x20 hook session-start|pre-tool-use|post-tool-use|stop   (Claude Code hook JSON on stdin)\n\
             \x20 ci-wait | ci-log   (this session's CI: wait for failure or completion | failed step output)\n\
             actor: $META_ACTOR, else owner",
        )
    }
}
