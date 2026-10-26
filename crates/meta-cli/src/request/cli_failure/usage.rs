use std::fmt;

#[derive(Debug)]
pub struct Usage;

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "usage: meta <command> [options] [--] [words]\n\
             \x20 concern <text> --division <path> [--source <text>] [--tag acceptance-candidate]\n\
             \x20 issue <text> --division <path>\n\
             \x20 open <text> --division <path> [--addresses <concern-id|issue-id>]... [--effort <tokens>] [--due <rfc3339>]\n\
             \x20 claim <task-id> [--from <actor>] [--worktree]   (assigns you; several may hold a task; --worktree: new worktree on branch <task-id>, prints its path)\n\
             \x20 worktrees   (this clone's worktrees: task status, missing, unrecorded)\n\
             \x20 gc [--dry-run]   (frees recorded worktrees whose pr is merged or closed or whose task stalled, once all work is on the remote and nothing has them open)\n\
             \x20 release <task-id>   (unassigns you)\n\
             \x20 done|reopen <task-id>\n\
             \x20 wait <task-id> --on <issue-id|task-id|review|ci|merge> | resume <task-id>\n\
             \x20 link <task-id> <pr-number>\n\
             \x20 retire <id> <reason>   (won't do; keeps the entry)\n\
             \x20 correct <id> <text>   (replaces the entry's text)\n\
             \x20 list [tasks|issues|concerns] [--all]\n\
             \x20 status [--flags]   (divisions down to tasks, live PR and CI; --flags: flags only, exit 1 if any)\n\
             \x20 hook session-start|pre-tool-use|post-tool-use|stop   (Claude Code hook JSON on stdin)\n\
             \x20 ci-wait | ci-log   (this session's CI: wait for failure or completion | failed step output)\n\
             actor: $META_ACTOR, else owner",
        )
    }
}
