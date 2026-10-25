# Bootstrap plan

Each stage is used on meta-development itself before the next begins (principle 4). Each has an exit test.
Until stage 0 exists, this file is the tracker; stage 0's first job is to import it and the postmortem as events.

| Stage | Build | Exit test | Status |
|---|---|---|---|
| 0 | `meta/log` branch; minimal schema (Ids; ConcernIdentified, TaskOpened/Claimed/Done, IssueReported); `meta` add/list/claim/done; hooks: SessionStart state, block merge/force-push/push-to-main, auto-push on commit. Seed with the postmortem's concerns. | One week of meta-development's own work run from the log, no hand-written handoff notes | in progress |
| 1 | Runs + timing for every check and long command; wall-time budgets that fail; flaky reruns recorded + flakiness budget; complexity and vagueness budgets as counts | A budget has actually blocked something | not started |
| 2 | `meta run` trail recorder; first procedure (Playwright lookup); researcher/tester roles | Two agents run the same procedure and get comparable, inspectable trails | not started |
| 3 | Typed subjects/properties/claims; evidence kinds; ladder typestate; severity budget; done needs evidence; evidence check in CI | meta-development's own features show computed status and confidence | not started |
| 4 | Affected-test selection; lints moved out of Basho; project templates | not yet defined | not started |
| 5 | (moved) Basho onboards as soon as possible: right after stage 0/1 (owner, 2026-10-25) | — | — |
| later | Confidence procedure; proptest invariants; waivers; perf ledger; hosted service when the git backend's budget is overspent | — | — |

Prerequisite (owner): cloud CI (fix GitHub billing or choose another runner) by stage 1.
Decided (owner, 2026-10-25): Basho onboards as soon as possible, right after stage 0/1.

## Basho as the test subject (owner, 2026-10-25)
- Basho is the test subject that confirms meta-development works as advertised.
- Acceptance: each problem in docs/basho-postmortem.md becomes a claim of the form "under meta-development, this is caught or prevented", with evidence gathered from Basho's real use. Examples:
  - a blocking PR is visible and budgeted;
  - CI wall time over budget fails;
  - a flaky check is recorded and quarantined;
  - a train leg without a platform can't be represented;
  - an agent can't finish without a tracker update.
- Isolation still holds (principle 5): Basho adopts meta-development only through its own config. meta-development contains nothing Basho-specific.
