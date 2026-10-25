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
             \x20 list [tasks|issues|concerns] [--all]\n\
             \x20 hook session-start|pre-tool-use|post-tool-use   (Claude Code hook JSON on stdin)\n\
             actor: $META_ACTOR, else $USER",
        )
    }
}
