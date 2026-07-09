# litchee — task runner. Run `just` (or `just --list`) to see recipes.
#
# Release tooling lives in the Nix dev shell (`nix develop`): git-cliff generates
# the changelog, cargo-edit's `set-version` bumps Cargo.toml, and gh publishes the
# GitHub release. All recipes assume you run them from inside `nix develop`.

repo := "obazin/litchee"

# Default recipe: list everything.
default:
    @just --list

# --- Release automation ------------------------------------------------------

# LEVEL is `patch`, `minor`, or `major`:
#   just release patch     # 0.1.3 -> 0.1.4
#   just release minor     # 0.1.3 -> 0.2.0
#   just release major     # 0.1.3 -> 1.0.0
# Pauses to show the notes and confirm before touching the remote (publishing is
# outward-facing), then commits the bump, tags, pushes, and creates the release.
#
# Bump the version, regenerate the changelog, tag, push, and publish a release.
release level: _require-clean
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{level}}" in
      patch|minor|major) ;;
      *) echo "level must be patch, minor, or major (got '{{level}}')"; exit 2 ;;
    esac

    # 0. Releases are cut from the default branch (the tag becomes GitHub's
    #    `--latest`); refuse other branches unless explicitly overridden.
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [ "${branch}" != "main" ] && [ "${ALLOW_RELEASE_BRANCH:-}" != "1" ]; then
      echo "Refusing to release from '${branch}' (expected main)." >&2
      echo "Set ALLOW_RELEASE_BRANCH=1 to override." >&2
      exit 1
    fi

    # 1. Bump the crate version in Cargo.toml (and Cargo.lock).
    cargo set-version --bump "{{level}}"
    version="$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)"
    tag="v${version}"
    echo "▶ Preparing ${tag} on ${branch}"

    # 2. Regenerate the full changelog. For the release body, take the same
    #    section but drop the redundant `## [vX]` heading (`--strip all` only
    #    removes the header/footer, not the body's version heading) — the GitHub
    #    release title already carries the version.
    git cliff --tag "${tag}" --output CHANGELOG.md
    notes="$(mktemp -t litchee-notes.XXXXXX)"
    git cliff --tag "${tag}" --unreleased --strip all | grep -v '^## \[' > "${notes}"

    # 3. Show the notes and gate the outward-facing steps on confirmation.
    #    `read` fails at EOF (non-TTY stdin); `|| true` keeps `set -e` from
    #    skipping the abort/revert branch below.
    echo "──────── release notes for ${tag} ────────"
    cat "${notes}"
    echo "──────────────────────────────────────────"
    reply=""
    read -r -p "Publish ${tag} with these notes? [y/N] " reply || true
    if [[ "${reply}" != "y" && "${reply}" != "Y" ]]; then
      echo "Aborted — reverting the version bump."
      git checkout -- Cargo.toml Cargo.lock 2>/dev/null || true
      git checkout -- CHANGELOG.md 2>/dev/null || rm -f CHANGELOG.md
      rm -f "${notes}"
      exit 1
    fi

    # 4. Commit the bump + changelog and create an annotated tag.
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore(release): ${tag}"
    git tag -a "${tag}" -m "litchee ${version}"

    # 5. Push the branch and the tag, then publish from the generated notes.
    #    If publishing fails after the push, the commit and tag are already on
    #    the remote — keep the notes and print an idempotent retry so a re-run of
    #    `just release` (which would bump *again*) is never needed.
    git push --follow-tags
    if ! gh release create "${tag}" \
      --title "litchee ${version}" \
      --notes-file "${notes}" \
      --latest --verify-tag; then
      kept="release-notes-${tag}.md"
      cp "${notes}" "${kept}"
      echo "✗ gh release failed, but ${tag} is already pushed. Retry with:" >&2
      echo "    gh release create ${tag} --title \"litchee ${version}\" --notes-file ${kept} --latest --verify-tag" >&2
      rm -f "${notes}"
      exit 1
    fi
    rm -f "${notes}"
    echo "✔ Published https://github.com/{{repo}}/releases/tag/${tag}"

# Preview the notes for the NEXT release at LEVEL without changing anything.
#
#   just release-preview minor
release-preview level:
    #!/usr/bin/env bash
    set -euo pipefail
    current="$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)"
    IFS='.' read -r major minor patch <<< "${current}"
    case "{{level}}" in
      patch) patch=$((patch + 1)) ;;
      minor) minor=$((minor + 1)); patch=0 ;;
      major) major=$((major + 1)); minor=0; patch=0 ;;
      *) echo "level must be patch, minor, or major (got '{{level}}')"; exit 2 ;;
    esac
    next="${major}.${minor}.${patch}"
    echo "Next {{level}} version: ${next} (from ${current})"
    git cliff --tag "v${next}" --unreleased --strip all | grep -v '^## \['

# Regenerate CHANGELOG.md from the full history (no bump, no release).
changelog:
    git cliff --output CHANGELOG.md

# Fail if the working tree has uncommitted changes.
_require-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain)" ]; then
      echo "working tree is not clean — commit or stash first" >&2
      exit 1
    fi
