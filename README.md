# meta-development

meta-development tracks, budgets and checks the development work of a Rust workspace.
It gives lints, budgets and CI tasks through one crate: `meta-xtask`.
Your project keeps all of its limits in its own file: `tooling/policy.toml`.

## Use the tooling in your workspace

Do these steps at the root of your workspace.

1. Add an `xtask` crate to `members` in `Cargo.toml`.

2. Add the dependency to `xtask/Cargo.toml`. Use a tag. Do not use a branch.
   The tag `v0.1.0` is available only after the owner makes it. Until then, use `rev = "<commit>"`.

   ```toml
   [package]
   name = "xtask"
   version = "0.0.0"
   edition = "2024"
   publish = false

   [dependencies]
   meta-xtask = { git = "https://github.com/Cinglein/meta-dev", tag = "v0.1.0" }
   ```

3. Write `xtask/src/main.rs`:

   ```rust
   use std::env;
   use std::process::ExitCode;

   use meta_xtask::Task;

   fn main() -> ExitCode {
       let arguments: Vec<String> = env::args().skip(1).collect();
       let task = arguments
           .first()
           .ok_or_else(Task::usage)
           .and_then(|argument| argument.parse::<Task>().map_err(|_| Task::usage()));
       match task.and_then(|task| task.run(arguments.into_iter().skip(1))) {
           Ok(()) => ExitCode::SUCCESS,
           Err(failure) => {
               eprintln!("{failure}");
               ExitCode::FAILURE
           }
       }
   }
   ```

4. Add the alias to `.cargo/config.toml`:

   ```toml
   [alias]
   xtask = "run --quiet --package xtask --"
   ```

5. Write `tooling/policy.toml`.
   Copy `examples/consumer/tooling/policy.toml` from this repository.
   Then change the values for your project.
   The file must have all of the keys. If a key is missing or unknown, the task stops and shows the error.
   Put the baseline file at the path that `baseline` gives. An empty file is correct.

6. Make sure that git has the base branch.
   The `[branches]` table in your policy gives it, for example `base = "origin/main"`.
   Your repository must have an `origin` remote. Run `git fetch origin` before the checks.
   The checks compare your work with the merge base of `HEAD` and that branch.
   Your workspace can be in a subdirectory of the git repository.

7. Run the checks:

   ```sh
   cargo xtask ci
   ```

   The command runs fmt, clippy, the tests, the test-time budget and the lints.
   It reads your `tooling/policy.toml` only. It does not read the policy of meta-development.

`examples/consumer/` is a full example. CI makes a new lockfile for it and runs `cargo xtask ci` in it.

Your lockfile gets `unicode-ident 1.0.24`. meta-development sets this exact version.
Newer versions stop `ra-ap-rustc_lexer` from compiling. Do not update `unicode-ident` until meta-development removes this limit.

## Versions

- All crates have the same version. The version is in `[workspace.package]` in `Cargo.toml`.
- Each release has a tag `vX.Y.Z` that agrees with that version.
- Before 1.0.0, a change to `Y` can break your policy file or the tasks. Read the commits between the two tags before you change the tag.
- There is no changelog. The git history between two tags is the record of the changes.
