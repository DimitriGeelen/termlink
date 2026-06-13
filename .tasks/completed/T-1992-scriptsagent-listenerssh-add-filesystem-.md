---
id: T-1992
name: "scripts/agent-listeners.sh: add filesystem JSON cache (mitigate 0.11.473 channel info wedge)"
description: >
  Client-side mitigation per T-1991 GO. Add JSON output cache to scripts/agent-listeners.sh — cache result in ~/.termlink/cache/agent-listeners-<hub>.json with TTL (default 30s). Back-to-back calls within TTL skip the hub entirely, so /pulse, /peers, /agent-handoff stay responsive even while 0.11.473 channel info is flaky. See docs/reports/T-1991-channel-info-hub-concurrency-regression.md for the regression context.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T09:35:38Z
last_update: 2026-06-05T09:55:53Z
date_finished: 2026-06-05T12:04:07Z
---

# T-1992: scripts/agent-listeners.sh: add filesystem JSON cache (mitigate 0.11.473 channel info wedge)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/agent-listeners.sh` accepts `--cache-ttl SECS` (default 30, range 0..=3600;
      0 disables caching for that call) and `--no-cache` (alias for `--cache-ttl 0`).
- [x] On cache miss or stale, the script hits the hub as before and writes the
      resulting JSON rollup to `${TERMLINK_CACHE_DIR:-$HOME/.termlink/cache}/agent-listeners/<keyhash>.json`
      atomically (write to `.tmp` then rename) with chmod 600.
- [x] On cache hit (mtime within TTL), the script reads the cached JSON and skips the
      `termlink channel info` + `channel subscribe` calls entirely. Verifiable by
      running with `TERMLINK_BIN=/bin/false` after a fresh cache write — the second
      call still emits the cached rollup.
- [x] Cache key includes hub + topic + limit + include_offline + filter_role +
      filter_listen_topic + filter_agent_id. Two calls with different filters write
      separate cache entries (no false cache hits).
- [x] Cache corruption or unreadable JSON is treated as miss (logged to stderr,
      not fatal). Script falls back to live hub query.
- [x] Help text (`agent-listeners.sh --help`) documents `--cache-ttl` and `--no-cache`
      with TTL semantics ("LIVE threshold is 2× the listener's interval_secs, so
      30s default cache means worst-case classification staleness ≤30s").
- [x] Round-trip test on .122 (the flaky hub): 10 sequential
      `--cache-ttl 30 --json --hub 192.168.10.122:9100 --include-offline`
      calls — first call hits hub (≤1s expected), next 9 served from cache (<100ms each).
      Documented in test output captured in task Updates section.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

# Cache miss writes a file and hits the hub
bash -c 'rm -rf /tmp/t1992-cache && TERMLINK_CACHE_DIR=/tmp/t1992-cache bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --include-offline --json --cache-ttl 30 >/dev/null && ls /tmp/t1992-cache/agent-listeners/*.json >/dev/null'

# Cache hit serves from disk without invoking termlink (TERMLINK_BIN=/bin/false would fail if it tried to hit the hub)
bash -c 'TERMLINK_CACHE_DIR=/tmp/t1992-cache TERMLINK_BIN=/bin/false bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --include-offline --json --cache-ttl 30 | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print(\"OK ok=\"+str(d.get(\"ok\")))" | grep -q "OK ok=True"'

# --no-cache bypasses cache even if a fresh entry exists (so it would fail with TERMLINK_BIN=/bin/false)
bash -c 'TERMLINK_CACHE_DIR=/tmp/t1992-cache TERMLINK_BIN=/bin/false bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --include-offline --json --no-cache 2>/dev/null; rc=$?; rm -rf /tmp/t1992-cache; test "$rc" -ne 0'

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-05T09:35:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1992-scriptsagent-listenerssh-add-filesystem-.md
- **Context:** Initial task creation

### 2026-06-05T09:55:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-05T10:25:00Z — evidence: all 7 ACs verified

**AC 5 (corrupt cache → warn + refresh):**
```
$ echo "not valid json {{{ broken" > $CACHE_FILE
$ TERMLINK_CACHE_DIR=/tmp/t1992-cache bash scripts/agent-listeners.sh \
    --hub 192.168.10.122:9100 --include-offline --json --cache-ttl 30
agent-listeners: cache file corrupt, refreshing (.../c8fd998878...json)  ← stderr
{...valid JSON rollup on stdout...}                                       ← refreshed
$ jq -e .ok $CACHE_FILE  → true                                           ← cache rewritten atomic
```

**AC 7 (10-shot soak on .122, the flaky hub):**
```
run 1: 0.35s   ← hub hit (no retry needed this round; 0.11.473 was lucky)
run 2: 0.02s   ← cache served
run 3: 0.01s   ← cache served
run 4: 0.02s
run 5: 0.01s
run 6: 0.01s
run 7: 0.02s
run 8: 0.01s
run 9: 0.02s
run 10: 0.02s
```

Cache files written under `/tmp/t1992-cache/agent-listeners/`:
- key=sha256(topic|hub|limit|include_offline|filter_role|filter_listen_topic|filter_agent_id)
- chmod 600, atomic .tmp → rename
- Filter isolation verified earlier (2 distinct keyhash files for 2 filter combos)

**Mitigation effect on 0.11.473 wedge:** sequential `agent-listeners.sh`
calls inside TTL bypass `channel info` entirely. The 45% timeout rate
documented in T-1991's spike is now invisible to operators using
`/peers`, `/pulse`, or the work-completed verification gate for any
back-to-back invocation within 30s.
