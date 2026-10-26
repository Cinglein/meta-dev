# Superset check: meta-development tooling on chess

Task: task-d3fa995600f58f8f. Checked 2026-10-26 against chess `main` at 0868628 and
meta-development `main` at 317d586 plus this PR.

## How to run it on another workspace

Build xtask here, then run the binary with the other workspace as the current directory:
`cargo build -p xtask` and, from the other repository, `<this checkout>/target/debug/xtask lint`.
The workspace is the first directory above the current one whose `Cargo.toml` has `[workspace]`;
the policy is that workspace's `tooling/policy.toml`. Before this PR both were fixed at compile
time to meta-development itself.

## Policy delta

chess's `tooling/policy.toml` does not parse as is: meta-development's `Policy` also needs
`bit_boundary` (chess keeps it in code, `tooling/task/src/bit_boundary.rs`; copied verbatim),
`baseline`, `surfaces`, `[branches]`, `[mutation]`, `limits.scc_items`, `floors.cut_side` and
`shares.cut_conductance`. The last three feed only the new crate-graph lint; they took
meta-development's values. Every shared limit kept chess's value.

## Per-lint comparison

Two runs. Plain: chess `main` unchanged. Seeded: every chess corpus case copied into
`crates/eval/src/seeded/`, one numeric literal added to the `chess` binary, one unused re-export
added to `eval`, and every limit, floor, share and the vague list tightened the same way in both
policies. Each per-lint task ran with each tool; "match" means the printed findings are the same
set, line for line.

| Lint | Plain chess / meta | Seeded chess | Seeded meta | Match |
|---|---|---|---|---|
| const-shape | 0 / 0 | 9 | 9 | yes |
| distinct-signatures | 0 / 0 | 4 | 4 | yes |
| expression-flow | 0 / 0 | 42 | 42 | yes |
| fn-shape | 0 / 0 | 329 | 329 | yes |
| literal-names | 0 / 0 | 285 | 285 | yes |
| manual-iteration | 0 / 0 | 4 | 4 | yes |
| module-nesting | 0 / 0 | 19 | 19 | yes |
| named-lifetimes | 0 / 0 | 85 | 85 | yes |
| named-numbers | 0 / 0 | 12 | 12 | yes |
| no-comments | 0 / 0 | 102 | 102 | yes |
| no-forwarders | 0 / 0 | 14 | 14 | yes |
| no-free-fns | 0 / 0 | 8 | 8 | yes |
| no-numbers-in-binaries | 0 / 0 | 1 | 1 | yes |
| no-parameter-bags | 0 / 0 | 4 | 4 | yes |
| primitive-boundary | 0 / 0 | 24 | 24 | yes |
| private-fns | 0 / 0 | 43 | 43 | yes |
| public-surface | 0 / 0 | 1 | 1 | yes |
| settings-shape | 0 / 0 | 2 | 2 | yes |
| state-graph | 0 / 0 | 15 | 15 | yes |
| test-budget | 0 / 0 | 258 | 258 | yes |
| type-shape | 0 / 0 | 6 | 6 | yes |

No chess finding is missed. The state-graph and test-budget summary lines also match.

## New findings chess would get

27 on plain chess `main`: 15 under `crates/`, 12 under `tooling/` (which the switch deletes).

- crate-graph (5): `crates/arena`, `crates/datagen`, `crates/trainer` (conductance 0.000: the
  binary's settings and command line form one group, the library another), `crates/uci`
  (0.071: the `go` clock types split from the rest), `tooling/task`.
- enum-predicates (15): `arena` `Termination::defeats_the_side_to_move`, `Thinking::forfeits`;
  `board` `Square::aligned_by`; `uci` `Command::interrupts`, `Moves::is_empty`,
  `Response::is_best_move`; 9 under `tooling/`.
- vec-out-params (2): `tooling/lints/primitive_boundary/.../sighting.rs` `Sighting::filed`, one
  per `&mut Vec` parameter.
- thin-main (5): the `mod` lines of `crates/arena`, `crates/chess` (2), `crates/datagen` and
  `crates/trainer` `main.rs`. chess's rules let a binary keep a `command_line` module beside
  `main.rs`; thin-main wants it in the library.

## Non-lint tasks

- affected: chess `affected --affected` and meta `affected` select the same crates for a change
  in `fen`, `search`, `tui`, a lint crate, `tooling/task` and a `.md` file. Difference by design:
  chess prints one plan per job, and an `xtask/src/main.rs` change puts every crate in chess's
  `full` mutation job; meta selects only `xtask`.
- test-time (`--affected`, change in `search`): both ran the same 49 tests, about 0.95 s, the
  same five slowest, both inside the 1 s and 4 s budgets.
- mutation plan: not compared. Both plans list mutants through cargo-mutants, which is not run
  locally for this task. Compare in CI when chess switches.
