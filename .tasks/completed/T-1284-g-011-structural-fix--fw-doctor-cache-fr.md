---
id: T-1284
name: "G-011 structural fix — fw doctor cache freshness check + own-hub live-read enforcement"
description: >
  Close G-011 (auth cache drift) by implementing the medium-term and long-term mitigations: (1) fw doctor compares ~/.termlink/secrets/<IP>.hex mtime/value against authoritative <runtime_dir>/hub.secret for self-hub profiles and warns on drift, (2) profiles using IP-keyed cache for self-hub read are deprecated with a migration hint to point secret_file directly at <runtime_dir>/hub.secret. Foundation for T-243 multi-turn agent conversation work — flaky auth blocks reliable multi-turn.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [auth, reliability, G-011]
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: []
created: 2026-04-25T22:31:17Z
last_update: 2026-04-26T11:12:14Z
date_finished: 2026-04-26T11:12:14Z
---

# T-1284: G-011 structural fix — fw doctor cache freshness check + own-hub live-read enforcement

## Context

T-1171 shipped a partial G-011 mitigation: `termlink doctor` audits `~/.termlink/secrets/*.hex` for (a) world/group-readable perms, (b) mtime older than the local hub's `hub.secret`. This catches some drift but the mtime heuristic produces false-positives for caches of OTHER hubs (legitimately older than the local hub's secret).

T-1284 closes the remaining G-011 gap with two structural improvements:

1. **Value comparison.** Read each cache file's hex value and compare against the local `hub.secret` value. When values match, the cache IS the local hub's secret — no drift. When values DIFFER and the cache mtime is older than the local hub's mtime, that's a real drift candidate, not a false-positive on mtime alone.

2. **hubs.toml profile audit for self-hub IP-keyed cache.** Scan `~/.termlink/hubs.toml`. For each profile whose `address` is loopback (`127.x.x.x`, `localhost`, `::1`) or `0.0.0.0` AND whose `secret_file` is under `~/.termlink/secrets/`, emit a deprecation warning + migration hint pointing `secret_file` directly at `<runtime_dir>/hub.secret`.

## Acceptance Criteria

### Agent
- [x] `audit_secret_cache` reads each cache file's hex contents and compares against `hub.secret` contents (when `local_hub` is provided). Matching values short-circuit — cache is healthy regardless of mtime.
- [x] Diverging-value + older-mtime path now uses "cache value diverges from local hub.secret" wording instead of legacy "older than" alone.
- [x] New function `audit_hubs_for_self_hub_cache(config, secrets_dir)` returns Vec<String> migration hints for self-hub profiles using IP-keyed cache (loopback addresses: `127.x.x.x`, `localhost`, `[::1]`, `0.0.0.0`).
- [x] `cmd_doctor` calls it; results emitted under check key `secret_cache_profiles` (warn on hint, pass on empty).
- [x] Test `cache_value_matches_hub_skips_mtime_warning`: matching value, older mtime → 0 issues
- [x] Test `cache_value_diverges_and_older_uses_diverges_wording`: divergent value, older mtime → 1 issue with new wording
- [x] Test `cache_diverging_but_newer_is_not_flagged`: divergent but newer (= probably remote-hub cache) → 0 issues
- [x] Test `audit_hubs_flags_loopback_profile_using_ip_cache`: 127.0.0.1 profile flagged with migration hint pointing at `<runtime_dir>/hub.secret`
- [x] Test `audit_hubs_does_not_flag_remote_profile_using_ip_cache`: remote IP profile NOT flagged
- [x] Test `audit_hubs_handles_localhost_and_ipv6_loopback`: `localhost:9100` + `[::1]:9100` both flagged
- [x] cargo test passes: `termlink` 223 unit + 172 integration. 0 failed.

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

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-25T22:31:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1284-g-011-structural-fix--fw-doctor-cache-fr.md
- **Context:** Initial task creation

### 2026-04-26T11:08:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T11:12:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
