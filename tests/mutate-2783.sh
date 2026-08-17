#!/usr/bin/env bash
# T-2783 mutation proof. Each mutation removes ONE guarded behaviour; the named
# test(s) must go RED. A guard nothing can break is a guard that proves nothing.
set -uo pipefail
cd "$(dirname "$0")/.."

CLI=crates/termlink-cli/src/commands/channel.rs
MCP=crates/termlink-mcp/src/tools.rs

# FAIL-CLOSED. Restore is `git checkout --`, which DISCARDS uncommitted work in
# these two files. This script is referenced from T-2783's `## Verification`
# block, so P-011 runs it at `--status work-completed` — exactly the moment the
# tree is most likely to be dirty. A mutation harness that eats the work it was
# meant to protect is worse than no harness, so refuse rather than risk it.
if ! git diff --quiet -- "$CLI" "$MCP" || ! git diff --cached --quiet -- "$CLI" "$MCP"; then
  echo "REFUSING: uncommitted changes in $CLI or $MCP." >&2
  echo "  This script restores by 'git checkout --', which would discard them." >&2
  echo "  Commit or stash those files first, then re-run." >&2
  exit 2
fi

restore() { git checkout -- "$CLI" "$MCP"; }
trap restore EXIT

run() { # <label> <pkg> <filter>
  local label=$1 pkg=$2 filter=$3
  local out
  out=$(cargo test -p "$pkg" "$filter" 2>&1 || true)
  if grep -q "0 failed" <<< "$out" && ! grep -q "FAILED" <<< "$out"; then
    echo "  [$label] STILL GREEN  <-- MUTATION NOT DETECTED"
  else
    local n
    n=$(grep -cE "^test .* FAILED" <<< "$out" || true)
    echo "  [$label] RED ($n test(s) failed) <-- detected"
  fi
}

echo "=== M1: discovery join removed (CLI) — return no discovered topics ==="
python3 - "$CLI" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
old="""    if fingerprint.is_empty() {
        return Vec::new();
    }
    let tracked: std::collections::HashSet<&str> =
        cursors.iter().map(|(t, _)| t.as_str()).collect();"""
new="""    return Vec::new();
    #[allow(unreachable_code)]
    let tracked: std::collections::HashSet<&str> =
        cursors.iter().map(|(t, _)| t.as_str()).collect();"""
assert old in s, "M1 anchor not found"
open(p,'w').write(s.replace(old,new,1))
PY
run "M1 cli-discovery" termlink discover
restore

echo "=== M2: discovery join removed (MCP) ==="
python3 - "$MCP" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
old="""    if fingerprint.is_empty() {
        return Vec::new();
    }
    let tracked: std::collections::HashSet<&str> =
        cursors.iter().map(|(t, _)| t.as_str()).collect();"""
new="""    return Vec::new();
    #[allow(unreachable_code)]
    let tracked: std::collections::HashSet<&str> =
        cursors.iter().map(|(t, _)| t.as_str()).collect();"""
assert old in s, "M2 anchor not found"
open(p,'w').write(s.replace(old,new,1))
PY
run "M2 mcp-discovery" termlink-mcp mcp_discover
restore

echo "=== M3: fleet TLS-fp dedup removed — keep every profile ==="
python3 - "$CLI" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
old="""    for (name, fp) in probed {
        match fp.as_deref() {
            Some(f) if !seen.insert(f) => skipped.push(name.clone()),
            _ => kept.push(name.clone()),
        }
    }"""
new="""    for (name, _fp) in probed {
        let _ = &mut seen;
        kept.push(name.clone());
    }"""
assert old in s, "M3 anchor not found"
open(p,'w').write(s.replace(old,new,1))
PY
run "M3 fleet-dedup" termlink fleet_dedups
restore

echo "=== M4: scope note emptied — the T-2782 guard must still be load-bearing ==="
python3 - "$CLI" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
old='''    let mut note = format!(
        "scope: read {where_}; {t} cursor-tracked topic(s) + {d} discovered dm topic(s) \\
         addressed to this identity.",
        t = s.tracked,
        d = s.discovered
    );'''
new='''    let _ = (&where_, s.tracked, s.discovered);
    let mut note = String::new();'''
assert old in s, "M4 anchor not found"
open(p,'w').write(s.replace(old,new,1))
PY
run "M4 scope-note" termlink inbox_scope
restore

echo "=== restored — residue check (want: no diff) ==="
git diff --stat -- "$CLI" "$MCP" | tail -3
echo "(empty above = clean restore)"
