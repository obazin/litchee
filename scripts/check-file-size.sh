#!/usr/bin/env bash
#
# Enforces the file half of CLAUDE.md rule 2: no Rust file may exceed 900 lines.
#
# The companion rule (no function over 20 lines) is enforced by clippy's
# too_many_lines lint, configured to 20 in clippy.toml — there is no clippy lint
# for file length, hence this script.
#
# Scope is tracked *and* new-but-unstaged files (`--others --exclude-standard`),
# so a module still being written is checked locally before it is ever added.
# Going through git keeps target/ and the vendored reference/lichess-api
# submodule out of it.
#
#   ./scripts/check-file-size.sh          # 900-line limit
#   LIMIT=600 ./scripts/check-file-size.sh
set -euo pipefail

limit="${LIMIT:-900}"
cd "$(git rev-parse --show-toplevel)"

# Collect into a file rather than a process substitution: `set -e`/`pipefail`
# do not cover a substitution, so a failing git would leave the loop reading
# nothing and the script would report "all clear" having inspected nothing.
# A command substitution can't stand in either — it strips the NUL separators.
# `-z` itself avoids core.quotePath mangling a path with non-ASCII bytes into
# an unopenable "src/caf\303\251.rs".
list=$(mktemp)
trap 'rm -f "${list}"' EXIT
git ls-files -z --cached --others --exclude-standard '*.rs' | sort -zu >"${list}"

status=0
checked=0
while IFS= read -r -d '' file; do
    # Tracked-but-deleted paths are still listed; skip rather than abort.
    [ -f "${file}" ] || continue
    checked=$((checked + 1))
    # awk over `wc -l`: wc counts newline terminators, so it reports one short
    # for a file whose last line is unterminated.
    lines=$(awk 'END { print NR }' "${file}")
    if [ "${lines}" -gt "${limit}" ]; then
        printf '%s: %d lines (limit %d)\n' "${file}" "${lines}" "${limit}" >&2
        status=1
    fi
done <"${list}"

if [ "${checked}" -eq 0 ]; then
    echo "check-file-size: found no .rs files to check — is this the repo root?" >&2
    exit 1
fi

if [ "${status}" -ne 0 ]; then
    cat >&2 <<'EOF'

CLAUDE.md rule 2: no file may exceed 900 lines of code.
Split the concern eagerly — extract helpers, or give it the oauth-style folder
treatment (a folder whose parts are independent units), as src/api/auth/oauth
and src/api/broadcasting/broadcasts do.
EOF
    exit 1
fi

echo "file size: ${checked} .rs files, all within ${limit} lines"
