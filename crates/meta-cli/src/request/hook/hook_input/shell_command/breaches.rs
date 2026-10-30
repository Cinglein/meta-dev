use super::ShellCommand;
use crate::request::scale_policy::ScalePolicy;

const GIT_ROWS: [(&str, bool); 12] = [
    ("git stash", true),
    ("git stash pop", true),
    ("git -C x stash", true),
    ("cd a && git stash drop", true),
    ("git push --force origin f", true),
    ("git push -f origin f", true),
    ("git push -uf origin f", true),
    ("git push --force-with-lease origin f", true),
    ("git push origin f --force-with-lease=f", true),
    ("git rm x && git mv a b", false),
    ("git push -u origin f", false),
    ("git commit -m 'git stash'", false),
];

const SHELL_ROWS: [(&str, bool); 56] = [
    ("rm -rf x", true),
    ("cd a && mv b c", true),
    ("sed -i s/a/b/ f", true),
    ("git diff | sed -n 1p", true),
    ("awk '{print}' f", true),
    ("python3 -c 1", true),
    ("nice -n 19 perl -e 1", true),
    ("FOO=1 /usr/bin/sed p", true),
    ("cat <<EOF\nx\nEOF", true),
    ("cat > f", true),
    ("echo x >> f", true),
    ("ls 2>/dev/null", true),
    ("ls &>f", true),
    ("git commit -m 'remove sed > rm'", false),
    ("git commit -m \"rm and sed\" && git rm x", false),
    ("ls crates/rm/src/sed.rs | grep rm", false),
    ("cargo build 2>&1 | tail -3", false),
    ("cat f < in; grep x <<< y", false),
    ("echo a\\>b; FOO=rm env ls", false),
    ("git diff | wc -l", false),
    ("timeout 5s sed p f", true),
    ("timeout -k 1 5 nice -n 2 awk 1", true),
    ("cat <<-'EOF'\nx\nEOF", true),
    ("echo $((1 << 2))", false),
    ("true; xargs ls", false),
    ("git commit -m \"fix; rm stuff\"", false),
    ("echo \"a | sed\" && git commit -m 'fix (sed) bug'", false),
    ("git commit -m \"title\n\nrm the thing\nsed it\npython x\"", false),
    ("command -v sed; echo a # > b", false),
    ("diff <(ls) <(ls -a); tee >(cat)", false),
    ("bash -c 'sed -i p f'", true),
    ("sh -c \"cd a && rm x\"", true),
    ("eval rm x", true),
    ("xargs -n 1 sh -lc 'mv a b'", true),
    ("echo $(sed p f)", true),
    ("echo `rm x`", true),
    ("echo \"$(rm x)\"", true),
    ("echo \"$(echo a > f)\"", true),
    ("env -u X sed p", true),
    ("sudo -u X rm x", true),
    ("timeout -s KILL 5 sed p", true),
    ("timeout --signal KILL -k 1 5 sed p", true),
    ("cd a; (sed p f)", true),
    ("sudo -n rm x", true),
    ("sudo -E rm x", true),
    ("sudo -S rm x", true),
    ("sudo -k rm x", true),
    ("if rm x; then :; fi", true),
    ("while sed p f; do :; done", true),
    ("echo rm x | bash", true),
    ("bash <<< 'rm x'", true),
    ("bash script.sh; source f", false),
    ("gh pr create --body \"Fix (a; b) > c\n\n- rm x; sed\"", false),
    ("cargo test -- --nocapture 2>&1 | tail", false),
    ("jq '.[] | select(.a > 1)' f", false),
    ("RUSTFLAGS=\"-D warnings\" nice -n 19 cargo clippy", false),
];

#[test]
fn a_shell_command_running_a_forbidden_program_git_use_heredoc_or_file_redirect_is_refused_and_nothing_else() {
    let rules = ScalePolicy::shell_in(None).unwrap();
    let refused = |command: &str| command.parse::<ShellCommand>().unwrap().breach(&rules).is_some();
    assert!(SHELL_ROWS.iter().all(|(command, verdict)| refused(command) == *verdict));
    assert!(GIT_ROWS.iter().all(|(command, verdict)| refused(command) == *verdict));
}
