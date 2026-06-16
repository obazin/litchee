---
description: Triage open Dependabot PRs — merge the green ones, fix or close the failing ones
argument-hint: "[pr-number]  (optional: triage only that PR)"
allowed-tools:
  - Bash(gh pr list:*)
  - Bash(gh pr view:*)
  - Bash(gh pr diff:*)
  - Bash(gh pr checks:*)
  - Bash(gh pr merge:*)
  - Bash(gh pr close:*)
  - Bash(gh run list:*)
  - Bash(gh run view:*)
  - Bash(gh run watch:*)
  - Bash(gh api:*)
  - Bash(git pull:*)
  - Bash(git submodule:*)
  - Bash(git fetch:*)
  - Bash(git add:*)
  - Bash(git commit:*)
  - Bash(git push:*)
  - Bash(git log:*)
  - Bash(git diff:*)
  - Bash(cargo build:*)
  - Bash(cargo test:*)
  - Bash(cargo clippy:*)
  - Bash(cargo fmt:*)
  - Bash(nix develop:*)
  - Edit
  - Read
  - Agent
---

You are triaging **Dependabot pull requests** for this repository and acting on
them. Obey the project's `CLAUDE.md` at all times — most relevantly:

- **Code review before every commit.** Run a code-review agent on the diff,
  address findings, *then* commit. Never commit unreviewed work.
- **Conventional commit prefixes** (`feat:`/`fix:`/`chore:`/`refacto:`/`test:`/`docs:`).
- **No `Co-Authored-By` / AI-attribution trailer** on commits.
- **Run cargo through the Nix dev shell**: `nix develop --command bash -c "..."`.
- When in doubt about a crate API, **consult context7** — don't guess the
  migration.
- Commit **atomically**, one coherent change per commit.

## Scope

Argument: `$ARGUMENTS`
- If a PR number is given, triage **only** that PR.
- If empty, triage **every** open PR authored by Dependabot.

## Current state (auto-collected)

- Open PRs: !`gh pr list --state open --json number,title,author,headRefName --jq '.[] | "#\(.number) [\(.author.login)] \(.title)"' 2>/dev/null || echo "(unable to list)"`

## Procedure

For each in-scope Dependabot PR:

1. **Gather status.** Read its title/diff and check results:
   `gh pr view <n> --json title,mergeable,mergeStateStatus` and
   `gh pr checks <n>`. Note whether every required check passes and whether the
   PR is mergeable against current `main` (it may be behind after earlier merges).

2. **Arbitrate by outcome:**

   - **All checks green + mergeable →** squash-merge and tidy up:
     `gh pr merge <n> --squash --delete-branch`. Prefer squash to keep history
     linear and preserve the conventional `chore: bump …` message.

   - **A check is failing →** find the *root cause* before acting. Pull the
     failed job log (`gh run view <run-id> --log-failed`) and decide which case
     it is:

     - **Misfired bump** — Dependabot changed something it shouldn't (classic
       example: it treats `dtolnay/rust-toolchain@<msrv>` as an action version
       and "bumps" the tag, which is actually our **MSRV channel**, breaking the
       toolchain install). Don't merge. Instead **add an `ignore` entry** to
       `.github/dependabot.yml` for that dependency, commit it
       (`fix:`/`chore:` + explanation), then **close the PR** with a comment
       pointing at the ignore rule so it isn't reopened.

     - **Genuine breaking change** — the new version needs code changes
       Dependabot can't make (e.g. a renamed module or function). Do the
       migration **locally on `main`**:
       1. Apply the dependency bump and the required code edits (verify the new
          API via context7 if unsure).
       2. Verify in the Nix dev shell — all must be clean:
          `cargo build --all-features`,
          `cargo clippy --all-targets --all-features -- -D warnings`,
          `cargo fmt --all --check`,
          `cargo test --all-features`.
       3. Run a **code-review agent** on the diff; address findings. For
          security-sensitive deps (crypto, RNG, auth) explicitly have the
          reviewer confirm no security/semantics regression.
       4. Commit atomically (conventional prefix, no AI trailer) and push.
       5. **Close the superseded Dependabot PR** with a comment referencing the
          commit SHA, and delete its branch.

3. **Sync the local working copy** once merges/pushes are done:
   `git pull --ff-only origin main` and, if a submodule moved,
   `git submodule update --init --recursive`.

4. **Verify `main` is green.** Watch the resulting CI + CodeQL runs
   (`gh run watch <id> --exit-status`) and confirm every job (stable test, MSRV,
   cargo audit, CodeQL) succeeds.

## Report

End with a concise table: each PR → action taken (merged / fixed-locally /
closed+ignored) and the final CI status. Flag anything left needing a human
decision (e.g. a major version bump with risky breakage).
