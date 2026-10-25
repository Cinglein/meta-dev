#!/bin/sh
set -eu

meta="${META_BIN:-meta}"
export META_ACTOR=seed

PM="context/basho-postmortem.md"

concern() {
  id=$("$meta" concern "$3" --source "$PM $2" --tag acceptance-candidate | awk '{print $5}')
  eval "$1=\$id"
}

concern dev_ci        "§3.1"   "CI and test speed was a huge blocker; the suite was huge, bloated and unoptimized"
concern dev_typelevel "§3.1"   "Much of the test suite enforced what should have been compile-time or type-level facts"
concern dev_fullsuite "§3.1"   "The full suite ran on unrelated changes; a small data change forced a full rebuild and recheck"
concern dev_coupled   "§3.1"   "App code and trip data were not separated (not a general travel app taking data files)"
concern dev_affected  "§3.1"   "Nobody knew which code affected which features, so everything was rechecked and CI ran out"
concern dev_serial    "§3.1"   "Development was serial: the current feature blocked unrelated work (worst with #72)"
concern dev_triage    "§3.1"   "Criticality was haphazard and miscalibrated in both directions, with no triage across the spec"
concern dev_features  "§3.1"   "Tracking implemented versus unimplemented features was a mess"
concern dev_cache     "§3.1 1" "Worktrees did not share a compiler cache or build directory"
concern dev_sims      "§3.1 2" "Simulators were shared instead of one per agent or test group"
concern dev_flaky     "§3.1 3" "Flaky tests were retried and forgotten instead of flagged and fixed"
concern dev_uicheck   "§3.1 4" "The UI check used a browser for facts that need no layout, and for untouched screens"
concern dev_ids       "§3.1 5" "Data was keyed by leg numbers, not stable ids; generated files were large and coarse"
concern dev_wait      "§3.1 7" "Most time was spent waiting for CI and tests"
concern dev_relay     "§3.2"   "The main session relayed every hand-off while agents sat idle"
concern dev_truth     "§3.2"   "No single source of truth: status files, a handoff note and the main session's context"
concern dev_collide   "§3.2"   "Agents collided on simulators, Chrome, search quota and data files"
concern dev_owner     "§3.2"   "Ownership of work was unclear"
concern dev_lost      "§3.2"   "Work died with the session or the laptop"
concern dev_green     "§3.3 2" "Exists plus green CI was taken to mean correct"
concern dev_dataver   "§3.3 2" "How data was verified (simulator, user, online sources) was tribal knowledge, not typed"
concern dev_spec      "§3.3 2" "The spec was not encoded; the code was its own definition of done"
concern dev_manual    "§3.3 5" "Manual steps (simulator runs, Playwright lookups) were never proceduralized and left no trail"
concern dev_vacuous   "§3.3"   "Tests that cannot fail were counted as evidence"
concern dev_perf      "§3.3"   "Performance profiles and optimisation effort were not recorded"
concern dev_pushrule  "§3.4"   "Push after every commit was a written rule; the overnight fixer pushed only once at the end"

"$meta" concern "The owner reviews and merges every PR, a bottleneck that worsens as agents parallelise; goal: most PRs merged sight-unseen, with merge authority earned per risk tier" \
  --source "context/SPEC.md Goal: sight-unseen merges" >/dev/null
"$meta" concern "The owner does triage, priority and wall-time watching by hand; goal: a steward role reads the log, flags budget trends and blocking work, escalates only decisions" \
  --source "context/SPEC.md Goal: automate the owner's triage/priority/wall-time role" >/dev/null

task() {
  "$meta" open "$@" | awk '{print $5}'
}
stage0=$(task "Stage 0: meta/log, minimal schema, meta CLI, hooks, seed. Exit test: one week of meta-development's own work run from the log, no hand-written handoff notes" \
  --addresses "$dev_truth" --addresses "$dev_pushrule" --addresses "$dev_lost" --addresses "$dev_owner")
task "Prerequisite for stage 1: cloud CI (fix GitHub billing or choose another runner)" --addresses "$dev_wait" >/dev/null
task "Stage 1: runs and timing for every check; failing wall-time budgets; flaky reruns recorded with a flakiness budget; complexity and vagueness budgets as counts. Exit test: a budget has actually blocked something" \
  --addresses "$dev_ci" --addresses "$dev_flaky" >/dev/null
task "Onboard Basho as the test subject right after stage 0/1, through its own meta.toml only" >/dev/null
task "Stage 2: meta run trail recorder; first procedure (Playwright lookup); researcher and tester roles. Exit test: two agents run the same procedure and get comparable, inspectable trails" \
  --addresses "$dev_manual" >/dev/null
task "Stage 3: typed subjects, properties, claims; evidence kinds; ladder typestate; severity budget; done needs evidence; evidence check in CI. Exit test: meta-development's own features show computed status and confidence" \
  --addresses "$dev_features" --addresses "$dev_triage" --addresses "$dev_green" --addresses "$dev_spec" --addresses "$dev_vacuous" >/dev/null
task "Stage 4: affected-test selection; lints moved out of Basho; project templates. Exit test: not yet defined" \
  --addresses "$dev_fullsuite" --addresses "$dev_affected" >/dev/null
task "Later: confidence procedure; proptest invariants; waivers; perf ledger; hosted log service when the git backend's budget is overspent" >/dev/null

echo "$stage0"
