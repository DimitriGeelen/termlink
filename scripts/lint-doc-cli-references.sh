#!/usr/bin/env bash
# lint-doc-cli-references.sh — PL-206 mitigation layer (b).
#
# Sweeps docs/operations/*.md, .claude/commands/*.md, and CLAUDE.md for
# the three known instances of doc-CLI drift (PL-206). Run before
# committing doc changes; wire to CI to catch regressions.
#
# Drift class: markdown code blocks and prose embed CLI invocations and
# struct/JSON field names without compile-time verification. When the
# CLI evolves (new flag mutex, field renames), recipe docs lapse
# silently — first failure surfaces at user copy-paste time.
#
# Captured instances (this script knows about):
#   1. `.claimed_by` / `claims.claimed_by` — the hub uses `claimer`
#      (crates/termlink-bus/src/claim.rs:22 ClaimInfo). Fixed in:
#      T-2129 (recipe), T-2130 (agent-find-idle.md), T-2131 (CLAUDE.md).
#   2. `agent dms --watch` — `--watch` is documented `--json`-incompatible
#      (T-1559); piping `agent dms --watch` to `jq` errors immediately.
#      Fixed in: T-2129.
#   3. `substrate primitive #11` — SUBSTRATE-PULSE is a composition,
#      not a §6 manifest primitive. T-2026 reserves #11 for typed
#      agent-launch. Fixed in: T-2127 (line 546), T-2129 (line 571).
#   4. `next_free_offset` — fictional ClaimsSummary field. Real struct
#      has active_count/expired_count/oldest_active_age_ms/next_active_expiry_ms.
#      Used in a broken "canonical orchestrator pattern" that polled a
#      non-existent offset. Fixed in: T-2134 (orchestrator-recipe.md).
#   5. `.result.X` for CLI verb output — raw JSON-RPC over Unix-socket
#      wraps response in `.result`, but the CLI verb (`hub status --governor
#      --json`) wraps in `.governor`. Mixing the envelopes silently breaks
#      every jq selector downstream. Fixed in: T-2135 (substrate-governor.md
#      + substrate-offline-queue-recipe.md + substrate-post-idempotency.md).
#
# Usage:
#   bash scripts/lint-doc-cli-references.sh              # scan and report
#   bash scripts/lint-doc-cli-references.sh --help       # show help
#   bash scripts/lint-doc-cli-references.sh --json       # JSON envelope
#
# Exit codes: 0 = clean, 1 = drift found, 2 = arg error
#
# References: PL-206 (.context/project/learnings.yaml), T-2129..T-2132.

set -u

show_help() {
  cat <<'EOF'
lint-doc-cli-references.sh — PL-206 doc-CLI drift lint.

Scans these paths for known drift patterns:
  docs/operations/*.md
  .claude/commands/*.md
  CLAUDE.md

Drift patterns (PL-206) — 5 known patterns:
  1. claimed_by as field name (correct: claimer)              — fixed T-2129/30/31
  2. agent dms --watch (incompatible with --json)             — fixed T-2129
  3. substrate primitive #11 (SUBSTRATE-PULSE not in §6)      — fixed T-2127/29
  4. next_free_offset (fictional ClaimsSummary field)         — fixed T-2134
  5. .result.{capacity_hits,rate_hits,dedupe_} envelope mix   — fixed T-2135

Flags:
  --help    Show this help and exit 0
  --json    Emit a JSON envelope instead of human-format

Exit codes:
  0   Clean — no drift found
  1   One or more patterns matched (drift present)
  2   Argument error

See: PL-206 in .context/project/learnings.yaml
     docs/operations/substrate-orchestrator-recipe.md (the canonical recipe)
EOF
}

JSON=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) show_help; exit 0 ;;
    --json) JSON=1; shift ;;
    *) echo "lint-doc-cli-references.sh: unknown arg: $1" >&2; show_help >&2; exit 2 ;;
  esac
done

cd "$(dirname "$0")/.." || { echo "lint-doc-cli-references: cannot cd to repo root" >&2; exit 2; }

# Collect target paths (glob expanded; missing dirs are tolerated).
paths=()
for p in docs/operations/*.md .claude/commands/*.md CLAUDE.md; do
  [[ -f "$p" ]] && paths+=("$p")
done
if [[ ${#paths[@]} -eq 0 ]]; then
  echo "lint-doc-cli-references: no target docs found (run from repo root)" >&2
  exit 2
fi

# Scan each pattern. Output is captured per-pattern so we can format
# either human or JSON at the end.
#
# Pattern 1: `claimed_by` in EXECUTABLE position. The orchestrator-recipe
# legitimately mentions `.claimed_by` in an explanatory comment ("uses
# `.claimer` ... NOT `.claimed_by`") — scope the regex to executable jq
# selectors, derivation pseudo-syntax, and API-contract phrasing rather
# than every textual mention. Mirrors T-2129's verification refinement.
hits_claimed_by=$(grep -nE '(jq -r ".*select.*claimed_by|DISTINCT\(claimed_by|current \`claimed_by\`|claims\.claimed_by)' "${paths[@]}" 2>/dev/null || true)

# Pattern 2: `agent dms --watch` invocation anywhere (always wrong in a
# documented pipeline-to-jq pattern; `--watch` excludes `--json` per
# T-1559).
hits_agent_dms_watch=$(grep -nE 'agent dms --watch' "${paths[@]}" 2>/dev/null || true)

# Pattern 3: `substrate primitive #11` (SUBSTRATE-PULSE manifest-slot
# claim). T-2127 + T-2129 fixed line 546 + 571 of substrate-orchestrator-recipe.md;
# this catches any reintroduction.
hits_primitive_11=$(grep -nE 'substrate primitive #11' "${paths[@]}" 2>/dev/null || true)

# Pattern 4: `next_free_offset` — fictional ClaimsSummary field used in
# a broken "canonical orchestrator pattern" (T-2134). Real ClaimsSummary
# struct has active_count/expired_count/oldest_active_age_ms/
# next_active_expiry_ms — see crates/termlink-bus/src/claim.rs. Catch any
# reintroduction in either jq selector, prose, or shell-variable name.
hits_next_free_offset=$(grep -nE 'next_free_offset' "${paths[@]}" 2>/dev/null || true)

# Pattern 5: `.result.X` envelope confusion for CLI verbs (T-2135). The
# raw JSON-RPC `hub.governor_status` wraps under `.result`, but the CLI
# verb `hub status --governor --json` wraps under `.governor`. Operators
# copy-paste a working raw-RPC selector into a CLI pipeline and every
# downstream jq read returns `null`. Scope tightly to the three known
# governor counter fields so the lint doesn't false-positive on
# legitimate `.result.X` in raw-RPC docs (which still wrap with `.result`).
hits_result_envelope=$(grep -nE '\.result\.(capacity_hits_total|rate_hits_total|dedupe_(hits_total|entries_active|ttl_ms))|\.result \| \{(capacity_hits_total|rate_hits_total|dedupe_)' "${paths[@]}" 2>/dev/null || true)

# Count hits per pattern.
n1=$([ -z "$hits_claimed_by" ] && echo 0 || echo "$hits_claimed_by" | wc -l)
n2=$([ -z "$hits_agent_dms_watch" ] && echo 0 || echo "$hits_agent_dms_watch" | wc -l)
n3=$([ -z "$hits_primitive_11" ] && echo 0 || echo "$hits_primitive_11" | wc -l)
n4=$([ -z "$hits_next_free_offset" ] && echo 0 || echo "$hits_next_free_offset" | wc -l)
n5=$([ -z "$hits_result_envelope" ] && echo 0 || echo "$hits_result_envelope" | wc -l)
total=$((n1 + n2 + n3 + n4 + n5))

if [[ "$JSON" -eq 1 ]]; then
  # Emit JSON envelope. Keep it simple; no jq dependency.
  esc() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/' | tr -d '\n'; }
  printf '{"ok":%s,"total_hits":%d,"patterns":[' "$([ "$total" -eq 0 ] && echo true || echo false)" "$total"
  printf '{"id":"claimed_by_as_field","hits":%d,"fixed_in":["T-2129","T-2130","T-2131"],"raw":"%s"},' "$n1" "$(esc "$hits_claimed_by")"
  printf '{"id":"agent_dms_watch","hits":%d,"fixed_in":["T-2129"],"raw":"%s"},' "$n2" "$(esc "$hits_agent_dms_watch")"
  printf '{"id":"substrate_primitive_11","hits":%d,"fixed_in":["T-2127","T-2129"],"raw":"%s"},' "$n3" "$(esc "$hits_primitive_11")"
  printf '{"id":"next_free_offset","hits":%d,"fixed_in":["T-2134"],"raw":"%s"},' "$n4" "$(esc "$hits_next_free_offset")"
  printf '{"id":"result_envelope_confusion","hits":%d,"fixed_in":["T-2135"],"raw":"%s"}' "$n5" "$(esc "$hits_result_envelope")"
  printf '],"reference":"PL-206"}\n'
  [[ "$total" -eq 0 ]] && exit 0 || exit 1
fi

# Human format.
echo "lint-doc-cli-references.sh — PL-206 doc-CLI drift sweep"
echo "  Scanned: ${#paths[@]} file(s) across docs/operations + .claude/commands + CLAUDE.md"
echo

if [[ "$total" -eq 0 ]]; then
  echo "  Status: clean — no drift found ✓"
  echo "  PL-206 prevention layer (b) reports no known-pattern hits."
  exit 0
fi

echo "  Status: DRIFT FOUND ($total hit(s) across $(( (n1>0)+(n2>0)+(n3>0)+(n4>0)+(n5>0) )) pattern(s))"
echo
if [[ "$n1" -gt 0 ]]; then
  echo "  Pattern 1 — claimed_by as field name (correct: claimer); fixed precedent: T-2129/30/31"
  echo "$hits_claimed_by" | sed 's/^/    /'
  echo
fi
if [[ "$n2" -gt 0 ]]; then
  echo "  Pattern 2 — agent dms --watch (incompatible with --json per T-1559); fixed precedent: T-2129"
  echo "$hits_agent_dms_watch" | sed 's/^/    /'
  echo
fi
if [[ "$n3" -gt 0 ]]; then
  echo "  Pattern 3 — substrate primitive #11 (SUBSTRATE-PULSE is a composition); fixed precedent: T-2127/29"
  echo "$hits_primitive_11" | sed 's/^/    /'
  echo
fi
if [[ "$n4" -gt 0 ]]; then
  echo "  Pattern 4 — next_free_offset (fictional ClaimsSummary field, see crates/termlink-bus/src/claim.rs); fixed precedent: T-2134"
  echo "$hits_next_free_offset" | sed 's/^/    /'
  echo
fi
if [[ "$n5" -gt 0 ]]; then
  echo "  Pattern 5 — .result.X envelope used for CLI verb output (CLI wraps under .governor, raw RPC wraps under .result); fixed precedent: T-2135"
  echo "$hits_result_envelope" | sed 's/^/    /'
  echo
fi
echo "  Fix: replace with the correct form, then re-run this script to confirm clean."
echo "  Reference: PL-206 in .context/project/learnings.yaml"
exit 1
