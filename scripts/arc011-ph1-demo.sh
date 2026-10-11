#!/usr/bin/env bash
# arc-011 PH-1 demonstration (architecture note section 8.1; R-69: the demo is the test).
#
#   bash scripts/arc011-ph1-demo.sh --scripted
#
# --scripted runs `hooks_scripted_demo` from crates/termlink-sidecar/tests/hooks_tests.rs:
# a throwaway estate (own HOME, runtime dir, hub, keys), two real `termlink-sidecar`
# processes, one message A -> B, B's hooks run with scripted hook input, a scripted
# transcript record in the shape the B1 spike measured, and A's status reaching
# HANDED_OVER. Nothing outside the estate is read or written; the estate is deleted
# afterwards.
#
# The live run with two real Claude Code sessions as `testdev` is B14 (OQ-3).
#
# Exit codes: 0 demo passed, 1 demo failed, 2 usage or tooling.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "${1:-}" in
  --scripted) ;;
  -h|--help)
    sed -n '2,15p' "$0"
    exit 0
    ;;
  *)
    echo "usage: $0 --scripted" >&2
    exit 2
    ;;
esac

command -v cargo >/dev/null 2>&1 || { echo "arc011-ph1-demo: cargo not found" >&2; exit 2; }
cd "$ROOT"
# The estate starts a throwaway hub from the workspace's own termlink build
# (newest of target/debug and target/release, or $TERMLINK_BIN); build one if none.
if [ -z "${TERMLINK_BIN:-}" ] && [ ! -x target/debug/termlink ] && [ ! -x target/release/termlink ]; then
  cargo build -q -p termlink --bin termlink || { echo "arc011-ph1-demo: cannot build termlink" >&2; exit 2; }
fi

if cargo test -q -p termlink-sidecar --test hooks_tests hooks_scripted_demo -- --exact --nocapture; then
  echo "arc011-ph1-demo --scripted: PASS"
  exit 0
fi
echo "arc011-ph1-demo --scripted: FAIL" >&2
exit 1
