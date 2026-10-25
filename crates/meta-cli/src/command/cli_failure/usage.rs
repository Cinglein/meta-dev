use std::fmt;

#[derive(Debug)]
pub struct Usage;

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "usage: meta <command> [options] [--] [words]\n\
             \x20 concern <text> [--source <text>] [--tag acceptance-candidate]\n\
             \x20 issue <text>\n\
             \x20 open <text> [--addresses <concern-id|issue-id>]...\n\
             \x20 claim <task-id> [--from <actor>]\n\
             \x20 release|done|reopen <task-id>\n\
             \x20 retire <id> <reason>   (won't do; keeps the entry)\n\
             \x20 correct <id> <text>   (replaces the entry's text)\n\
             \x20 list [tasks|issues|concerns] [--all]\n\
             \x20 hook session-start|pre-tool-use|post-tool-use|stop   (Claude Code hook JSON on stdin)\n\
             \x20 ci-wait | ci-log   (this session's CI: wait for failure or completion | failed step output)\n\
             actor: $META_ACTOR, else $USER",
        )
    }
}
