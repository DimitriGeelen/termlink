#!/usr/bin/env bash
# guard-layer: source
# pipefail-grep-q-fixtures.sh — T-3631: hermetic proof for scripts/check-pipefail-grep-q.sh (the L-387 ratchet).
# Scratch roots under mktemp -d; the real baseline is never read or written. Weighted to the firing cases, plus a
# mutant showing the "count went up" case bites. Site lines are built from $P so this file has none of its own.
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHK="$REPO_ROOT/scripts/check-pipefail-grep-q.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }
P='|'
SITE="ls $P grep -q x"

root() {  # fresh scan root with one clean pipefail script and a baseline holding a.sh=1
    R="$TMP/r$1"; rm -rf "$R"; mkdir -p "$R/scripts" "$R/tests"
    printf '#!/usr/bin/env bash\nset -uo pipefail\necho clean\n' > "$R/scripts/clean.sh"
    printf '#!/usr/bin/env bash\nset -uo pipefail\n%s\n' "$SITE" > "$R/scripts/a.sh"
    printf '# hdr\n1 scripts/a.sh\n' > "$R/base"
}
run() { OUT="$(PIPEFAIL_GREPQ_ROOT="$R" PIPEFAIL_GREPQ_BASELINE="$R/base" bash "${CHK2:-$CHK}" "$@" 2>&1)"; RC=$?; }
has() { grep -q -- "$1" <<< "$OUT"; }

echo "T-3631 pipefail grep -q ratchet fixtures"
root 1; run;  [ "$RC" = 0 ] && has "clean — no new or grown site; 1 remaining" && ok "C1 baseline matches => clean (rc 0)" || bad C1 "rc=$RC $OUT"
root 2; printf '#!/usr/bin/env bash\nset -o pipefail\n%s\n' "$SITE" > "$R/tests/new.sh"; run
[ "$RC" = 1 ] && has "tests/new.sh: 1 (baseline 0)" && ok "F1 a NEW file with a site FIRES and is named" || bad F1 "rc=$RC $OUT"
root 3; printf '%s\n' "$SITE" >> "$R/scripts/a.sh"; run
[ "$RC" = 1 ] && has "scripts/a.sh: 2 (baseline 1)" && ok "F2 a baselined file whose count went UP fires" || bad F2 "rc=$RC $OUT"
root 4; printf '#!/usr/bin/env bash\nset -uo pipefail\necho fixed\n' > "$R/scripts/a.sh"; run
[ "$RC" = 0 ] && has "scripts/a.sh: 0 (baseline 1)" && ok "C2 a count that went DOWN is reported, never fires" || bad C2 "rc=$RC $OUT"
root 5; printf '#!/usr/bin/env bash\n%s\n' "$SITE" > "$R/scripts/nopf.sh"; run
[ "$RC" = 0 ] && ok "C3 a file without pipefail is out of scope" || bad C3 "rc=$RC $OUT"
root 6; printf '#!/usr/bin/env bash\nset -o pipefail\n# %s\n' "$SITE" > "$R/scripts/cmt.sh"; run
[ "$RC" = 0 ] && ok "C4 a commented site is not counted" || bad C4 "rc=$RC $OUT"
root 11; printf '#!/usr/bin/env bash\nset -o pipefail\ntrue %s%s grep -q x f\n' "$P" "$P" > "$R/scripts/or.sh"; run
[ "$RC" = 0 ] && ok "C5 'a || grep -q x f' is not a pipeline and is not counted" || bad C5 "rc=$RC $OUT"
root 7; rm -f "$R/base"; run
[ "$RC" = 2 ] && has "missing (fail-closed" && ok "T1 missing baseline => tooling rc 2, never clean" || bad T1 "rc=$RC $OUT"
root 8; printf '%s\n' "$SITE" >> "$R/scripts/a.sh"; cp "$R/base" "$R/base.before"; run --update-baseline
[ "$RC" = 1 ] && cmp -s "$R/base" "$R/base.before" && has "refusing to raise" && ok "U1 --update-baseline refuses to RAISE a count and leaves the baseline" || bad U1 "rc=$RC $OUT"
root 9; printf '#!/usr/bin/env bash\nset -uo pipefail\necho fixed\n' > "$R/scripts/a.sh"; run --update-baseline
[ "$RC" = 0 ] && ! grep -q "scripts/a.sh" "$R/base" && ok "U2 --update-baseline lowers the ratchet when counts went down" || bad U2 "rc=$RC $(cat "$R/base")"

# Mutant: the grew-check never trips — F2's input must then read clean, proving F2 bites.
CHK2="$TMP/mutant.sh"; sed 's/-gt "\$b" \]; then grew+=/-gt 999999 ]; then grew+=/' "$CHK" > "$CHK2"
if cmp -s "$CHK" "$CHK2"; then bad M1 "mutant sed matched nothing"
else root 10; printf '%s\n' "$SITE" >> "$R/scripts/a.sh"; run
     [ "$RC" = 0 ] && ok "M1 with the grew-check disabled, F2's input reads clean — F2 is load-bearing" || bad M1 "rc=$RC"; fi
unset CHK2

echo "T-3631 pipefail grep -q fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
