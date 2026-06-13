#!/usr/bin/env bash
# scripts/check-error-code-docs.sh
#
# T-2217 (Level-C prevention): lint operator-facing docs/skills for error-code
# symbol↔number pairings that disagree with the authoritative `error_code`
# constants in crates/termlink-protocol/src/control.rs.
#
# Origin: a fictional `CLAIM_LAPSED`/`CLAIM_ALREADY_HELD` symbol (T-2215/T-2216)
# and two mislabeled rows (`CLAIM_NOT_FOUND` paired with -32018, `AUTH_FAIL`
# paired with -32001) propagated across 5 operator surfaces because nothing
# cross-checked prose against the source enum. This is the standing check that
# turns that whack-a-mole into a one-command gate.
#
# Scans: docs/operations/*.md, .claude/commands/*.md, CLAUDE.md
# Flags:  any `SYMBOL` immediately followed by `(-320NN)` where SYMBOL is not
#         the authoritative name for code -320NN.
# Exit:   0 = clean, 1 = at least one mismatch, 2 = tooling error.
#
# Usage: check-error-code-docs.sh [--json]

set -u
JSON_MODE=0
[ "${1:-}" = "--json" ] && JSON_MODE=1

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTROL="$ROOT/crates/termlink-protocol/src/control.rs"
[ -f "$CONTROL" ] || { echo "check-error-code-docs: control.rs not found at $CONTROL" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "check-error-code-docs: python3 required" >&2; exit 2; }

JSON_MODE=$JSON_MODE python3 - "$ROOT" "$CONTROL" <<'PY'
import re,sys,glob,os,json
root,control=sys.argv[1],sys.argv[2]
json_mode=os.environ.get("JSON_MODE")=="1"
auth={}
for line in open(control):
    m=re.search(r'pub const ([A-Z_]+): i64 = (-?\d+);',line)
    if m: auth[m.group(2)]=m.group(1)
files=glob.glob(os.path.join(root,"docs/operations/*.md")) \
    + glob.glob(os.path.join(root,".claude/commands/*.md")) \
    + [os.path.join(root,"CLAUDE.md")]
pat=re.compile(r'`?([A-Z][A-Z_]{3,})`?\s*\(\s*(-320\d{2})\s*\)')
mismatches=[]
for f in files:
    if not os.path.isfile(f): continue
    for i,line in enumerate(open(f,errors='replace'),1):
        for m in pat.finditer(line):
            sym,code=m.group(1),m.group(2)
            if code in auth and sym!=auth[code]:
                mismatches.append({"file":os.path.relpath(f,root),"line":i,
                                   "code":code,"doc_symbol":sym,"real_symbol":auth[code]})
if json_mode:
    print(json.dumps({"ok":len(mismatches)==0,"mismatches":mismatches,
                      "files_scanned":len([f for f in files if os.path.isfile(f)])}))
else:
    if mismatches:
        print("error-code doc lint: %d MISMATCH(es) — doc symbol != authoritative error_code name:"%len(mismatches))
        for d in mismatches:
            print("  %s:%d  %s  doc says '%s'  REAL='%s'"%(d["file"],d["line"],d["code"],d["doc_symbol"],d["real_symbol"]))
    else:
        print("error-code doc lint: CLEAN — all SYMBOL(-320NN) pairings match control.rs")
sys.exit(1 if mismatches else 0)
PY
