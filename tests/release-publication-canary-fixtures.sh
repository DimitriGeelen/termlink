#!/usr/bin/env bash
# T-3243 — fixtures for scripts/check-release-publication-freshness.sh.
#
# Hermetic (PL-213): every case drives the canary through RELEASE_PUB_TEST_DIR
# (canned raw `gh api` responses + rc + stderr) and RELEASE_PUB_TEST_NOW, so no
# network, no gh, no GitHub state. Every case passes --no-heartbeat except the
# heartbeat case, which runs a COPY inside a scratch git repo so a fixture run can
# never refresh the real cron heartbeat (that would mask a dead cron, T-1723).
#
# Weighted toward the firing and fail-closed cases: a canary that is trivially
# green when its inputs are healthy proves nothing on its own.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHK="${CHK:-$REPO_ROOT/scripts/check-release-publication-freshness.sh}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }

NOW=1790000000                           # 2026-09-21T14:13:20Z
iso() { date -u -d "@$1" +%Y-%m-%dT%H:%M:%SZ; }
RECENT="$(iso $((NOW - 2*86400)))"       # 2 days old
OLD="$(iso $((NOW - 10*86400)))"         # 10 days old

# world <name> — a healthy baseline: published release, both workflows green 2d ago
world() {
    W="$TMP/$1"; mkdir -p "$W"
    echo "v9.9.9" > "$W/tag.txt"
    echo '{"tag_name":"v9.9.9","draft":false,"published_at":"2026-09-20T10:00:00Z"}' > "$W/release.json"
    for wf in install-check.yml doc-lint.yml; do
        echo "{\"total_count\":1,\"workflow_runs\":[{\"created_at\":\"$RECENT\",\"head_sha\":\"abcdef0123456\",\"conclusion\":\"success\"}]}" > "$W/runs-$wf.json"
    done
}
run() { # run <world-dir> [args...] → OUT, RC
    local w="$1"; shift
    OUT="$(RELEASE_PUB_TEST_DIR="$w" RELEASE_PUB_TEST_NOW="$NOW" bash "$CHK" --no-heartbeat "$@" 2>&1)"; RC=$?
}
expect() { # expect <label> <rc> [grep-pattern]
    if [ "$RC" = "$2" ] && { [ -z "${3:-}" ] || grep -q -- "$3" <<< "$OUT"; }; then ok "$1 (rc=$RC)"
    else bad "$1" "want rc=$2${3:+ + /$3/}, got rc=$RC: $(head -3 <<< "$OUT")"; fi
}

echo "T-3243 release-publication canary fixtures"

echo "== healthy =="
world h; run "$W";           expect "H1 published release + both workflows green is healthy" 0 "healthy"
run "$W" --quiet
if [ "$RC" = "0" ] && [ -z "$OUT" ]; then ok "H2 --quiet is SILENT when healthy (empty log = healthy)"
else bad "H2 --quiet healthy" "rc=$RC out=[$OUT]"; fi

echo "== firing: release =="
world r1; printf '' > "$W/release.json"; echo 1 > "$W/release.rc"; echo "gh: Not Found (HTTP 404)" > "$W/release.err"
run "$W";                    expect "R1 newest tag with NO release (HTTP 404) FIRES and names the tag" 1 "v9.9.9 has NO GitHub Release"
world r2; echo '{"tag_name":"v9.9.9","draft":true,"published_at":null}' > "$W/release.json"
run "$W";                    expect "R2 a DRAFT release FIRES" 1 "only a DRAFT"

echo "== firing: workflows =="
world w1; echo '{"total_count":0,"workflow_runs":[]}' > "$W/runs-install-check.yml.json"
run "$W";                    expect "W1 a workflow never green on main FIRES" 1 "install-check.yml has NO successful run"
world w2; echo "{\"workflow_runs\":[{\"created_at\":\"$OLD\",\"head_sha\":\"0123456789\"}]}" > "$W/runs-doc-lint.yml.json"
run "$W";                    expect "W2 last green older than the window FIRES with its age" 1 "doc-lint.yml last green on main 10d ago"
run "$W" --max-age-days 30;  expect "W3 the same run inside a wider window is healthy" 0
world w4; echo '{"total_count":0,"workflow_runs":[]}' > "$W/runs-doc-lint.yml.json"
echo 1 > "$W/release.rc"; echo "gh: Not Found (HTTP 404)" > "$W/release.err"
run "$W" --json
if [ "$RC" = "1" ] && python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); sys.exit(0 if len(d["firing"])==2 and not d["ok"] else 1)' <<< "$OUT"
then ok "W4 two independent findings are BOTH reported (rc=1, 2 in firing[])"
else bad "W4 both findings" "rc=$RC: $OUT"; fi

echo "== fail-closed (exit 2, never healthy) =="
world t1; printf '' > "$W/tag.txt"
run "$W";                    expect "T1 no v* tag is tooling, not healthy" 2 "no v\* tag"
world t2; echo 1 > "$W/release.rc"; echo "gh: Bad credentials (HTTP 401)" > "$W/release.err"
run "$W";                    expect "T2 a non-404 release error (auth) is tooling, not a finding" 2 "release lookup"
world t3; echo 'not json' > "$W/release.json"
run "$W";                    expect "T3 an unparseable release response is tooling" 2 "unparseable release"
world t4; echo 1 > "$W/runs-doc-lint.yml.rc"; echo "error connecting to api.github.com" > "$W/runs-doc-lint.yml.err"
run "$W";                    expect "T4 a failed run lookup is tooling" 2 "run lookup for doc-lint.yml"
world t5; echo '{"nope":1}' > "$W/runs-install-check.yml.json"
run "$W";                    expect "T5 an unparseable runs response is tooling" 2 "unparseable runs"
world t6
echo 1 > "$W/release.rc"; echo "gh: HTTP 500" > "$W/release.err"
run "$W" --json
if [ "$RC" = "2" ] && python3 -c 'import json,sys; d=json.loads(sys.stdin.read().splitlines()[0]); sys.exit(0 if d["verdict"]=="tooling" and d["ok"] is False else 1)' <<< "$OUT"
then ok "T6 --json on tooling says verdict=tooling, ok=false (rc=2)"
else bad "T6 json tooling" "rc=$RC: $OUT"; fi

echo "== flags =="
world f1; run "$W" --json
if [ "$RC" = "0" ] && python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); sys.exit(0 if d["ok"] and d["scope"] and len(d["checks"])==3 else 1)' <<< "$OUT"
then ok "F1 --json healthy carries ok, scope and one entry per check"
else bad "F1 json healthy" "rc=$RC: $OUT"; fi
world f2; echo 1 > "$W/release.rc"; echo "gh: Not Found (HTTP 404)" > "$W/release.err"
run "$W" --quiet
if [ "$RC" = "1" ] && head -1 <<< "$OUT" | grep -qE '^=== [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z ===$'
then ok "F2 a firing --quiet entry opens with a dated frame (T-3002)"
else bad "F2 dated frame" "rc=$RC: $(head -2 <<< "$OUT")"; fi
run "$W" --bogus;            expect "F3 unknown argument is rc=2" 2
run "$W" --max-age-days 0;   expect "F4 --max-age-days 0 is rc=2" 2
run "$W" --max-age-days x;   expect "F5 non-numeric --max-age-days is rc=2" 2
world f6; run "$W" --workflows "install-check.yml"
expect "F6 --workflows narrows the watched set" 0
grep -q "doc-lint" <<< "$OUT" && bad "F6b doc-lint excluded" "still reported: $OUT" || ok "F6b an unwatched workflow is not reported"

echo "== heartbeat (EXIT trap, T-2843) =="
HR="$TMP/hbrepo"; mkdir -p "$HR/scripts" && git -C "$HR" init -q && cp "$CHK" "$HR/scripts/"
world hb
( cd "$HR" && RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" bash scripts/check-release-publication-freshness.sh >/dev/null 2>&1 )
[ -s "$HR/.context/working/.release-publication-canary.heartbeat" ] && ok "B1 a completed run writes the heartbeat" || bad "B1 heartbeat" "not written"
rm -rf "$HR/.context"
( cd "$HR" && RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" bash scripts/check-release-publication-freshness.sh --no-heartbeat >/dev/null 2>&1 )
[ -e "$HR/.context/working/.release-publication-canary.heartbeat" ] && bad "B2 --no-heartbeat" "heartbeat written anyway" || ok "B2 --no-heartbeat writes no heartbeat"
echo 1 > "$W/release.rc"; echo "gh: HTTP 500" > "$W/release.err"
( cd "$HR" && RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" bash scripts/check-release-publication-freshness.sh >/dev/null 2>&1 )
[ -s "$HR/.context/working/.release-publication-canary.heartbeat" ] && ok "B3 a tooling exit still writes the heartbeat (the run completed)" || bad "B3 heartbeat on tooling" "not written"

echo "== mutant: the release check disabled must turn a case red =="
MUT="$TMP/mutant.sh"
sed 's/FIRING+=("release/CHECKS+=("release-muted/' "$CHK" > "$MUT"
if cmp -s "$CHK" "$MUT"; then bad "M0 mutant applied" "sed matched nothing — the mutant would test nothing"
else
    world m1; echo 1 > "$W/release.rc"; echo "gh: Not Found (HTTP 404)" > "$W/release.err"
    OUT="$(RELEASE_PUB_TEST_DIR="$W" RELEASE_PUB_TEST_NOW="$NOW" bash "$MUT" --no-heartbeat 2>&1)"; RC=$?
    [ "$RC" = "0" ] && ok "M1 with the release finding muted, R1's input reads healthy — so R1 is load-bearing" \
                    || bad "M1 mutant" "expected the mutant to go green (rc=0), got rc=$RC"
fi

echo ""
echo "release-publication canary fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ] || exit 1
