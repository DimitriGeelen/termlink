---
id: T-1419
name: "api-usage --json: add last_seen_iso per legacy caller (post-deploy freshness)"
description: >
  Extend `fw metrics api-usage --json` to surface `last_seen_iso` (and
  `last_seen_ts_ms`) per row in the three legacy_callers* arrays. Today the
  output gives counts but no freshness signal — operators verifying a
  post-deploy migration (T-1418, future hub rebuilds) can't distinguish
  "still calling" from "stale rolling-window residue." Tracking max(ts) per
  attribution key adds the missing signal in two additive JSON fields.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1166, observability, metrics, post-deploy-verification]
components: [.agentic-framework/agents/metrics/api-usage.sh]
related_tasks: [T-1166, T-1414, T-1416, T-1418, T-1408, T-1409, T-1410]
created: 2026-04-30T08:32:26Z
last_update: 2026-04-30T08:37:01Z
date_finished: 2026-04-30T08:37:01Z
---

# T-1419: api-usage --json: add last_seen_iso per legacy caller (post-deploy freshness)

## Context

The cut-ready gate (`--cut-ready`, T-1416) is binary: ready or not. The full
JSON output (T-1312) gives counts per (method, peer_ip), (method, peer_pid),
(method, from). Counts in a 24h or 7d rolling window can be high *even
after* a successful migration — the window includes pre-migration calls
that are aging out. An operator who just upgraded .143 (T-1418) and wants
to verify "no new calls since the restart" has no signal in the current
output beyond the count.

`last_seen_iso` is the missing signal. With it, the operator can:

- Compare row's `last_seen_iso` against deploy-restart timestamp
- Confirm migration took effect even when count > 0
- Identify which specific holdouts are live vs. stale

Implementation is contained: track `max(ts)` per attribution key during the
existing single-pass aggregation, format as ISO 8601 in the output.

## Implementation sketch

In `stats_for_window`, alongside `legacy_callers` Counter:
```python
last_seen_callers: dict[(str,str), int] = {}  # max ms ts per (method, from)
last_seen_pids:    dict[(str,int), int] = {}
last_seen_ips:     dict[(str,str), int] = {}
```
On each legacy entry, update `last_seen_X[key] = max(existing, ts)`.

Return tuple grows from 7 to 10 elements. Builders gain a `last_seen` dict
parameter and add two fields per row:
- `last_seen_ts_ms`: int (raw, sortable)
- `last_seen_iso`: ISO 8601 string (UTC, "Z" suffix)

JSON shape change: additive only (no removed fields, no renames). Existing
consumers unaffected.

## Acceptance Criteria

### Agent
- [x] `stats_for_window` now tracks `max(ts)` per (method, from), (method, peer_pid), (method, peer_ip) — three new dicts populated in the same pass that updates the existing Counters.
- [x] Return tuple grows from 7 to 10 elements; both call sites (`--cut-ready` short-circuit and `--json` paths) updated to either ignore the new fields or pass them to the builders.
- [x] `build_legacy_callers`, `build_legacy_callers_by_pid`, `build_legacy_callers_by_ip` accept the matching last_seen dict and emit two new fields per row: `last_seen_ts_ms` (int) and `last_seen_iso` (UTC string with "Z" suffix).
- [x] No fields removed/renamed in JSON output (additive only); cut-ready gate output shape unchanged.
- [x] Smoke test: `bash -c '.agentic-framework/bin/fw metrics api-usage --last-Nd 7 --json || true'` parses and contains `last_seen_iso` for every row that has a count.
- [x] `last_seen_iso` for the .143 inbox.status entry is within last 24h (current observed last_seen verifies live polling) — confirmed end-to-end against live audit log.
- [x] Mirrored to upstream `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` per channel-1 protocol.

### Human
- [ ] [REVIEW] Post-deploy of T-1418, the freshness signal correctly distinguishes live from stale
  **Steps:**
  1. Note current ISO timestamp before deploying T-1418's binary
  2. Deploy the binary + restart polling agents on .143
  3. Wait 5 min, then run: `fw metrics api-usage --last-Nd 1 --json | python3 -c "import json,sys; d=json.load(sys.stdin); [print(x) for x in d['legacy_callers_by_ip'] if x['peer_ip']=='192.168.10.143']"`
  **Expected:** `last_seen_iso` for .143 is BEFORE the deploy timestamp (i.e., no calls after restart). Count may still be non-zero (rolling window).
  **If not:** the upgrade didn't take — likely the polling agent supervisor wasn't restarted; re-do that step.

## Verification

bash -c '.agentic-framework/bin/fw metrics api-usage --last-Nd 7 --json || true' > /tmp/api-usage-t1419.json
test -s /tmp/api-usage-t1419.json
python3 -c 'import json; d=json.load(open("/tmp/api-usage-t1419.json")); rows = d["legacy_callers_by_ip"] + d["legacy_callers_by_pid"] + d["legacy_callers"]; missing = [r for r in rows if "last_seen_iso" not in r]; assert not missing, f"missing last_seen_iso in {len(missing)} rows: {missing[:3]}"'
python3 -c 'import json,re; d=json.load(open("/tmp/api-usage-t1419.json")); [re.match(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}", r["last_seen_iso"]) or (_ for _ in ()).throw(AssertionError(f"bad iso: {r}")) for r in d["legacy_callers_by_ip"]]'
test -f /opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh
diff -q .agentic-framework/agents/metrics/api-usage.sh /opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh

## Decisions

### 2026-04-30 — Why both ts_ms and ISO

- **Chose:** Emit both `last_seen_ts_ms` (int) and `last_seen_iso` (string)
- **Why:** ISO is human-readable and UI-friendly; raw ms is sort-friendly
  and arithmetic-safe (no parsing needed for "calls since X" comparisons).
  Cost is two fields instead of one — trivial.
- **Rejected:** ISO-only — forces consumers to re-parse a string for math.
  Rejected ts_ms-only — humans can't read it.

### 2026-04-30 — Why not breaking change to default human output

- **Chose:** Add freshness only to `--json` path; human output unchanged
- **Why:** Human output is multi-line, formatting is fragile, and the
  primary consumers of freshness are operator scripts and the watchtower
  page (both already use `--json`). Touching human output risks breaking
  unrelated agent-readable scrapes.
- **Rejected:** Add a "Last seen" column to human output — out of scope;
  small follow-up if anyone actually wants it.

## Updates

### 2026-04-30T08:32:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1419-api-usage---json-add-lastseeniso-per-leg.md
- **Context:** Initial task creation

### 2026-04-30T08:37:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
