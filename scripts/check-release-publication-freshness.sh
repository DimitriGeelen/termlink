#!/usr/bin/env bash
# check-release-publication-freshness.sh — T-3243 (SQ-9 Q3, G-069 class at the release layer)
#
# THE BLINDNESS THIS CLOSES
# -------------------------
# v0.12.0 was tagged and pushed, the OneDev->GitHub mirror carried the tag (the
# mirror-drift canary, T-1140, was green), and NO GitHub Release was ever
# published: the newest release stayed v0.11.2 while every install path
# (install.sh, Homebrew) kept serving it. Separately, `install-check.yml` was red
# on main for ~3 months — 947 consecutive failures, last green 2026-06-12 — and
# `doc-lint.yml` had no green run on main since 2026-08-16. Nothing fired on any
# of it (F19, T-3211 R6). The mirror canary proves a tag REACHES GitHub; nothing
# proved it became a RELEASE, or that the workflows gating a release still pass.
# That is shipped != live (G-069) one layer past where the other canaries look.
#
# WHAT IT CHECKS
# --------------
#   (a) RELEASE  — the newest local `v*` tag has a PUBLISHED (non-draft) GitHub
#                  Release. HTTP 404 on releases/tags/<tag> FIRES; a draft FIRES.
#   (b) WORKFLOW — each watched workflow (default install-check.yml, doc-lint.yml)
#                  has a SUCCESSFUL run on `main` created within --max-age-days
#                  (default 7). Never-green FIRES; last-green-too-old FIRES.
# It reads the REST API directly (`gh api`), NOT `gh run list --status success`:
# measured 2026-09-29, the list form reported install-check's last green as
# 2026-06-12 while the API returned a green run from that same afternoon.
#
# SCOPE: this detects whether the newest tag was PUBLISHED and whether the named
# workflows have been GREEN recently. It does NOT verify that release assets are
# downloadable or correct (check-release-artifact-drift.sh compares names), that
# the tag reached GitHub (check-mirror-freshness.sh), or that a green run tested
# anything in particular.
#
# EXIT: 0 healthy · 1 FIRING · 2 tooling (no gh, a failed/unauthenticated call,
#       an unparseable response, no v* tag). FAIL-CLOSED: "I could not look" is
#       never reported as healthy.
#
# FLAGS: --json · --quiet (silent when healthy; firing entries open with a dated
#        `=== <UTC> ===` frame, T-3002) · --no-heartbeat · --max-age-days N ·
#        --workflows "a.yml b.yml" · --repo OWNER/NAME
#
# TEST SEAM (PL-213 — no network): RELEASE_PUB_TEST_DIR=<dir> containing
#   tag.txt                      newest v* tag (empty file = no tag)
#   release.json / release.rc / release.err   raw releases/tags/<tag> response
#   runs-<workflow>.json / .rc / .err         raw workflows/<wf>/runs response
# RELEASE_PUB_TEST_NOW=<epoch seconds> pins "now".
set -uo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd))" || exit 2

HEARTBEAT_FILE=".context/working/.release-publication-canary.heartbeat"
FORMAT=text; QUIET=0; HEARTBEAT=1
MAX_AGE_DAYS="${RELEASE_PUB_MAX_AGE_DAYS:-7}"
WORKFLOWS="${RELEASE_PUB_WORKFLOWS:-install-check.yml doc-lint.yml}"
REPO="${RELEASE_PUB_REPO:-}"
[ -n "$REPO" ] || REPO='{owner}/{repo}'   # gh resolves these from the git remotes
TD="${RELEASE_PUB_TEST_DIR:-}"

usage() {
    sed -n '2,45p' "$0" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)          FORMAT=json; shift ;;
        --quiet)         QUIET=1; shift ;;
        --no-heartbeat)  HEARTBEAT=0; shift ;;
        --max-age-days)  MAX_AGE_DAYS="${2:-}"; shift 2 ;;
        --workflows)     WORKFLOWS="${2:-}"; shift 2 ;;
        --repo)          REPO="${2:-}"; shift 2 ;;
        -h|--help)       usage; exit 0 ;;
        *) echo "check-release-publication: unknown arg: $1" >&2; exit 2 ;;
    esac
done

case "$MAX_AGE_DAYS" in ''|*[!0-9]*) echo "check-release-publication: --max-age-days must be a positive integer" >&2; exit 2 ;; esac
[ "$MAX_AGE_DAYS" -ge 1 ] || { echo "check-release-publication: --max-age-days must be >= 1" >&2; exit 2; }
[ -n "$WORKFLOWS" ] || { echo "check-release-publication: no workflows to watch" >&2; exit 2; }

# Heartbeat on EXIT (T-2691/T-2843): freshness proves the run FINISHED. A tooling
# exit (2) still writes it — that run completed; its stderr is what /canaries reads.
_canary_hb() {
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null && date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT_FILE" 2>/dev/null || true
}
if [ "$HEARTBEAT" -eq 1 ]; then trap _canary_hb EXIT; fi

command -v python3 >/dev/null 2>&1 || { echo "check-release-publication: python3 not found — no verdict" >&2; exit 2; }
if [ -z "$TD" ]; then
    command -v gh >/dev/null 2>&1 || { echo "check-release-publication: gh CLI not found — no verdict" >&2; exit 2; }
fi

tooling() { # tooling <message> — fail closed
    if [ "$FORMAT" = "json" ]; then
        python3 -c 'import json,sys; print(json.dumps({"ok": False, "verdict": "tooling", "error": sys.argv[1]}))' "$1"
    fi
    echo "check-release-publication: $1 — no verdict" >&2
    exit 2
}

# api_get <seam-key> <api-path> → sets API_OUT, API_RC, API_ERR
api_get() {
    if [ -n "$TD" ]; then
        API_OUT="$(cat "$TD/$1.json" 2>/dev/null)"
        API_RC="$(cat "$TD/$1.rc" 2>/dev/null || echo 0)"
        API_ERR="$(cat "$TD/$1.err" 2>/dev/null)"
        return 0
    fi
    local errf; errf="$(mktemp)"
    API_OUT="$(timeout 60 gh api "$2" 2>"$errf")"; API_RC=$?
    API_ERR="$(cat "$errf")"; rm -f "$errf"
}

NOW="${RELEASE_PUB_TEST_NOW:-$(date -u +%s)}"
FIRING=()   # "check<TAB>detail"
CHECKS=()   # "check<TAB>state<TAB>detail"

# ---- (a) RELEASE -----------------------------------------------------------
if [ -n "$TD" ]; then
    TAG="$(head -1 "$TD/tag.txt" 2>/dev/null | tr -d '[:space:]')"
else
    TAG="$(git tag -l 'v*' --sort=-v:refname 2>/dev/null | head -1)"
fi
[ -n "$TAG" ] || tooling "no v* tag found — nothing to check a release against"

api_get release "repos/$REPO/releases/tags/$TAG"
if [ "$API_RC" != "0" ]; then
    if printf '%s' "$API_ERR" | grep -q 'HTTP 404'; then
        FIRING+=("release	newest tag $TAG has NO GitHub Release (HTTP 404) — tagged but never published")
        CHECKS+=("release	firing	$TAG: no release")
    else
        tooling "release lookup for $TAG failed (rc=$API_RC): $(printf '%s' "$API_ERR" | head -1)"
    fi
else
    rel="$(printf '%s' "$API_OUT" | python3 -c '
import json,sys
try:
    d=json.load(sys.stdin)
except Exception:
    print("PARSE"); sys.exit(0)
if not isinstance(d,dict) or "draft" not in d:
    print("PARSE"); sys.exit(0)
print("DRAFT" if d.get("draft") or not d.get("published_at") else "OK\t"+str(d.get("published_at")))
')"
    case "$rel" in
        PARSE)  tooling "unparseable release response for $TAG" ;;
        DRAFT)  FIRING+=("release	newest tag $TAG has only a DRAFT release — not published")
                CHECKS+=("release	firing	$TAG: draft") ;;
        OK*)    CHECKS+=("release	ok	$TAG published ${rel#OK	}") ;;
        *)      tooling "unexpected release classification for $TAG" ;;
    esac
fi

# ---- (b) WORKFLOWS ---------------------------------------------------------
for wf in $WORKFLOWS; do
    api_get "runs-$wf" "repos/$REPO/actions/workflows/$wf/runs?branch=main&status=success&per_page=1"
    [ "$API_RC" = "0" ] || tooling "run lookup for $wf failed (rc=$API_RC): $(printf '%s' "$API_ERR" | head -1)"
    res="$(printf '%s' "$API_OUT" | python3 -c '
import json,sys,datetime
now=int(sys.argv[1]); maxd=int(sys.argv[2])
try:
    d=json.load(sys.stdin); runs=d["workflow_runs"]
except Exception:
    print("PARSE"); sys.exit(0)
if not runs:
    print("NEVER"); sys.exit(0)
r=runs[0]; ts=r.get("created_at") or ""
try:
    t=int(datetime.datetime.strptime(ts,"%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=datetime.timezone.utc).timestamp())
except Exception:
    print("PARSE"); sys.exit(0)
age=(now-t)//86400
sha=(r.get("head_sha") or "")[:9]
print(("STALE" if now-t > maxd*86400 else "OK")+"\t%s\t%d\t%s" % (ts, age, sha))
' "$NOW" "$MAX_AGE_DAYS")"
    case "$res" in
        PARSE)  tooling "unparseable runs response for $wf" ;;
        NEVER)  FIRING+=("workflow	$wf has NO successful run on main at all")
                CHECKS+=("workflow:$wf	firing	never green on main") ;;
        STALE*) IFS=$'\t' read -r _ ts age sha <<< "$res"
                FIRING+=("workflow	$wf last green on main ${age}d ago ($ts, $sha) — older than ${MAX_AGE_DAYS}d")
                CHECKS+=("workflow:$wf	firing	last green $ts") ;;
        OK*)    IFS=$'\t' read -r _ ts age sha <<< "$res"
                CHECKS+=("workflow:$wf	ok	last green $ts ($sha, ${age}d)") ;;
        *)      tooling "unexpected run classification for $wf" ;;
    esac
done

SCOPE="detects whether the newest v* tag has a published GitHub Release and whether the watched workflows had a successful run on main within the window; does NOT verify release assets, tag mirroring, or what a green run tested"

# ---- verdict ---------------------------------------------------------------
if [ "$FORMAT" = "json" ]; then
    python3 - "$SCOPE" "$TAG" "$MAX_AGE_DAYS" "${#FIRING[@]}" <<'PYEOF' "${FIRING[@]+"${FIRING[@]}"}" "--CHECKS--" "${CHECKS[@]+"${CHECKS[@]}"}"
import json,sys
scope,tag,maxd,nf=sys.argv[1:5]; rest=sys.argv[5:]
i=rest.index("--CHECKS--"); fir=rest[:i]; chk=rest[i+1:]
print(json.dumps({
  "ok": len(fir)==0,
  "verdict": "healthy" if not fir else "firing",
  "newest_tag": tag, "max_age_days": int(maxd),
  "firing": [dict(zip(("check","detail"), f.split("\t",1))) for f in fir],
  "checks": [dict(zip(("check","state","detail"), c.split("\t",2))) for c in chk],
  "scope": scope}))
PYEOF
elif [ "${#FIRING[@]}" -gt 0 ]; then
    [ "$QUIET" -eq 1 ] && echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-release-publication: FIRING — ${#FIRING[@]} finding(s)"
    for f in "${FIRING[@]}"; do echo "  FIRING  ${f#*	}"; done
    echo "  operator: publish the release (re-run release.yml for $TAG, or cut the next tag"
    echo "            once CI is green); for a stale workflow, open its latest run and fix main."
    echo "  SCOPE: $SCOPE"
elif [ "$QUIET" -eq 0 ]; then
    echo "check-release-publication: healthy"
    for c in "${CHECKS[@]}"; do IFS=$'\t' read -r n s d <<< "$c"; echo "  $s  $n: $d"; done
    echo "  SCOPE: $SCOPE"
fi

[ "${#FIRING[@]}" -eq 0 ] || exit 1
exit 0
