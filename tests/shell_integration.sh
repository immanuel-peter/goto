#!/usr/bin/env bash
set -euo pipefail

BIN="$(pwd)/target/debug/__goto_bin"
ORIG="$(pwd -P)"
TMPDIR="$(mktemp -d)"
TMPDIR="$(cd "$TMPDIR" && pwd -P)"
export GOTO_CONFIG_DIR="$TMPDIR/config"
PASS=0
FAIL=0

trap 'cd "$ORIG"; rm -rf "$TMPDIR"' EXIT

__goto_bin() {
    "$BIN" "$@"
}

source shell/goto.sh

assert_dir() {
    local label expected actual
    label="$1"
    expected="$2"
    actual="$(pwd)"

    if [ "$actual" = "$expected" ]; then
        printf '  PASS %s\n' "$label"
        PASS=$((PASS + 1))
    else
        printf '  FAIL %s\n' "$label"
        printf '    expected: %s\n' "$expected"
        printf '    got:      %s\n' "$actual"
        FAIL=$((FAIL + 1))
    fi
}

printf 'Running shell integration tests...\n'

goto add shelltest "$TMPDIR"
goto shelltest
assert_dir "goto <alias> changes cwd" "$TMPDIR"
cd "$ORIG"

goto remove shelltest
if __goto_bin resolve shelltest >/dev/null 2>&1; then
    printf '  FAIL alias should have been removed\n'
    FAIL=$((FAIL + 1))
else
    printf '  PASS goto remove cleans up alias\n'
    PASS=$((PASS + 1))
fi

goto nonexistent_alias_xyz >/dev/null 2>&1 || true
assert_dir "bad alias does not change cwd" "$ORIG"

goto --help >/dev/null
assert_dir "goto --help does not change cwd" "$ORIG"

goto --version >/dev/null
assert_dir "goto --version does not change cwd" "$ORIG"

printf '\nResults: %s passed, %s failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
