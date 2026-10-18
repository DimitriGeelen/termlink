#!/usr/bin/env bash
# T-3472 — fw arc approve-driver must PERSIST a --scoring-file spec inline.
#
# Before the fix, arc_approve_driver handed --scoring-file to the static reviewer
# only and wrote a scoped_drivers entry without `scoring:`; the estimator reads only
# an inline block, so the driver scored nothing (arc-011's three drivers, approved
# 2026-10-06 with valid specs, were stored without them). Hermetic: a scratch
# project with one throwaway arc; the real vendored verb runs against it. No real
# arc, task or policy file is touched.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FWR="$ROOT/.agentic-framework"
EXAMPLE="$FWR/policy/driver-scoring-example.yaml"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL $1${2:+ — $2}"; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/.context/arcs" "$TMP/.tasks/active" "$TMP/.tasks/completed"
new_arc() {
    cat > "$TMP/.context/arcs/arc-t.yaml" <<'YAML'
id: arc-t
slug: arc-t
name: test arc
status: in-progress
scoped_drivers: []
proposed_scoped_drivers: []
YAML
}
approve() {  # args after the verb; human path (this is a scratch arc, not real state)
    (cd "$TMP" && PROJECT_ROOT="$TMP" FRAMEWORK_ROOT="$FWR" \
        bash -c 'source "$FRAMEWORK_ROOT/lib/arc.sh" 2>/dev/null; arc_approve_driver "$@"' _ "$@") >"$TMP/out" 2>&1
}
has_scoring() {  # <driver>: exit 0 when the stored entry carries a block the estimator accepts
    FWR="$FWR" python3 - "$TMP/.context/arcs/arc-t.yaml" "$1" <<'PY'
import importlib.util, os, sys, yaml
spec = importlib.util.spec_from_file_location("e", os.path.join(os.environ["FWR"], "agents/termlink/bvp-estimator/estimator.py"))
est = importlib.util.module_from_spec(spec); spec.loader.exec_module(est)
d = yaml.safe_load(open(sys.argv[1]))
e = next((s for s in d.get("scoped_drivers") or [] if s.get("name") == sys.argv[2]), None)
b = est.load_scoring_spec(e) if e else None
sys.exit(0 if b and not est.validate_scoring_spec(b) else 1)
PY
}
field() { python3 -c "import yaml,sys;d=yaml.safe_load(open('$TMP/.context/arcs/arc-t.yaml'));e=next((s for s in d['scoped_drivers'] if s['name']==sys.argv[1]),{});print(e.get(sys.argv[2],''))" "$1" "$2"; }

[ -f "$EXAMPLE" ] || { echo "tooling: $EXAMPLE missing"; exit 2; }
printf 'kind: signals\nlevels:\n  3: {bogus: [x]}\n' > "$TMP/invalid.yaml"

# 1. New approval with --scoring-file stores the spec inline (the bug).
new_arc
approve arc-t "D_ONE" --weight 4 --i-am-human --scoring-file "$EXAMPLE"
if has_scoring D_ONE; then ok "1 new approval persists the scoring block inline"; else bad "1 new approval persists the scoring block inline" "$(tail -3 "$TMP/out")"; fi

# 2. Approval without --scoring-file is unchanged (no scoring key).
approve arc-t "D_TWO" --weight 3 --i-am-human
if ! has_scoring D_TWO && [ -n "$(field D_TWO approved_at)" ]; then ok "2 approval without a spec stays spec-less"; else bad "2 approval without a spec stays spec-less"; fi

# 3. Invalid spec: refused, and NO driver entry is left behind.
approve arc-t "D_BAD" --weight 2 --i-am-human --scoring-file "$TMP/invalid.yaml"; rc=$?
if [ "$rc" -ne 0 ] && [ -z "$(field D_BAD weight)" ]; then ok "3 invalid spec refused before the entry exists"; else bad "3 invalid spec refused before the entry exists" "rc=$rc"; fi

# 4. Attach to an existing approved driver: spec stored, approval record kept.
before="$(field D_TWO approved_at)"
approve arc-t "D_TWO" --i-am-human --scoring-file "$EXAMPLE"
if has_scoring D_TWO && [ "$(field D_TWO approved_at)" = "$before" ]; then ok "4 attach stores the spec and keeps approved_at"; else bad "4 attach stores the spec and keeps approved_at" "$(tail -2 "$TMP/out")"; fi

# 5. Attach with a different --weight is refused (attaching never re-weights).
approve arc-t "D_TWO" --weight 6 --i-am-human --scoring-file "$EXAMPLE"; rc=$?
if [ "$rc" -ne 0 ] && [ "$(field D_TWO weight)" = "3" ]; then ok "5 attach refuses a weight change"; else bad "5 attach refuses a weight change" "rc=$rc"; fi

# 6. Attach without the operator path is refused.
approve arc-t "D_ONE" --scoring-file "$EXAMPLE"; rc=$?
if [ "$rc" -ne 0 ] && grep -q "needs the operator path" "$TMP/out"; then ok "6 attach refuses the non-operator path"; else bad "6 attach refuses the non-operator path" "rc=$rc"; fi

echo "--- $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
