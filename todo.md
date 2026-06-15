# TODO — Freeze & document the public API surface (toward 1.0)

A practical playbook for locking down and documenting `litchee`'s public API,
tailored to the current crate. Ordered roughly the way it should be done.

> Tooling note: this project gets its toolchain from a **Nix dev shell**
> (`flake.nix` → `obazin/chess-flake`'s `rustShell` bundle). Add tools through the
> bundle's `extra` package list — **not** `cargo install`. The `rustShell`
> signature is `{ name, extra ? [ ], env ? {}, hook ? "" }`, and the workspace
> `lib` re-exports `pkgs`, so a tool from the pinned nixpkgs is reachable as
> `lib.pkgs.<tool>`.
>
> Both `cargo public-api` and `cargo-semver-checks` also need a **nightly rustdoc
> JSON** toolchain (they invoke nightly themselves). The pinned `rustToolchain`
> is currently stable, so a nightly rustdoc must be on PATH when these run — fine
> for tooling/CI even though the crate builds on stable + MSRV 1.95.

---

## 1. See exactly what's public today

- [ ] Add `cargo-public-api` to the dev flake's `extra` list (verified present
      in the pinned nixpkgs as **0.50.1** — no new flake input / `flake.lock`
      change needed):
      ```nix
      default = lib.bundles.rustShell {
        name = "litchee";
        extra = [ lib.pkgs.cargo-public-api ];
      };
      ```
      Confirm with `nix develop --command cargo public-api --version`.
- [ ] `cargo public-api` — prints every public item, fully qualified.
- [ ] Read it as a checklist. The bulk comes from `pub mod api` (every concern
      module + every `Lichess*` DTO + every builder), plus the root re-exports
      (`LichessClient`, `LichessClientBuilder`, `LichessError`, `Result`,
      `RetryPolicy`, `Secret`) and `pub mod error` / `pub mod model`.
- [ ] For each item ask: *is this intended, stable API?*

## 2. Minimize the surface (the real work)

- [ ] Demote plumbing to `pub(crate)` (already done for `Host`, `Config`,
      `ApiRequest`; catch any stragglers the audit surfaces).
- [ ] `#[doc(hidden)]` anything that must be `pub` for technical reasons but
      isn't real API.
- [ ] Decide the re-export shape deliberately: deep paths
      (`litchee::api::gameplay::games::LichessGame`) vs. flatter re-exports.
      Whatever is chosen becomes part of the frozen contract.
- [ ] Keep builders over public fields (already the pattern) to retain internal
      freedom.

## 3. Future-proof what stays public

- [ ] Confirm `#[non_exhaustive]` on **every** public enum and on structs that
      may gain fields. One missing attribute on a DTO makes adding a field a
      breaking change. (Widely applied already — verify completeness.)
- [ ] Seal any public trait that downstream should not implement.
- [ ] Decide the **leaked-dependency-types policy**: signatures currently expose
      `reqwest`, `url::Url`, and `serde_json::Value`. Each makes that crate's
      major version part of `litchee`'s semver contract. For 1.0, choose
      consciously which to re-export (so users get a matching version) and which
      to wrap.

## 4. Freeze it — snapshot + CI guard

- [ ] **API snapshot test** (catches *any* surface change):
      `cargo public-api > tests/public-api.txt` (commit the golden file), then a
      CI step (`cargo public-api diff tests/public-api.txt`) that fails on any
      change — forcing a conscious golden-file update + changelog entry.
- [ ] **Semver linting** (catches *breaking* changes): add `cargo-semver-checks`
      to the dev flake's `extra` list (`extra = [ lib.pkgs.cargo-semver-checks ];`),
      same mechanism as `cargo-public-api` above — not `cargo install`. Then add
      `cargo semver-checks check-release` as a release-gate CI job.

## 5. Documentation completeness

- [ ] Flip `missing_docs = "warn"` → `"deny"` in `Cargo.toml` so nothing public
      ships undocumented (CI already builds docs with `-D warnings`).
- [ ] Ensure every public module has a module-level (`//!`) overview.
- [ ] Add runnable `examples/` + doctests (compile-checked documentation that is
      itself part of the contract).
- [ ] Add `CHANGELOG.md` (Keep a Changelog format).
- [ ] Consider `#![doc(html_root_url = "https://docs.rs/litchee/<version>")]`.

## 6. Commit to a versioning policy

- [ ] Document in the README: pre-1.0 vs. the semver guarantees made at 1.0,
      the MSRV policy (currently pinned 1.95 — state whether a bump is minor or
      patch), and that `#[non_exhaustive]` means `match` arms need a wildcard.

---

## Suggested order for litchee

1. `cargo public-api` → audit, demote stragglers to `pub(crate)`,
   `#[doc(hidden)]` the rest.
2. Confirm `#[non_exhaustive]` on all public structs/enums; seal public traits.
3. Decide the leaked-dependency-types policy (reqwest / url / serde_json).
4. Commit the `cargo public-api` golden file + add the diff check and
   `cargo-semver-checks` to CI.
5. Flip `missing_docs` to `deny`, add `examples/` + `CHANGELOG.md`.

None of these are blockers for using the crate today; they are the work that
earns a stable 1.0 contract.
