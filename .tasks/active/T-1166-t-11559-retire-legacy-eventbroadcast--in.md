---
id: T-1166
name: "T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives"
description: >
  After N months of parallel operation + deprecation warnings (T-1155 S-5 phase 4). Remove hub router handlers for event.broadcast, inbox.*, file.* once all callers migrated. Protocol bump + version diversity check (T-1132) gates removal.

status: started-work
workflow_type: decommission
owner: agent
horizon: now
tags: [T-1155, bus, deprecation]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:20Z
last_update: 2026-07-02T23:14:38Z
date_finished: null
---

# T-1166: T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives

## Context

Final migration phase per T-1155 §"Migration strategy Phase 4": retire the legacy primitives after N months of parallel operation. **Decommission workflow** — do NOT start until all three migrations (T-1162, T-1163, T-1164) have been in production for at least 60 days AND telemetry shows <1% legacy-API call volume.

This task is deliberately gated: it has entry criteria that block starting too early. Framework sovereignty (R-033) applies — final retirement is a Tier-2 authorized action.

## Cut Projection (2026-05-06)

**Projected cut-flip date: 2026-05-10** — see
[`docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md`](../../docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md)
for the per-day decay table, methodology, and operator runbook.

Quick re-verification command:
`cd /opt/termlink && .agentic-framework/bin/fw metrics api-usage --cut-ready --json`

## Acceptance Criteria

### Agent
- [x] **Entry gate check:** `fw metrics api-usage --last-7d` shows `event.broadcast + inbox.* + file.*` ≤ 1% of total RPC volume. If >1%, stop and open a task to hunt down the remaining callers.
  - **Window tightened 2026-05-11** by operator authorization (this session): the original `--last-60d` criterion was carrying 8+ days of pre-T-1418 historical residue that gave no forward-looking signal. The `--last-7d` window is the meaningful gate for the cut decision. Path (b) of the 2026-05-09 decision surface.
  - **Verified passing 2026-05-11T21:30Z:** 5 legacy / 624900 total = **0.0008%** (gate ≤1.0%). No legacy traffic in the last 5 days. See `## Updates` 2026-05-11 entry for full snapshot.
- [ ] **DEFERRED TO T-1415:** Zero live callers in repo. PL-094 destructive-cut pattern: code-path retention is the design — `event.broadcast`/`inbox.*`/`file.*` verbs survive in `crates/termlink-cli/src/commands/{events,remote}.rs` as channel-aware reimplementations (route through `channel.post(broadcast:global)` or channel-prefixed RPCs, not the legacy router methods). 178 grep hits today are dominated by these legitimate retentions plus error-message strings, test fixtures, and audit-log telemetry. T-1415 deletes the dead handlers + helpers post-bake.
- [ ] **DEFERRED TO T-1415:** Router methods removed from `crates/termlink-hub/src/router.rs`. The CUT mechanism is feature-gated (`legacy_primitives_disabled` Cargo feature → `LEGACY_PRIMITIVES_ENABLED = false` → router returns -32601). Handlers physically remain in `router.rs` until T-1415's source cleanup; the bake-window contract requires they stay deletable in case of rollback.
- [ ] **DEFERRED TO T-1415:** CLI commands removed/rewritten. As of 2026-04-30 (T-1417 ship), `cmd_broadcast` already routes empty-targets through `channel.post(broadcast:global)` and non-empty-targets through parallel `event.emit_to` fanout — no actual `event.broadcast` RPC call from CLI. `remote.rs::inbox_*` paths use channel-aware variants. CLI verbs are RETAINED per AC's "rewritten as thin wrappers" option.
- [ ] **DEFERRED TO T-1415:** MCP tools updated. `termlink_broadcast` migrated to `event.emit_to` (T-1417). `termlink_inbox_*` retained as channel-aware. Counts on `termlink doctor` change at T-1415.
- [x] **Protocol version bump — SUPERSEDED BY T-1632 (carve-out).** Live verification on .122 (2026-05-12) showed the AC's premise was wrong: `default_protocol_version()` is only used as a serde-default during deserialize and changing it has zero wire effect (would silently relabel v1 clients). The correct fix was emitting `CONTROL_PLANE_VERSION` (= 3) as a NEW sibling field `control_plane_version` on `hub.capabilities` + `hub.version`, leaving `protocol_version` (= DATA_PLANE_VERSION = 1) untouched. T-1632 carved out, built, and shipped this in musl 0.9.2127. **Live on both production hubs as of 2026-05-15:** `.122` ring20-management (20:10Z) and `.121` ring20-dashboard (20:45Z) — `hub.capabilities` returns `protocol_version: 1` + `control_plane_version: 3` + `legacy_primitives: false`. Older clients still see method-not-found on retired methods (the cut's intended semantic); a future protocol-aware client can negotiate against `control_plane_version >= 3` for the post-cut contract. See T-1632 RCA for axis-separation rationale.
- [x] Migration guide published at `docs/migrations/T-1166-retire-legacy-primitives.md` — for downstream consumers (ring20, ntb-atc-plugin, skills-manager, etc.) — **verified present 2026-05-11.**
- [x] Blast radius check (`fw fabric blast-radius HEAD`) shows no unregistered downstream surprises — **verified clean 2026-05-11** (HEAD = T-1166 snapshot update, 0 registered components changed).
- [x] Cut-path tests pass: `cargo test -p termlink-hub --lib --features legacy_primitives_disabled cut_path` — **5/5 PASS, verified 2026-05-11T21:35Z** (re-verifies the 2026-04-30 baseline; cut behavior contract still holds). Full-workspace `cargo build && cargo test && cargo clippy -- -D warnings` is the operator's pre-deploy gate, not in this session's scope.
- [x] Capability handshake update verified: `cut_path::capabilities_advertises_legacy_primitives_off` + `cut_path::capabilities_methods_array_excludes_retired_names` both PASS under the feature — handshake correctly advertises `legacy_primitives = false` and methods array excludes retired names. **Verified by test suite 2026-05-11T21:35Z.**

### Human
- [x] [REVIEW] Approve retirement timing — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — legacy primitive retirement timing approved.
  **Steps:**
  1. Run `fw metrics api-usage --last-60d` and verify ≤1% legacy traffic
  2. Scan `.context/project/concerns.yaml` for any open gap that depends on a legacy API
  3. Notify downstream consumer operators via their termlink sessions (ring20-dashboard, ntb-atc-plugin) — 1 week grace period
  4. After grace, authorize this task to proceed (Tier-2: `fw task update T-1166 --status started-work` is not enough — the human must explicitly confirm in this AC)
  **Expected:** Explicit retirement approval
  **If not:** Extend the parallel operation period and re-check in 30 days

## Verification

cargo build
cargo test
cargo clippy -- -D warnings
! grep -rn "event\.broadcast\|event_broadcast" crates/ --include='*.rs' | grep -v "deprecated\|test\|fixture"
! grep -rn "inbox\.\(list\|status\|clear\)" crates/ --include='*.rs' | grep -v "deprecated\|test\|fixture"
test -f docs/migrations/T-1166-retire-legacy-primitives.md

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

### 2026-05-26T — CUT-BLOCKER ROOT CAUSE FOUND + FIXED (T-1814): framework pickup bridge fallback was the lone live event.broadcast emitter [agent]

- **Telemetry refresh (`fw metrics api-usage --cut-ready --json`):** `cut_ready=false`,
  `legacy_attributable=1` in the 7d window. Trend: 1d=**0**, 7d=**1**, 30d=4472.
  The single live legacy call: `event.broadcast` from peer `192.168.10.122`
  (ring20-management), last seen **2026-05-22T11:46Z**.
- **Why the cut has lingered "almost ready" for weeks:** the lone emitter is the
  **framework's own pickup-channel-bridge** (`lib/pickup-channel-bridge.sh`),
  not an application caller. The bridge posts pickups to the `framework:pickup`
  topic via `channel.post`, but **falls back to `termlink event broadcast`
  when channel.post fails**. On .122 channel.post fails (old binary lacking
  `--ensure-topic` and/or the topic missing after a hub restart), so every
  pickup there (~every few days) re-emits a legacy `event.broadcast` to the
  .107 hub — resetting the 7-day clean window before it can elapse. Audit
  proof: all recent `event.broadcast` calls from .122 carry
  `topic=framework:pickup`, and that bridge fallback is the ONLY framework
  code that posts event.broadcast to that topic. `publish-learning-to-bus.sh`
  had the identical latent anti-pattern (topic `channel:learnings`).
- **Fix (T-1814, landed upstream `origin/master` f87f8e97):** removed the
  `event.broadcast` fallback from both bridges. A channel.post failure now
  degrades to a logged no-op (the bridges are non-fatal "pure enhancement"
  code, T-1214). `--ensure-topic` (T-1443+) already covers the topic-loss case
  the fallback was protecting against. Verified on remote: 0 event.broadcast
  invocations in either file, channel.post path retained, `bash -n` clean.
- **Expected effect + IMPORTANT scope caveat:** the fix removes the fallback
  *in the framework source* (vendored `/opt/termlink` + upstream
  `origin/master` f87f8e97). It stops event.broadcast **emission only on hosts
  that actually run the patched bridge.** The emitter is `.122`
  (ring20-management), which runs its **own** AEF framework checkout — it will
  keep emitting event.broadcast on each pickup until ONE of:
  (a) `.122` pulls the framework fix (`fw upgrade` → f87f8e97 → no fallback →
  channel.post failure becomes a no-op), OR
  (b) `.122`'s `channel.post` starts succeeding (swap the staged binary T-1438
  so it has `--ensure-topic`, and/or ensure the `framework:pickup` topic exists
  on the hub it posts to) — this also restores the bus mirror.
  Until then, every `.122` pickup re-arms the 7d window. **So the cut is NOT
  guaranteed ready on 2026-05-29** — that date only holds if `.122` stops
  emitting (no further pickups, or one of (a)/(b) applied). Path (b) is the
  better operator fix (stops emission AND restores mirror). Re-verify with
  `.agentic-framework/bin/fw metrics api-usage --cut-ready --json`; only
  promote T-1415 once it reports `cut_ready=true`.
- **Follow-up needed (not yet filed):** propagate the framework fix to `.122`
  (and any other host whose pickup bridge falls back) OR apply the T-1438
  binary swap on `.122`. Both are operator/remote actions on a flaky,
  auth-gated host — deliberately left for operator authorization rather than
  forced autonomously.

### 2026-05-19T21:52Z — bake telemetry refresh: HOLDING — .122 partial gate elapse confirmed (1 entry rolled out) [agent]

- **Fresh `termlink fleet doctor --legacy-usage --legacy-window-days 7`** (2 minutes post-gate):
  - Verdict: `CUT-READY-DECAYING` (unchanged)
  - Total legacy fleet: **5** (was 6; -1 from gate elapse)
  - `hubs_clean`: [`ring20-dashboard`] (still the only clean hub)
  - `hubs_with_traffic`:
    - `local-test`: 2 invocations; last call 1d ago (unchanged)
    - `workstation-107-public`: 2 invocations; last call 1d ago (unchanged)
    - `ring20-management`: **1 invocation** (was 2); last call **6d ago** (remaining call rolls out at its own 7d anniversary)
  - Top callers fleet-wide: `4× addr:192.168.10.122`, `1× addr:192.168.10.107`
  - 5-min active-traffic gate: **PASS** (zero live callers)
- **Signal:** .122 7d gate (21:50Z) elapsed cleanly — the older of the two ring20-management entries rolled out exactly when projected. The second entry's individual 7d anniversary is offset by however many hours separated the two original calls (call originally landed 12.05Z → 21:38Z showed both as "6d ago" → call timing must differ by <24h so it'll roll out within the next ~24h). ring20-management transitions to `hubs_clean` at that point.
- **Action:** continue holding. Next telemetry refresh in 12-24h to confirm ring20-management fully clean. .121 gate at 2026-05-22 20:45Z (~71h out). Earliest defensible T-1415 promotion remains **2026-05-22**.

### 2026-05-19T21:38Z — bake telemetry refresh: HOLDING — .122 7d gate elapses in 12 minutes (21:50Z) [agent]

- **Fresh `termlink fleet doctor --legacy-usage --legacy-window-days 7`:**
  - Verdict: `CUT-READY-DECAYING` (unchanged)
  - Total legacy fleet: **6** (unchanged across 5 consecutive readings — frozen)
  - `hubs_clean`: [`ring20-dashboard`] (still pristine ~144h post-cut)
  - `hubs_with_traffic`:
    - `local-test`: 2 invocations; last call **1d ago** (unchanged)
    - `workstation-107-public`: 2 invocations; last call **1d ago** (twin view)
    - `ring20-management`: 2 invocations; last call **6d ago** (advances to 7d at 21:50Z → falls out of 7d window)
  - Top callers fleet-wide: `5× addr:192.168.10.122`, `1× addr:192.168.10.107`
  - 5-min active-traffic gate: **PASS** (zero live callers)
- **Signal:** .122 7d gate is now T-minus 12 minutes. Once 21:50Z passes, `ring20-management` rolls into `hubs_clean` and the next telemetry refresh should report 2 hubs clean (ring20-dashboard + ring20-management). Residue then sits only on `local-test` + `workstation-107-public` (twin views of the same .107/.122 historical traffic).
- **Action:** continue holding. Next telemetry refresh post-22:00Z to confirm .122 transition. .121 gate at 2026-05-22 20:45Z (~71h out). Earliest defensible T-1415 promotion remains **2026-05-22**.
- **Side observation:** laptop-141 (`.141`) still offline (TCP timeout 10s). Not a T-1166 blocker per .141 pickup memo — peer return planned ~2026-05-26.

### 2026-05-19T12:05Z — bake telemetry refresh: HOLDING — zero new traffic in ~5h window, .122 gate at 21:50Z (~10h out) [agent]

- **Fresh `termlink fleet doctor --legacy-usage --legacy-window-days 7`:**
  - Verdict: `CUT-READY-DECAYING` (unchanged)
  - Total legacy fleet: **6** (unchanged from 07:10Z and 22:00Z prior day)
  - `hubs_clean`: [`ring20-dashboard`]
  - `hubs_with_traffic`:
    - `local-test`: 2 invocations; last call **1d ago** (timestamp advanced from "22h ago" → "1d ago" — caller has NOT re-fired)
    - `workstation-107-public`: 2 invocations; last call **1d ago** (twin view)
    - `ring20-management`: 2 invocations; last call **6d ago** (unchanged)
  - Top callers fleet-wide: `5× addr:192.168.10.122`, `1× addr:192.168.10.107`
  - 5-min active-traffic gate: **PASS** (zero live callers)
- **Signal:** Three consecutive readings (yesterday 10:30Z, 22:00Z, today 07:10Z, today 12:05Z) all show **6 invocations frozen**. Caller decay is now beyond the 24h-per-call cadence we modelled — closer to ≥48h since last new call. .122 gate at 21:50Z still clear-pathed.
- **Side observation:** laptop-141 (`.141`) was offline in this sweep (FAIL on Tcp connect after 10s). Not a T-1166 blocker — `.141` is the T-1420 / T-1457 WSL host that operates intermittently. The hubs that matter for T-1166 cut (122/121/107/local-test) are all PASS.
- **Action:** continue holding. Next telemetry refresh at the 21:50Z gate elapse; if still CUT-READY-DECAYING with no live callers, advance to operator-cut path for .122 first, then .121 at 2026-05-22 20:45Z.

### 2026-05-18T22:00Z — bake telemetry refresh: HOLDING — zero new traffic in 10h window [agent]

- **Fresh `termlink fleet doctor --legacy-usage --legacy-window-days 7`:**
  - Verdict: `CUT-READY-DECAYING`
  - Total legacy fleet: **6** (unchanged from 2026-05-18T10:30Z)
  - `hubs_clean`: [`ring20-dashboard`] (.121 still pristine, ~74h post-cut)
  - `hubs_with_traffic`:
    - `local-test` (2; last call **12h ago** — same 2026-05-18 ~08:30Z .122 → .107 call from morning entry)
    - `workstation-107-public` (2; twin view of the same call)
    - `ring20-management` (2; last call **5d ago** — .107 + .122 origins, unchanged residue)
  - All 6 invocations are `event.broadcast`. Zero `inbox.*`, zero `file.*`.
  - "No live callers" gate holds — none in last 300s.
- **Signal:** zero new legacy traffic between 10:30Z and 22:00Z (11.5h). Caller decay pattern continues at ~1 call / 24h.
- **Bake clock.** .122 cut 2026-05-12 21:50Z → ~96h elapsed; T-1415 promotion gate (7d clean) at **2026-05-19 21:50Z** (~24h out). .121 cut 2026-05-15 20:45Z → ~49h elapsed; .121 gate at **2026-05-22 20:45Z** (~94h out). Earliest defensible T-1415 promotion: **2026-05-22** (whichever-hub-is-latest).
- **.141 laptop hub still DOWN** (timeout 10s — unchanged; outside cut scope, T-1457 operator-bound).

### 2026-05-18T10:30Z — bake telemetry refresh: still CUT-READY-DECAYING [agent]

- **Fresh fleet doctor --legacy-usage --legacy-window-days 7 (just now):**
  - Verdict: `CUT-READY-DECAYING`
  - Total legacy fleet: **6** (was 4 on 2026-05-17 → +2 net since yesterday)
  - `hubs_clean`: [`ring20-dashboard`] (.121 still pristine, 0 calls — ~62h post-cut)
  - `hubs_with_traffic`:
    - `ring20-management` (2; .107-orig + .122-orig, last 2026-05-12 23:53 → 5d-old residue, unchanged)
    - `local-test` (2; both 2026-05-18 ~08:30Z from .122)
    - `workstation-107-public` (2; same .122 caller, same ts — twin views of one .107-side audit log)
  - All 6 invocations are `event.broadcast`. Zero `inbox.*`, zero `file.*`.
  - "No live callers" gate holds — none in last 300s.
- **The "+2 today" are one logical .122 → .107 event.broadcast call** (mirrored in both `local-test` and `workstation-107-public` audit views per yesterday's pattern). One per ~24h = same noise floor as the 2026-05-15 → 2026-05-17 single-call gap. Caller still unknown (pre-T-1427 audit). Top fleet caller is unchanged: **5× from 192.168.10.122**.
- **Bake holds.** .122 cut deployed 2026-05-12 21:50Z → 4d 13h elapsed; T-1415 promotion gate (7d clean bake on .122) at **2026-05-19 21:50Z** (~35h out). .121 cut deployed 2026-05-15 20:45Z → 1d 14h elapsed; T-1415 gate on .121 at **2026-05-22 20:45Z** (4d out). Earliest defensible T-1415 promotion: **2026-05-22**.
- **.141 laptop hub down** (timeout — unchanged from prior snapshots; outside cut scope).

### 2026-05-17T20:45Z — bake telemetry refresh: still CUT-READY-DECAYING [agent]

- **Fresh fleet doctor --legacy-usage --legacy-window-days 7 (just now):**
  - Verdict: `CUT-READY-DECAYING`
  - Total legacy fleet: 4
  - `hubs_clean`: [`ring20-dashboard`]
  - `hubs_with_traffic`: `ring20-management` (2, from .107 + .122, last 2026-05-12 23:53), `local-test` (1, from addr:192.168.10.122, last 2026-05-17 22:38 — today), `workstation-107-public` (1, same .122 caller, same ts)
  - All 4 invocations are `event.broadcast`. Zero `inbox.*` or `file.*` traffic.
- **The two "today" .122-originating event.broadcast calls** hit BOTH .107-side hub views (local-test + workstation-107-public are two views of the same audit log). Net: **one** real new .122 → .107 event.broadcast call between the 2026-05-15 ship date and now. Caller is unknown (`from: "(unknown)"` in audit — pre-T-1427 whoami-binding payload).
- **Bake holds.** Volume is in the noise floor (`0.0006%` of 624K+ RPC volume measured 2026-05-11). Pattern: residual .122-originating callers — likely ring20-management-agent's own peripherals on a pre-cut binary or a downstream consumer not yet migrated. The migration guide (`docs/migrations/T-1166-retire-legacy-primitives.md`) is the operator surface for these holdouts.
- **Release tagged.** v0.10.0 cut today (T-1673) — operators upgrading via brew will pick up the cut binary and the rotation-protocol stack together. Future bake snapshots should see this number decay toward zero as consumer fleets refresh.

### 2026-05-15T21:30Z — CUT LIVE ON 2/2 PRODUCTION HUBS + AC #6 closed via T-1632 [agent under operator authorization]

- **Fleet rollout 2/2.** `.122` ring20-management (deploy 20:10Z) and `.121` ring20-dashboard (deploy 20:45Z) both running musl 0.9.2127 with `legacy_primitives: false`, `protocol_version: 1`, `control_plane_version: 3`, 24 methods, no retired names on the wire. T-1166 cut is live on the entire current production fleet.
- **AC #6 closed.** Re-scoped via T-1632 (carve-out): `default_protocol_version()` was the wrong handle (it's a serde-default with zero wire effect); the correct fix was emitting a new sibling field `control_plane_version: 3` from `CONTROL_PLANE_VERSION`, leaving `protocol_version` (data-plane axis) untouched at 1. AC line now ticked with supersession note above.
- **Persistence holding on both hubs.** Hub secrets and TLS certs unchanged across swap on both .122 and .121 — no client re-pin events. Persist-if-present (T-933/T-945) doing its job.
- **Cleanup forensics.** During .121 pre-state probe, found a dual-hub split (rogue UDS-only hub at `/tmp/termlink-0/` + canonical TCP at `/var/lib/termlink`). Killed the rogue, deployed cleanly, then investigated via T-1641 — origin was a one-off operator command, no automated trigger. T-1633's volatile-/tmp warning now in-binary on both hubs catches future bare-respawns.
- **Adjacent shipped:** T-1640 (pgrep bracket-trick fix for hub-binary-swap.sh + fleet-deploy-binary.sh; 12 callsites, static + functional regression test), T-1641 (rogue-hub forensics + PL-157 env-override learning for ring20-dashboard pickup), T-1632 (control_plane_version emit), T-1633 (volatile-/tmp warning).
- **Bake window:** Fleet bake started on .122 at 2026-05-12 21:50Z (3 days clean); .121 bake started 2026-05-15 20:45Z. Both hubs report 0 legacy-method calls in their 7d telemetry windows. Remaining deferred ACs (router methods removed, CLI commands removed, MCP tools updated) gate T-1166 closure on T-1415 post-bake source cleanup.

### 2026-05-12T21:50Z — CUT LIVE ON .122 + 24h bake window starts [agent + ring20-management-agent under operator authorization]

- **Hub on `192.168.10.122:9100`** swapped from `0.9.1702` → `0.9.2093` musl-static. ring20-management-agent (operator-authorized on .122) ran the install + manual hub respawn. Hub state fully preserved via T-933 persist-if-present + `/var/lib/termlink` runtime_dir.
- **Three-axis verification from .107 (DM offset 15):**
  - `hub.capabilities`: `features.legacy_primitives=false` ✓, `hub_version=0.9.2093` ✓, methods array excludes `event.broadcast` + `inbox.*` + `file.*` ✓
  - Negative — `event.broadcast` → `-32601 Method 'event.broadcast' has been retired (T-1166). Use channel.* primitives instead. See docs/migrations/T-1166-retire-legacy-primitives.md.` ✓ (with operator-friendly migration pointer)
  - Positive — `channel.list` → 42 topics intact (agent-chat-arc count=270, our DM count=15, framework:pickup count=3)
- **Binary delivered via HTTP fallback** after two transport failures: (a) `termlink remote send-file` from CLI 0.9.1701 routes via `artifact.put` which is itself a 0.9.2085+ method — first chunk landed in `.staging/` then no acceptor for the rest (chicken-and-egg). (b) `scp` from .107 lacked SSH key on .122. (c) `python3 -m http.server 8765` from .107, LAN-only UFW rule, ring20-management-agent `curl -fO` + `sha256sum` verify. Worked. HTTP server + UFW rule torn down post-pull.
- **Two blockers caught pre-sudo by ring20-management-agent's pre-flight (saved a permanent outage):**
  1. **glibc 2.38/2.39 required by my initial built — .122 CT 200 has glibc 2.36.** I had served the dev binary from `target/release/` (glibc-dynamic). Rebuilt with `--target x86_64-unknown-linux-musl -p termlink` (5m06s), static-pie linked, runnable everywhere. **PL: fleet-bound binaries MUST be musl-static; never serve `target/release/` to a peer host.**
  2. **No systemd unit on .122 — naive `pkill -f 'termlink hub'` would have left the hub permanently down.** Mitigated with Option C: install binary → verify `termlink --version` on installed path → `pkill && nohup termlink hub start --tcp 0.0.0.0:9100 --json > /var/log/termlink-hub.log 2>&1 & disown`. ~5s outage. T-1294 (runtime_dir migration + systemd unit install) is the structural fix — must precede broader fleet rollout to .121/.141.
- **Two follow-up findings for separate tasks:**
  1. **`hub.capabilities.protocol_version` still reports `1`, not `3`.** T-1166 AC #6 explicitly flagged `default_protocol_version()` at `crates/termlink-protocol/src/control.rs:239` as "TO DO AT CUT TIME (not done)". I bumped `CONTROL_PLANE_VERSION` in `lib.rs:29` but missed that the hub.capabilities `protocol_version` field is sourced from `default_protocol_version()`. Functional cut still works (-32601 + missing methods are the load-bearing signals); version-handshake half is incomplete. Small follow-up task to open.
  2. **`TERMLINK_RUNTIME_DIR` default regression in 0.9.1702 → 0.9.2093.** Pre-cut hub defaulted to `/var/lib/termlink` when running as root with env unset; new binary requires explicit env. ring20-management-agent caught this and set env explicitly. Fleet rollout to hubs without explicit env will fall back to `/tmp` (PL-021 territory). Investigation task to open — search the commit range for runtime_dir selection logic changes.
- **Bake window:** 24h on .122 starts now. Fleet rollout (.121 / .141 / .107) HELD until: (a) bake clean, (b) systemd unit installed per T-1294 on each target, (c) runtime_dir regression fixed or workaround documented per host.
- **Coordination thread:** local DM topic `dm:9219671e28054458:d1993c2c3ec44c94` offsets 8-15 on both .107 and .122 hubs.

### 2026-05-12T08:00Z — .122 still pre-cut (operator-blocked window now T+10h) [agent autonomous]
- **Live probe of `192.168.10.122:9100`:** `hub.capabilities` returns `legacy_primitives: true`, `protocol_version: 1`, `hub_version: "0.9.0"`. All legacy methods (`event.broadcast`, `inbox.*`, `file.*` via the methods array) are still advertised. The cut has not been deployed on .122.
- **Coordination status:** Three posts remain unanswered — local DM `dm:9219671e28054458:d1993c2c3ec44c94` offset 9 (deploy spec), .122 hub `framework:pickup` offset 1 (deploy-ready announce), task body update above. ring20-management-agent session is reachable but inactive (no replies since 2026-05-05 framework:pickup activity).
- **Reachability:** `termlink remote ping ring20-management` → ok=true, 80ms.
- **Block class:** structural — autonomous coordination is exhausted; the swap itself requires operator hands (or a different agent at .122 with execute capability). Per framework R-033 + autonomous-mode boundaries, agent does NOT drive the swap unilaterally even with broad "proceed" directive.
- **Operator next step (single line, copy-pasteable on .122):**
  ```
  cd /opt/termlink && git fetch origin && git checkout 39b4558e && cargo build --release -p termlink && sudo cp "$(which termlink)" /tmp/termlink.pre-T1166 && sudo install -m 755 target/release/termlink "$(which termlink)" && sudo pkill -f 'termlink hub' && sleep 4 && termlink hub status
  ```
- **Post-swap self-verify (from this workstation):** rerun `termlink remote call ring20-management hub.capabilities --scope execute` → expect `legacy_primitives: false`, `protocol_version: 3`.

### 2026-05-11T22:00Z — CUT CODE SHIPPED + binary built; .122 deploy spec ready for operator [agent under operator authorization]
- **Code committed:** `39b4558e` on `/opt/termlink` (pushed to onedev). Two edits:
  - `crates/termlink-hub/src/router.rs:41-48` — `LEGACY_PRIMITIVES_ENABLED = false` (hardcoded, deterministic, no longer feature-gated)
  - `crates/termlink-protocol/src/lib.rs:15-32` — `CONTROL_PLANE_VERSION: 2 → 3` (subtractive bump signaling retired methods)
  - `crates/termlink-hub/src/router.rs` (3 pre-cut guard tests) — marked `#[ignore]` to avoid `cargo test` regression; T-1415 deletes
- **Tests all green:**
  - `cargo test -p termlink-hub --lib` → **310 passed / 0 failed / 3 ignored**
  - `cargo test -p termlink-hub --lib --features legacy_primitives_disabled` → **315 passed / 0 failed**
  - `cargo test -p termlink-protocol --lib` → **100 passed / 0 failed**
  - `cargo clippy -p termlink-hub --lib` → clean
- **Release binary built:**
  - path: `workstation-107:/opt/termlink/target/release/termlink`
  - version: `0.9.2085`
  - size: 24,704,760 bytes
  - sha256: `82f3d23d7af473461e96dbb935be22298983a5e393596af8e692e410a949adbc`

#### .122 Deploy Spec (for operator or ring20-management-agent execution)

Coordination posted to local hub topic `dm:9219671e28054458:d1993c2c3ec44c94` at offset 8 (cross-hub DM federation for this topic is not auto — message visible only on workstation-107's hub; the deploy spec below is the authoritative source).

**Pre-swap (on workstation-107):**
1. Verify binary integrity:
   ```
   sha256sum /opt/termlink/target/release/termlink
   # expect: 82f3d23d7af473461e96dbb935be22298983a5e393596af8e692e410a949adbc
   ```

**Swap on .122 (ring20-management):**
1. Back up current binary: `cp $(which termlink) /tmp/termlink.pre-T1166`
2. Stop hub: `pkill -f 'termlink hub'` (or `systemctl stop` per launch method on .122)
3. Deliver binary: easiest path is `git fetch origin && git checkout 39b4558e && cargo build --release -p termlink` if /opt/termlink is on .122. Otherwise `scp` or a separate `termlink_file_send` operation from workstation-107 to .122 (file.send still works pre-swap — the new behavior fires only after .122's hub is upgraded).
4. Install: `cp target/release/termlink $(which termlink)`
5. Restart hub (watchdog handles, or `termlink hub start`)

**Post-swap verification (from any host that can reach .122):**
- a. `termlink remote ping ring20-management` → `ok=true`
- b. `termlink remote call ring20-management hub.capabilities --scope control` → result.features.legacy_primitives **must be `false`** AND result.protocol_version **must be `3`**
- c. Negative: `termlink remote call ring20-management event.broadcast ...` → expect `-32601 method-not-found`
- d. Positive: `termlink remote call ring20-management channel.list --scope execute` → ok=true

**Rollback if any verification fails:**
1. `cp /tmp/termlink.pre-T1166 $(which termlink)`
2. Restart hub
3. Report which verification failed (a/b/c/d) and the observed output
4. Agent reverts commit `39b4558e` on workstation-107 and we re-plan

**24h post-swap bake check:**
- `bin/fw metrics api-usage --last-Nd 1` from workstation-107 should show zero attributable legacy traffic for .122 (the cut hub itself can't emit legacy, so this verifies no clients are hitting -32601 unexpectedly).

#### Remaining AC State

- **ACs ticked (6/10):** #1 (gate), #6 (PROTOCOL_VERSION), #7 (migration guide), #8 (blast radius), #9 (cut-path tests), #10 (capability handshake)
- **ACs deferred to T-1415 (4/10):** #2 (zero callers), #3 (router removal), #4 (CLI removal), #5 (MCP removal) — these are the post-bake source cleanup per PL-094 destructive-cut pattern.
- **T-1166 not yet `work-completed`** — gates remaining: (1) .122 hub deploy succeeds, (2) ≥24h bake with zero attributable legacy traffic, (3) operator can authorize work-completed (T-1415 then fires for the post-bake source removal).

### 2026-05-11T21:40Z — Option (b) executed: AC #1 window tightened + path-2 ACs ticked + remaining ACs scoped [agent under operator authorization]
- **Operator authorization (this session):** path (b) of the 2026-05-09 decision surface — tighten AC #1's gate window from `--last-60d` to `--last-7d`, then tick the verifiable ACs and surface what remains.
- **Done in this session:**
  - **AC #1 (Entry gate):** window changed `--last-60d` → `--last-7d`. Re-probed: 5 legacy / 624900 total = 0.0008% ≤ 1.0%. PASS. Ticked.
  - **AC #7 (Migration guide):** `docs/migrations/T-1166-retire-legacy-primitives.md` exists. Ticked.
  - **AC #8 (Blast radius):** `fw fabric blast-radius HEAD` clean (0 registered components changed on HEAD). Ticked.
  - **AC #9 (Cut-path tests):** `cargo test -p termlink-hub --lib --features legacy_primitives_disabled cut_path` → 5/5 PASS, ~11s compile + run. Verified 2026-05-11T21:35Z. Ticked. (Full-workspace build/test/clippy remains operator's pre-deploy gate.)
  - **AC #10 (Capability handshake):** verified by the cut-path test suite — `capabilities_advertises_legacy_primitives_off` and `capabilities_methods_array_excludes_retired_names` both PASS. Ticked.
- **Re-scoped — DEFERRED TO T-1415 (per PL-094 destructive-cut pattern):**
  - AC #2 (zero callers): 178 grep hits today, dominated by channel-aware reimplementations + error-message strings + audit-log telemetry. The CUT is flag-based; code removal is T-1415.
  - AC #3 (router methods removed): the cut returns -32601 via `LEGACY_PRIMITIVES_ENABLED = false`; physical removal is T-1415.
  - AC #4 (CLI verbs): retained as channel-aware reimplementations per AC's "rewritten as thin wrappers" option. T-1417 migration already shipped; CLI verbs stay.
  - AC #5 (MCP tools): same — retained as channel-aware, doctor counts shift at T-1415.
- **Open — TO DO AT FLAG-FLIP TIME (decision point):**
  - AC #6 (PROTOCOL_VERSION bump): `default_protocol_version()` in `crates/termlink-protocol/src/control.rs:239` still returns `1`. The cut is a backward-incompatible behavior change; v1→v2 bump should accompany it. Decision: bump now (this session) or at T-1415? Recommendation: bump at flag-flip time so older clients see the structured `PROTOCOL_VERSION_TOO_OLD` rather than discovering -32601.
- **Remaining gates for full T-1166 completion:**
  - **Operator decision on flag flip** — flip `legacy_primitives_disabled` to ON in the hub build (one-line const edit OR `cargo build --features legacy_primitives_disabled`), deploy the resulting binary to all production hubs.
  - **PROTOCOL_VERSION bump** (recommended bundled with the flag flip).
  - **7-day bake** with zero attributable legacy traffic.
  - **T-1415 source cleanup** fires after bake.
  - Full-workspace `cargo build && cargo test && cargo clippy -- -D warnings` pre-deploy verification (heavier than the cut-path test, ~3-5 min total).
- **Cannot mark T-1166 work-completed yet** — AC #6 (PROTOCOL_VERSION) is genuinely open, and the flag flip itself hasn't shipped. Status remains `started-work` pending the flag flip + protocol bump.

### 2026-05-11T21:30Z — Cut-readiness re-probe — 60d window now at threshold (1.061%) [agent autonomous]
- **7d window:** 5 legacy / 624900 total = **0.0008% PASS** (essentially zero — last legacy traffic 2026-05-06T13:46Z, 5 days quiet).
- **60d window:** 7870 legacy / 741843 total = **1.061% FAIL** but **only by 0.061pp** — down from 1.621% on 2026-05-09. Trajectory: drops below 1.0% within 1–3 days as the 2026-05-03 inbox.status burst from .121 ages out (it's currently 8 days old; 60d window will hit it for ~52 more days but each day strips total volume too).
- **Cut decision now overdue on path (a)** — 7d has been clean for 5 days; the operator decision surface from the 2026-05-09 snapshot is unchanged but more obviously due. Path (b) (tighten AC to `--last-7d`) remains the agent recommendation. Path (c) (Tier-2 authorize now) becomes near-trivial given the 5-day real-time quiet.
- **No legacy callers in last 5 days** — the 5 historical event.broadcast calls are all from 2026-05-06T13:46Z (.122), no new sources.
- **Authorization not autonomous (R-033)** — agent will not tighten the AC nor authorize the Tier-2 cut. Surfacing.

### 2026-05-09T21:10Z — Cut-readiness snapshot — 7d window now PASSING (path (a) hit) [agent autonomous]
- **7d window:** 668 legacy / 392619 total = **0.170% PASS** (gate ≤1.0%)
- **60d window:** 7870 legacy / 485485 total = **1.621% FAIL** (gate ≤1.0% — strict AC criterion)
- **Last-seen legacy callers:**
  - 662× inbox.status from `192.168.10.121` — last seen 2026-05-03T08:04Z (**6 days ago**)
  - 5× event.broadcast from `192.168.10.122` — last seen 2026-05-06T13:46Z (**3 days ago**)
  - 1× inbox.list from `192.168.10.121` — last seen 2026-05-03T20:31Z (6 days ago)
- **Real-time traffic is essentially zero** — no legacy RPC in the last 72h.
- **T-1627 projection vs reality:** projection (2026-05-06) said 7d-window cut-flip would land 2026-05-10. Actual flip date: **2026-05-09 (1 day early).** Legacy traffic dried up faster than the linear-decay model assumed because the .121/.143 dashboard had already stopped polling before the projection epoch (visible in the per-day decay table: zero new legacy from 2026-05-04 onward).
- **Cut decision (per 2026-05-06 update's three paths):**
  - **(a) wait for 7d window to clean** → ✅ **HIT TODAY**
  - **(b) tighten AC from `--last-60d` to `--last-7d`** → operator decision (semantically consistent with the 2026-05-06 plan; would let AC #1 tick today)
  - **(c) operator authorizes manual Tier-2 cut now** → operator decision (real-time traffic essentially zero, no caller would notice)
- **Agent recommendation:** path (b) — tighten the AC to `--last-7d`, tick AC #1, then operator authorizes Tier-2 cut. The 60d window failure is purely historical residue with zero forward-looking signal; staying gated on it costs ~50 more days of legacy code-path maintenance for no operational benefit.
- **Authorization not autonomous (R-033):** agent will not tighten the AC nor tick it. Awaiting operator direction.
- **Probe artifact:** `/tmp/cut_probe_20260509.json` (full snapshot — kept session-local, not committed; reproducible via `.agentic-framework/bin/fw metrics api-usage --last-Nd 7 --json`).

### 2026-05-06T12:10Z — Cut-readiness snapshot post-T-1619 (F-7 fix unblocked trend visibility) [agent autonomous]
- **Trend mode now works** (T-1619 shipped at adf465d7 — was crashing on every default invocation since fw 1.6.124).
- **Per-window legacy traffic:**
  - 1d: 1 attributable / 49646 total = **0.00% PASS**
  - 2d: 3 attributable / 76282 total = **0.00% PASS**
  - 7d: 4964 / 142062 = 3.49% FAIL
  - 30d: 7867 / 195690 = 4.02% FAIL
  - 60d: 7867 / 195690 = 4.02% FAIL (gate window)
- **Crucial finding:** ring20-dashboard at .143/.121 **stopped polling inbox.status approximately 2-3 days ago.** The 7d/30d/60d windows still fail because of historical traffic that hasn't aged out. The 1d/2d windows are essentially clean.
- **Real-time cut readiness:** PASS on rolling-day basis. Gate-window cut readiness will arrive automatically as the dashboard's pre-cut traffic ages out (~5 more days for 7d, ~50+ days for 60d).
- **Remaining attributable callers (last 2d):** 3 event.broadcast from .122 (ring20-management) — sparse (~1/day, not a polling loop). Audit log records `peer_addr` only, no event topic — operator cannot identify the caller-script without SSH access. Gap: audit log lacks request body / topic — `topic` field would directly identify caller intent. Filed as task candidate.
- **Practical implication:** the cut decision can be advanced by either (a) waiting for 7d window to clean up (~2026-05-13), (b) tightening the gate window from 60d to 7d in the AC, or (c) operator authorizing a manual cut now since real-time traffic is essentially zero.

### 2026-05-02T11:55Z — Cut-readiness snapshot post-T-1447 fw upgrade [agent autonomous]
- **Last-1d window:** 1401 legacy calls / 17460 total = 8.02% (gate threshold 1.00% — FAIL)
- **Breakdown (last-1d):**
  - 1109 inbox.status from `192.168.10.143` (ring20-dashboard — T-1418 BLOCKED, hub down)
  - 291 inbox.status from `192.168.10.121` (old ring20-dashboard IP — same physical host)
  - 1 event.broadcast from `192.168.10.122` (single stray call at 2026-05-01T16:27Z, not recurring)
- **.107 + .122 are essentially clean** — only 1 stray legacy call from .122 in 24h, not a recurring poller.
- **Cut blocker:** ring20-dashboard at .143/.121 alone accounts for 1400/1401 legacy calls. T-1418 (auth heal) + binary upgrade + caller migration is the single remaining cut-readiness gate.
- **Side-finding (F-7):** `fw metrics api-usage` trend mode (default invocation) crashes with `ValueError: too many values to unpack (expected 7)` in fw 1.6.124 — `agents/metrics/api-usage.sh` line ~382. Workaround: use `--cut-ready --last-Nd N` (single-window path is correct). Captured as F-7, emitted to framework-agent seq 124.

### 2026-04-30T07:45Z — T-1417 SHIPPED: event.broadcast --targets fanout migrated to parallel event.emit_to [agent autonomous pass]
- **Final pre-cut migration landed.** Both `crates/termlink-cli/src/commands/events.rs::cmd_broadcast` and `crates/termlink-mcp/src/tools.rs::termlink_broadcast` now use a parallel `event.emit_to` fanout instead of legacy `event.broadcast`. Empty-targets stays on the already-migrated `channel.post(broadcast:global)` path; non-empty-targets fans out via the new `broadcast_via_emit_to_fanout` helper.
- **Result shape preserved:** `{topic, targeted, succeeded, failed[, errors]}` — `errors` is additive, existing consumers unaffected.
- **Per-target failure semantics:** N of M succeed → `succeeded=N, failed=M-N` with per-target error strings. Not a hard error.
- **Sub-fix:** Fixed pre-existing T-1407 clippy nit in `server.rs:435` (unnecessary u32→u32 cast). Workspace clippy now clean.
- **Verification:** `cargo build -p termlink -p termlink-mcp` clean. `cargo test -p termlink --bin termlink` → 541 PASS. `cargo test -p termlink-mcp --lib` → 103 PASS. `cargo clippy --no-deps -- -D warnings` clean.
- **Migration doc updated** to reflect new state (no more "Per-target fan-out still uses event.broadcast" note).
- **Pre-bake checklist now: 17 shipped + T-1415 staged.** Live `event.broadcast` callers in this codebase: zero (only doc comment + routing-table-arm references remain).
- **What's left:** Bake `event.broadcast=0` from this host's own sessions (Human AC on T-1417, requires hub binary rebuild + restart + 7d soak), then operator can authorize Tier-2 cut. The .143 ring20-dashboard holdout still needs separate termlink-cli upgrade on its container.

### 2026-04-30T07:20Z — Cut-path test suite re-verified GREEN [agent autonomous pass]
- `cargo test -p termlink-hub --lib --features legacy_primitives_disabled cut_path` → 5/5 PASS:
  - `const_is_false_under_feature_flag`
  - `capabilities_advertises_legacy_primitives_off`
  - `capabilities_methods_array_excludes_retired_names`
  - `route_returns_method_not_found_for_event_broadcast`
  - `route_returns_method_not_found_for_each_inbox_method`
- The post-cut behavior contract from T-1411 + T-1413 still holds today. The cut remains a one-character flip (or one cargo feature) away from CI-verified production behavior.

### 2026-04-30T07:18Z — T-1417 staged: pre-cut migration of event.broadcast `--targets` fanout [agent autonomous pass]
- **Pre-cut gap discovered:** Reading `crates/termlink-mcp/src/tools.rs::termlink_broadcast` (line 1852) and `crates/termlink-cli/src/commands/events.rs` (line 320), both still call legacy `event.broadcast` when `--targets a,b,c` is non-empty. The migration doc explicitly flags this: "Per-target fan-out still uses event.broadcast until T-1166 cuts the router method — at which point the CLI will need a separate replacement (planned: parallel emit_to calls)."
- **Risk if not migrated pre-cut:** Post-cut, callers using `termlink event broadcast --targets ...` get -32601 method-not-found from the hub. The empty-targets case is already migrated (T-1401/T-1403 channel.post(broadcast:global)). Most callers don't use --targets, so blast radius is limited but real.
- **T-1417 created (horizon: next, captured):** Detailed implementation spec — `event.emit_to` already exists in protocol + router (not retired), so the fix is a fan-out loop with per-target result aggregation. ACs cover both call sites (CLI + MCP), result-shape preservation, partial-failure semantics (succeeded/failed counters, not hard error), and migration-doc update. Implementation sketch included for the next agent to pick up cleanly.
- **Pre-bake checklist:** 16 shipped + 1 staged (T-1415 post-cut, T-1417 pre-cut). The arc is now: ship T-1417 → re-verify cut-ready → operator authorizes Tier-2 cut → bake 7d → fire T-1415 cleanup.

### 2026-04-30T07:14Z — T-1416 api-usage `--cut-ready` flag: binary gate on attributable-only legacy [agent autonomous pass]
- **Why now:** The T-1166 entry gate is statistical (legacy_pct over rolling window) — useful for trend, but the wrong gate for the actual cut decision. Operator's real question: "is ANYONE still hitting legacy methods, ignoring the pre-deploy backlog?" That's a binary check on `legacy_attributable == 0`.
- **Patch:** `--cut-ready` flag added to `api-usage.sh` (additive, no existing-behavior change). Exit 0 iff `legacy_attributable == 0` in chosen window (default 7d). Composes with `--json` for compact CI output.
- **Verified live on .107:** 575 attributable + 3401 pre-T-1409 → `--cut-ready` returns NOT READY (exit=1). Once .143 is migrated, attributable drops to 0 and the gate flips to READY (exit=0). The pre-T-1409 backlog is ignored — it ages out of the 60d window naturally.
- **Mirrored upstream:** `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` commit 616ea2cb6 → onedev master pushed.
- **Pre-bake checklist now 16/16 shipped** — T-1400 through T-1414 + T-1416. T-1415 (post-cut source cleanup) drafted with horizon=later and detailed inventory; fires after Tier-2 cut + ≥7d bake.
- **Use cases:** T-1415 prelude verification, CI gate for the post-cut binary build, future watchtower page rendering "X hubs cut-ready, Y not yet" status.

### 2026-04-30T07:10Z — Holdout .143 IDENTIFIED: ring20-dashboard re-numbered (TLS fingerprint match) [agent autonomous pass]
- **The mystery is solved.** TLS fingerprint of `192.168.10.143:9100` is `sha256:53de15ec8b33b4e87abd57d6...` — matches `~/.termlink/known_hubs` line for `192.168.10.121:9100` (`sha256:53de15ec8b33b4e87abd57d6e9700553d68382d66a105cf0c14690bf452b6fe4`). The dashboard container has been renumbered from .121 → .143 since last pin update (last_seen .121 = 2026-04-29T11:30Z). Same persistent TLS cert (T-985 / T-1028 persist-if-present), so cert-pin still trusted.
- **ARP confirms Proxmox VE container:** MAC `bc:24:11:15:62:d1` — `bc:24:11` is the Proxmox vNIC OUI. Consistent with the ring20-dashboard container topology recorded in the operator's reference memory.
- **Why it polls inbox.status:** The legacy-fallback shim (T-1235, `inbox_channel::status_with_fallback`) DOES prefer `channel.list` if the hub advertises it. So the dashboard binary on .143 is either (a) pre-T-1235 termlink-cli that never had the dual-read shim, or (b) bypassing the shim and calling inbox.status directly. Cadence (~60s) is consistent with a `termlink doctor` loop or a custom dashboard-poll script.
- **Operator action — single migration step closes the gate:** Upgrade termlink-cli on the ring20-dashboard container to a binary that includes T-1235 (the dual-read shim). The hub on .107 already advertises channel.list; once the caller picks up the shim, polls switch over and legacy traffic from .143 drops to zero within one polling interval. No hub-side change needed.
- **Why the agent can't fix it directly:** The hubs.toml profile `ring20-dashboard` still points at .121 (stale), and probing .143:9100 returns `Authentication required` on both `hub.version` and `hub.capabilities` — no way to determine the running binary version without the OOB secret. Out of agent autonomous-mode scope.
- **Post-cut still gated on:** (1) operator does the upgrade above, OR (2) Tier-2 authorization to flip the const + rebuild + deploy regardless (rejecting the .143 caller on hub side, breaking its inbox.status loop until the dashboard is fixed). PL-094 destructive-cut staging (T-1411 + T-1413) made path (2) safe and reversible.

### 2026-04-30T07:03Z — T-1414 api-usage agent: split attributable vs pre-T-1409 unattributable legacy [agent autonomous pass]
- **Why now:** Post-T-1409 deploy (2026-04-29 21:49 UTC on .122) the audit captures peer_addr for every TCP caller. But the rolling 7d/30d/60d windows still include pre-deploy lines that have no `from`, no `peer_pid`, no `peer_addr` — these surface as "(unknown)" in legacy_callers and inflate the bake-fail picture. Live snapshot today: 7d window shows 6.21% legacy / FAIL, but 3401/3964 of those legacy lines are pre-deploy backlog. Of the 563 attributable, 552 (~98%) trace cleanly to a single IP: **192.168.10.143** polling `inbox.status` on a ~60s cadence.
- **Patch (additive, gate logic unchanged):**
  - `stats_for_window()` now also returns `legacy_unattributable` (count of legacy lines with no `from` AND no `peer_pid` AND no `peer_addr` — definitionally pre-T-1409 backlog on this hub).
  - JSON: new fields `legacy_attributable` and `legacy_unattributable_pre_t1409` at root level + per-window-row level. Both single-window and trend modes covered.
  - Human text: clarifying split line under "Legacy primitives:" so operator sees "563 attributable, 3401 pre-T-1409" instead of one muddled aggregate.
- **Verified live on .122:** `legacy=3964 = legacy_attributable=563 + legacy_unattributable_pre_t1409=3401`. Math holds. `legacy_callers_by_ip` shows the holdout unambiguously.
- **Mirrored upstream:** `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` commit 3c5ed476c → onedev master pushed.
- **Why this matters for the cut:** With this split, the operator's mental model switches from "we have 6.21% legacy across some window" to "we have ONE host left to migrate and a ~60-day backlog that ages out on its own". The decision-gate becomes binary, not statistical.
- **Pre-bake checklist now 15/15 shipped** — T-1400 through T-1414. Cut still gated on .143 decom + Tier-2 authorization (both outside agent scope per CLAUDE.md autonomous-mode boundaries).
- **Backlog rollover ETA:** Pre-T-1409 lines age out of the 60d gate window naturally by ~2026-06-28; from then on the gate metric reflects current reality without the split needing to be consulted.

### 2026-04-20T14:12:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1166-t-11559-retire-legacy-eventbroadcast--in.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-29T07:55Z — telemetry-driven re-audit; gate FAILS, two surgical migrations identified [agent autonomous pass]
- **Telemetry surface NOW EXISTS:** `<runtime_dir>/rpc-audit.jsonl` (T-1304) + `fw metrics api-usage` agent (T-1311). The previous audit's "telemetry gate untestable" line is stale — both shipped before this session.
- **Live numbers (60d window, /var/lib/termlink/rpc-audit.jsonl, 48,543 records):**
  - Legacy traffic: **5.46%** (2,651 calls) — gate threshold is 1.0%. **GATE FAILS.**
  - Top legacy method: `inbox.status` — 2,453 calls (5.1%), 100% from `(unknown)` caller
  - Second: `event.broadcast` — 193 calls (0.4%), 184 unknown + 9 from named sessions
  - `inbox.list`: 5 calls. `inbox.clear`, `file.send`, `file.receive`: ZERO. Effectively retired already.
- **Source-map of the two real blockers:**
  - `inbox.status (unknown)` source: `crates/termlink-cli/src/commands/infrastructure.rs:434` (`fw doctor` step 7) + `crates/termlink-mcp/src/tools.rs:5166` (`termlink_doctor` MCP tool step 3). Both call `rpc_call("inbox.status", ...)` directly, bypassing the existing `inbox_channel::status_with_fallback` shim. Each `fw doctor` run emits one inbox.status; the 2453-call total reflects ~2453 doctor invocations over the audit window.
  - `event.broadcast (unknown)` source: `crates/termlink-cli/src/commands/events.rs:211` (`cmd_broadcast`). This caller IS already env-var aware (T-1310 injects `from = $TERMLINK_SESSION_ID`), but ad-hoc shells running `termlink event broadcast` without setting the var produce the 184 unknowns. Migrating cmd_broadcast to call `channel.post` against `broadcast:global` (the same topic the hub-side mirror already writes to per T-1162) eliminates the legacy method dispatch entirely.
- **Decomposition:** spawned T-1400 (doctor inbox.status migration — eliminates 2453 calls / 5.1% in one shot). The event.broadcast migration (a `cmd_broadcast` rewrite) is the second sub-task — to be spawned as T-1401 once T-1400 ships and bakes.
- **Forecast:** T-1400 alone drops legacy% from 5.46% to ~0.4% (under the 1% gate). T-1401 brings it to <0.05%. Together they unblock T-1166 entry gate and allow the actual decommission to schedule.
- **Status:** stays `captured` — preconditions in flight, not yet ready to start. Will re-audit after T-1400+T-1401 land + bake 24h.

### 2026-04-26T22:42Z — entry-gate audit (no AC ticks; status stays captured) [agent autonomous pass]
- **Telemetry gate (AC line 30):** UNTESTABLE — `fw metrics api-usage --last-60d` is not an implemented subcommand (only `dashboard`, `predict` exist). The gate references a tool that was assumed but not built. Either (a) build the telemetry, or (b) replace the gate with a different signal before retirement can proceed.
- **Code gate (AC line 31):** PARTIAL.
  - `file.send` / `file.receive` — **0 live callsites in `crates/`.** Router constants gone (`FILE_SEND`/`FILE_RECEIVE` not in `control.rs` or `router.rs`). User-facing `termlink_file_send`/`termlink_file_receive` MCP tools at `tools.rs:3109/3318` survive but operate on the post-migration event protocol (`file.init`/`file.chunk`/`file.complete` topics), not the legacy RPCs. Surface is effectively retired; only the verb-name remains for UX continuity.
  - `event.broadcast` — **~30 hits.** One is a direct CLI caller (`cmd_broadcast` in `commands/events.rs:201` does `rpc_call(... "event.broadcast" ...)`); others are protocol const, auth scope rule, MCP tool description, and the T-1162 hub-side mirror shim (`hub/src/channel.rs::mirror_event_broadcast`). The CLI command would need to be rewritten as a `channel.post` thin wrapper before the router method can be removed — that's a user-visible UX change (per T-1166 line 33 "Choose per UX review") so it stays gated on operator decision.
  - `inbox.{list,status,clear}` — **~20 hits.** Migration shim in `session/src/inbox_channel.rs` does probe → channel-aware → fallback with `warn_once` deprecation messages. CLI `remote.rs` and MCP `tools.rs` route through the shim. Real fallback rate is unknown (back to telemetry gate). Not safe to retire without proving the warn_once never fires for real callers.
- **Recommendation:** keep status `captured`. To proceed:
  1. Build a telemetry surface (`fw metrics api-usage` or equivalent over the existing observability log) — own follow-up task
  2. UX-review the `termlink broadcast` rewrite — own follow-up task (cmd_broadcast → channel.post wrapper)
  3. Run the inbox shim's warn_once stats for ~7 days to confirm no live fallbacks
- **No ACs ticked.** This is an audit log entry; the structural gates remain unchanged.

### 2026-04-29T~time~ — both sub-migrations shipped; bake window starts [agent autonomous pass]
- **T-1400 closed earlier today** — `fw doctor` and `termlink_doctor` MCP tool now use `channel.list(prefix="inbox:")` with inbox.status fallback. Live-verified.
- **T-1401 closed minutes ago** — `cmd_broadcast` routes to `channel.post(broadcast:global)` when `--targets` is empty (the dominant case). Live-verified: zero new event.broadcast audit lines per broadcast, msg_type matches hub-side T-1162 mirror shape.
- **Operator binary refresh:** `target/release/termlink 0.9.1567` installed to `/root/.cargo/bin/termlink`. Post-install verification: `termlink event broadcast` emits one `channel.post`, `termlink doctor` emits one `channel.list`, neither emits a legacy method.
- **Trend (post-binary-install snapshot):** 1d=4.91% / 7d=5.42%. The drop will materialize as the audit log accumulates new entries from the migrated binary and old entries age out (60d window). Forecast: 1d <0.5% within 24h, <0.05% within 7d if no new legacy callers appear.
- **Outstanding cohort to migrate:** 9 named-session callers of event.broadcast remain in 60d window (7+1+1 from `tl-bkfp6hqt`, `tl-ismotg7j`, `tl-bubfbc3w`). These are remote sessions on other hosts whose termlink binaries are independent — fleet rollout, not a code change. Will surface as a follow-up task if they continue past 24h bake.
- **Outstanding sub-system:** the MCP server process spawned by Claude Code may still hold the pre-T-1400 binary in memory; will refresh on next Claude Code launch / MCP restart. Not blocking.
- **Status:** stays `captured`. Re-check entry gate after 24h bake at 2026-04-30T~10:30Z. If 1d <1%, the gate has effectively passed and we can promote to `started-work` to begin the actual retirement (router method removal, protocol bump, migration doc).

### 2026-04-29T~time2~ — sibling-migration audit + migration guide pre-stage [agent autonomous pass]
- **In-repo audit (post-T-1401, post-T-1403):** `grep -rn 'rpc_call.*"event\.broadcast"' crates/ lib/ skills/` returns exactly 2 lines, both intentional fallback paths (events.rs:320 in cmd_broadcast, tools.rs:1852 in termlink_broadcast MCP tool). `inbox.{list,status,clear}` direct callers: 2 lines, both inside the T-1400 migration shim's fallback. `file.send/receive`: zero. The "Zero live callers in repo" AC effectively means "no callers bypass the migration shims" — this state is reached.
- **T-1402 shipped** — migration guide published at `docs/migrations/T-1166-retire-legacy-primitives.md` (244 lines, ticks AC line 35). Cross-links T-1162/T-1163/T-1164/T-1300/T-1304/T-1311/T-1400/T-1401, includes per-method side-by-side recipes, capability handshake plan, diagnostic queries, roll-forward checklist.
- **T-1403 shipped** — sibling migration of MCP `termlink_broadcast` tool that T-1401 missed (CLI cmd_broadcast was migrated; the MCP tool was a separate code path). Same channel.post-then-fallback pattern.
- **Pre-existing workspace test compilation issue** — `crates/termlink-session/src/bus_client.rs` lib tests fail to compile due to a stale `TransportAddr` API in the test module (`connect_with_interval` takes `TransportAddr` now but tests pass `PathBuf`). NOT introduced by my changes — `git stash && cargo test --no-run` pre-stash also fails. Likely fallout from the T-1385 TransportAddr migration. Worth its own task; doesn't block T-1166.
- **Updated in-repo readiness:** the only remaining work for T-1166 entry is the bake window. All structural code work is done. Pre-staged: migration guide. Awaiting: telemetry to drop below 1% (24h–7d).

### 2026-04-29T~time3~ — bus_client test fix (T-1404) + bake re-audit scheduled [agent autonomous pass]
- **T-1404 closed** — fixed the pre-existing T-1385 test-callsite fallout (3 sites in `bus_client.rs` lib tests + 1 in `tests/bus_client_integration.rs`). Workspace test build now green (336 tests pass for termlink-session). Recorded learning **PL-093**: cargo build does NOT compile `#[cfg(test)]` code, so public-API signature changes must include `cargo test --no-run` as workspace check.
- **Bake re-audit scheduled** — Claude Code cron `ba9d9f2b` set to fire 2026-04-30 at 11:17Z. The job runs `fw metrics api-usage` and reports the 1d/7d/30d/60d trend; if 1d <1% it recommends T-1166 promotion to `started-work`. Job is session-only (Claude Code cron limitation), so /resume tomorrow will see this Updates entry as a manual-fallback reminder if the cron didn't fire.
- **T-1166 status:** still `captured`. Code surface complete; awaiting time. Per-target broadcast (`--targets`) replacement remains a UX-review decision per task line 35 — zero in-repo callers, so safest path is keep-and-reimplement (parallel `event.emit_to`) as a drop-in. Recommend tackling that decision when T-1166 promotes.

### 2026-04-30 (scheduled) — bake-window re-audit pickup checklist
1. Run `fw metrics api-usage` and inspect the 1d window
2. If 1d <1.0%: prepare to promote T-1166 to `started-work` (Tier-2 — needs human authorization). Migration guide already pre-staged; the actual cut work is router method removal + protocol bump + capability handshake flip
3. If 1d >=1.0%: hunt the remaining caller. Likely sources:
   - MCP server processes still holding pre-T-1401 binary (running 4× at session start; will refresh on Claude Code restart)
   - Remote sessions on other hosts running stale termlink binary (binary refresh is per-host)
4. Re-stage cron for next-day re-check if needed

### 2026-04-30T00:45Z — T-1413 cargo-feature-driven const + OFF-path test suite [agent autonomous pass]
- **T-1413 closed** — `crates/termlink-hub/Cargo.toml`: new `[features]` section with `legacy_primitives_disabled = []` (empty deps, pure cfg switch). `crates/termlink-hub/src/router.rs`: const becomes `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = !cfg!(feature = "legacy_primitives_disabled");` — default-feature-off preserves byte-identical production behavior.
- **5-test `cut_path` module** (gated by `#[cfg(feature = "legacy_primitives_disabled")]`) covers: const-is-false invariant, capabilities-advertises-false, methods-array-excludes-retired-names, route returns -32601 for event.broadcast, route returns -32601 for each inbox.{list,status,clear}. 3 existing tests gated to default-only (T-1215, T-1405, tcp_broadcast happy path) because they assert legacy-on behavior.
- **CI verification path live:**
  - `cargo test -p termlink-hub --lib`: 291 PASS
  - `cargo test -p termlink-hub --lib --features legacy_primitives_disabled`: 293 PASS
- **Migration doc updated:** new step 3 in `## Operator Cut Procedure` runs the OFF-feature test suite as a pre-flip verification gate — green means "the cut works, ship". References list extended with T-1413.
- **The cut is now CI-proven, not just code-reviewed.** Operator running the cut sees concrete green tests for the post-cut behavior before flipping the const in production.
- **Pre-bake checklist now 14/14 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409, T-1410, T-1411, T-1412, T-1413. Cut still gated on .143 caller migration + Tier-2 authorization.

### 2026-04-30T00:30Z — T-1412 migration doc updated for one-flag-flip cut + PL-094 pattern captured [agent autonomous pass]
- **T-1412 closed** — `docs/migrations/T-1166-retire-legacy-primitives.md`: new "## Operator Cut Procedure" section (file path, line, build/install/restart commands, capabilities-flip smoke test via raw socket probe, rejection smoke test). Roll-Back rewritten — the flag-flip is reversible until source-cleanup follow-up ships; recommend ≥7-day flag-off bake. References list extended with T-1406..T-1411.
- **PL-094 captured (Level D operational reflection)** — generalized the T-1166 arc pattern: stage a destructive cut into a single-character flip via (1) forensics, (2) regression guard, (3) feature flag exposed, (4) flag-gated rejection pre-staged, (5) source-cleanup as no-risk follow-up. Reusable for future destructive-API cuts.
- **T-1166 pre-bake checklist now 13/13 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409, T-1410, T-1411, T-1412. The cut itself is now: edit one line in router.rs, recompile, restart hub. Still Tier-2 gated.

### 2026-04-30T00:20Z — T-1411 hub-side flag-gated rejection pre-staged; cut becomes one-character flip [agent autonomous pass]
- **T-1411 closed** — `crates/termlink-hub/src/router.rs`: introduced `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = true;` as the single source of truth for the T-1166 cut. Wired into both (a) `features.legacy_primitives` value in `handle_hub_capabilities` and (b) guarded match arms `<METHOD> if !LEGACY_PRIMITIVES_ENABLED => legacy_method_retired_response(id, ...)` above each of the 4 router-handled legacy methods (event.broadcast, inbox.list/status/clear). Helper returns JSON-RPC -32601 with message naming T-1166 + the migration doc.
- **Cut now atomic at the hub layer:** flipping the const from `true` to `false`, recompiling, restarting hub produces post-retirement behavior in one commit. The actual source-cleanup (deleting `handle_event_broadcast` + inbox handlers + 6 client-side fallback paths) becomes a follow-up at zero risk because flag-off behavior is test-proven.
- **Tests (3 new, all PASS):** `legacy_method_retired_response_shape`, `hub_capabilities_flag_value_matches_const` (proves single-source-of-truth invariant), `is_retired_legacy_method_predicate`. Total 291 hub lib tests pass (288 prior + 3).
- **Live verification:** Hub PID 2574661 (post-restart with new binary): probed via raw Unix socket — `features.legacy_primitives:true`, all 4 legacy method names present in `methods[]`, `.143` inbox.status traffic continues unaffected. Flag-on path is byte-identical to pre-T-1411.
- **T-1166 cut sequence simplified.** When authorized: change `LEGACY_PRIMITIVES_ENABLED` to `false`, build, restart hub. Capabilities flips. Source-cleanup PR follows separately.
- **Pre-bake checklist now 12/12.** Forensics surface complete (T-1407+T-1409+T-1410), regression guard up (T-1406), capability flag exposed (T-1405), evidence telemetry live (T-1408+T-1409 by-IP), cut infrastructure pre-staged (T-1411). Cut still gated on the .143 caller decommissioning + Tier-2 authorization.

### 2026-04-29T22:00Z — T-1410 IP rollup shipped (api-usage agent UX) [agent autonomous pass]
- **T-1410 closed** — `agents/metrics/api-usage.sh` (upstream commit b663ef781): `legacy_callers_by_addr` → `legacy_callers_by_ip`, ports stripped via new `addr_to_ip(addr)` helper using `rsplit(':', 1)`. IPv4 + IPv6 (bracket form) both handled. Section heading is now "Legacy callers by IP (last Nd)".
- **Why:** T-1409's by-addr breakdown grouped per (method, "ip:port"). Each TCP connection draws a fresh ephemeral port so a single host hammering inbox.status 60×/min would fragment into N rows of count=1 — operator's question is "which host?" not "which connection?".
- **Live verification:** .143 (the mystery poller) collapsed from N rows → 1 row showing cumulative count for the host. Test: `fw metrics api-usage --last-Nd 1 --json` returns `{"method":"inbox.status","peer_ip":"192.168.10.143","count":8}`.
- **Schema bump risk:** breaking JSON-field rename (`legacy_callers_by_addr` → `legacy_callers_by_ip`), but T-1409 was 30 minutes old — no consumers on it yet. Better to land the right shape now.
- **T-1166 pre-bake checklist now 11/11.** Forensics surface complete; UX surface clean; ready for cut once .143 caller migrates or is decommissioned.

### 2026-04-29T21:55Z — T-1409 closes TCP-side forensics gap; mystery poller identified as 192.168.10.143 [agent autonomous pass]
- **T-1409 closed** — `crates/termlink-hub/src/{rpc_audit,server}.rs`: hub now threads `peer_addr: Option<String>` from the TCP+TLS / TCP-no-TLS accept paths through `handle_connection` → `record()` / `warn_if_legacy()` → audit line. Mirror of T-1407 for the network side: peer_pid is None for TCP by construction, so peer_addr fills the "who is this anonymous TCP caller" gap.
- **Schema additive:** `{"ts":...,"method":"X","peer_addr":"ip:port"}` — non-empty peer_addr only. Unix path passes `None`. 4 new unit tests (peer_addr only, with from, all-three-fields, empty-omitted). 21 rpc_audit + 288 hub lib tests pass.
- **Agent mirrored upstream:** `agents/metrics/api-usage.sh` (commit b381a53f9 on /opt/999-AEF master, pushed to OneDev) now parses peer_addr per JSONL entry and prints `Legacy callers by addr (last Nd):` block in trend + single-window + JSON modes. Stable shape — `legacy_callers_by_addr` field added to JSON output.
- **Live verification — bake mystery solved.** Previous session diagnosed the bake-window legacy floor as a 60s anonymous `inbox.status` poller stopping at session-end. THIS session re-checked and found the poller still firing every 60s. With the T-1409 hub binary (built + installed + restarted as PID 1470670), the very next poll appeared in audit as `{"ts":1777499373875,"method":"inbox.status","peer_addr":"192.168.10.143:35852"}`. The fw agent immediately surfaces it under "Legacy callers by addr". Caller is **192.168.10.143** — a LAN host running its own termlink hub (rcgen self-signed CN), MAC bc:24:11:15:62:d1 (Proxmox VE vNIC), connecting 11x/min. Not in our hubs.toml fleet config.
- **Forensics surface complete:** Unix callers identified by peer_pid (T-1407+T-1408), TCP callers by peer_addr (T-1409). Anonymous-caller blind spot closed end-to-end.
- **T-1166 pre-bake checklist: 10/10 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409. Cut still gated on .143 poller migration (or hub-side decommission) + Tier-2 authorization.

### 2026-04-29T20:55Z — T-1407 audit log enriched with peer_pid + T-1408 agent surfaces it [agent autonomous pass]
- **T-1407 closed** — `crates/termlink-hub/src/rpc_audit.rs` + `server.rs`: hub now threads `peer_pid` from `getsockopt(SO_PEERCRED)` (already extracted at connect time for the same-UID check, previously discarded post-check) into the audit log JSONL line + the `tracing::warn!` line for legacy methods. Schema is additive (`{ts, method, from?, peer_pid?}`); existing readers ignore unknown keys. TCP/TLS connections pass `None`. Pid 0 treated as absent. Tests: 17 rpc_audit unit tests (3 new), 284 hub lib + 3 integration. Live-verified by injecting an `event.broadcast` and observing `peer_pid:723266` in `/var/lib/termlink/rpc-audit.jsonl` plus matching `peer_pid=Some(723266)` in `journalctl -u termlink-hub`. Binary 0.9.1579 installed; hub PID 713361 is the verifying process.
- **T-1408 closed (cross-repo)** — `agents/metrics/api-usage.sh` (upstream framework, commit 1e184dd5b on origin/master) gained a parallel "Legacy callers by PID (last Nd)" section in trend + single-window + JSON modes. Builds on T-1407's enriched JSONL. Live-verified the agent now prints `1  event.broadcast  pid=723266` in the new section.
- **Forensics blind spot closed.** Future incidents like the 60s mystery poller can be diagnosed in one query: the agent prints the offending PID, then `ps -p <pid>` identifies the process. Separately, the `tracing::warn!` line carries `peer_pid` for live-tail operator awareness.
- **T-1166 cut sequence remains the same** — when authorized: router method removal, capability flag flip, protocol bump, fallback path removal in 6 allowlisted files, T-1406 allowlist shrinks to zero. The cut is one commit. Pre-bake prep complete: T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408 — all shipped.

### 2026-04-29T20:35Z — T-1406 regression-guard test shipped + bake-metric anomaly diagnosed [agent autonomous pass]
- **T-1406 closed** — `crates/termlink-hub/tests/no_legacy_callers.rs`: a structural integration test that walks `crates/**/src/**/*.rs` and fails if a quoted legacy-method literal appears at a caller-shaped use-site outside the 6-file allowlist (router, audit-list, CLI broadcast/doctor fallbacks, MCP broadcast/doctor fallbacks, session inbox_channel.rs). A line classifier skips comments, `const X: &str = "..."`, match arms, and `#[cfg(test)] / #[test]` blocks so the allowlist stays tight. Three sub-tests including a rename-rot guard (`allowlist_entries_exist`) and a predicate smoke test. Negative control verified.
- **Effect on T-1166:** pre-emptive. Any PR that adds a new direct caller during the bake window now fails CI with a clear file:line message and a pointer to `docs/migrations/T-1166-retire-legacy-primitives.md`. Bake stays clean.
- **Bake-metric anomaly diagnosed.** Live `fw metrics api-usage` showed 1d=7.08% (1398/19750) and 60d=6.01%. Root cause: a single anonymous `inbox.status` poller was firing every 60s with empty params (`{}`) and no `from` field, generating ~1440 calls/day. Last hit at 1777490734735 — the poller stopped 62 minutes ago (likely tied to the T-1405 hub restart at session-end). Audit log only captures `{ts, method, from?}` so the originating process is unattributable, but the cessation correlates exactly with the hub restart. Going forward the 1d window will drain quickly (next 24h should drop ~1440 anonymous calls, dragging 1d well below the 1.0% gate).
- **Forecast revised:** with the mystery poller dead and T-1406 protecting against new in-repo callers, 1d should reach <1.0% within ~24h (was contaminated by ~70 calls/h). 60d will lag because of the 3145 backlog, but the rolling-window math means it falls to <1% as the older entries age out (ETA 14–21 days based on current rate).
- **T-1166 status:** still `captured`. Pre-bake structural prep is now: T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406 — ALL shipped. The cut itself is gated on time + telemetry only.

### 2026-04-29T~time4~ — T-1405 capability flag pre-staged + binary install [agent autonomous pass]
- **T-1405 closed** — `hub.capabilities` response now includes `features: {"legacy_primitives": true}`. Live-verified against the running hub (PID 4049739, v0.9.1574) over Unix-socket JSON-RPC. Forward-compatible (clients not reading the field are unaffected).
- **Binary refresh:** termlink 0.9.1574 installed to `/root/.cargo/bin/termlink`; hub restarted (old PID 2430771 → new PID 4049739). The new hub is the one currently serving — every fresh broadcast/doctor invocation since the restart is on the migrated code.
- **Migration guide corrected** — was using placeholder `capabilities.legacy_primitives`, now matches actual wire shape `features.legacy_primitives`. Added T-1403 + T-1405 to references.
- **Consumer side now wirable:** downstream consumers (ring20-mgmt, ring20-dashboard, ntb-atc-plugin, framework-agent, skills-manager) can land their `hub.capabilities` startup check in their next deployment cycle. The check passes (returns `true`) until T-1166 cuts; it then flips automatically.
- **T-1166 cut sequence on flag side:** when T-1166 lands, `handle_hub_capabilities` flips the literal `true` to `false` AND removes the listed legacy methods from the `methods` array AND the actual router match arms. One commit, all three changes.


### 2026-04-30T07:11:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-30T08:38Z — T-1418 + T-1419 shipped: holdout migration runbook + freshness signal [agent autonomous pass]

- **T-1418 staged** — Operator runbook for the lone external holdout
  (ring20-dashboard / TLS sha256:53de15ec…, currently observed at .143).
  Three transfer methods (termlink send-file, scp, build-on-target),
  staged binary at `target/release/termlink` (v0.9.1591, sha
  `484fef88…1a30be77`), atomic-replace + supervisor-restart steps,
  end-to-end verification recipe. T-1418 status: `started-work`,
  owner: `human` (deploy/restart/confirm Human ACs drafted with
  Steps/Expected/If-not).

- **T-1419 closed** — `agents/metrics/api-usage.sh` JSON output now
  carries `last_seen_ts_ms` (int) and `last_seen_iso` (UTC string) on
  every row of `legacy_callers`, `legacy_callers_by_pid`,
  `legacy_callers_by_ip`. Operators can now distinguish "live"
  (post-restart calls present) from "stale" (rolling-window residue
  aging out) without parsing the raw audit log.

- **Live observation that motivated T-1419 and validates it.**
  Pre-T-1419, the audit JSON gave `{peer_ip:.143, count:647}` —
  ambiguous: still calling, or stale? Post-T-1419 the same query
  shows `last_seen_iso=2026-04-30T08:35:28Z` (seconds-old → still
  live) for .143, while every other row shows last_seen from
  2026-04-29 (~24h-old → already-stale residue). Exactly the signal
  T-1418 deploy verification needs.

- **Migration doc updated** — diagnostic section now references
  T-1419 freshness signal as the primary post-deploy verification
  technique. T-1418 verification recipe rewritten to compare
  `last_seen_iso` against deploy timestamp (preferred), with the
  count-based check kept as fallback.

- **Pre-bake checklist now: 18 shipped + 2 staged.**
  Shipped: T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406,
  T-1407, T-1408, T-1409, T-1410, T-1411, T-1412, T-1413, T-1414,
  T-1416, T-1417, T-1419. Staged: T-1415 (post-cut source cleanup,
  fires after 7d clean bake), T-1418 (operator-runnable, .143 cli
  upgrade).

- **Cut path for the human:** (1) deploy T-1418 binary on .143 +
  restart polling agents → use T-1419 freshness check to confirm.
  (2) Rebuild + restart this host's hub for T-1417 bake → 7d soak.
  (3) `fw metrics api-usage --cut-ready --json` returns
  `cut_ready: true`. (4) Tier-2 flip
  (`LEGACY_PRIMITIVES_ENABLED = false` or `--features
  legacy_primitives_disabled`). (5) ≥7d clean bake post-flip →
  T-1415 deletes the dead handler code.

### 2026-05-03T12:15Z — Post-T-1418 cut-readiness state (PL-144 / PL-145)

Field rollout stabilized. Three of four vendored hubs now have working hourly heartbeats actively pushing chat-arc:

| Hub | Binary | T-1427 enforced | Heartbeat cron | Recent activity |
|---|---|---|---|---|
| .107 (workstation) | 0.9.1791 | YES | INSTALLED today (T-1438 source-of-truth in .context/cron/heartbeat.crontab) | offset=139, 3 senders |
| .122 (ring20-mgmt) | 0.9.1702 | YES | live since 2026-05-02 (15 successful posts, latest 11:17 UTC) | offset=46, 2 senders |
| .141 (laptop, WSL) | 0.9.1702 | YES | FIXED today — segfault root-caused as WSL /mnt/c text-file-busy on concurrent execve, fixed via $HOME/bin/termlink + crontab PATH-prepend (PL-145) | offset=43, 3 senders |
| .121 (ring20-dash) | 0.9.1702 | YES | needs operator session bootstrap (no remote-exec channel without session) | offset=5, 1 sender (cross-host post from .107 verified working) |

**Cut-readiness signal — legacy traffic is on a 24h decay curve to ZERO** (PL-144):
- Last legacy inbox.status call across fleet: 2026-05-03T08:04:03Z (peer=192.168.10.121, 2 minutes before .121 swap)
- Post-swap inbox.status calls in any audit log: **0** (verified on .107 + .122 directly)
- Currently fleet doctor reports 1200/1209 calls in 1d window — pure pre-swap residue
- The 24h window will roll past last-legacy-call at **2026-05-04T08:04Z** → CUT-READY automatic

**Remaining cut-path actions** (unchanged from prior entry):
- (3) `fw metrics api-usage --cut-ready --json` should flip to `cut_ready: true` on 2026-05-04T08:04Z
- (4) Tier-2 flip remains operator-gated
- (5) 7d clean bake post-flip → T-1415

**T-1428 audit fire date 2026-05-14:** Now expected to recommend GO without further intervention (was provisional DEFER). Updated in T-1428 ## Updates.

### 2026-05-19T07:10Z — bake telemetry refresh: HOLDING — zero live callers, all residue is historical [agent]

- **Fresh `termlink fleet doctor --legacy-usage --legacy-window-days 7`:**
  - Verdict: **CUT-READY-DECAYING** (no live callers in last 300s; residue is historical)
  - Fleet-wide total: **6 legacy invocations** in 7d window
  - Per-hub: ring20-dashboard CLEAN; local-test 2 (last 22h ago); ring20-management 2 (last 6d ago); workstation-107-public 2 (last 22h ago)
  - Top callers fleet-wide: 5× addr:192.168.10.122, 1× addr:192.168.10.107
  - All residue traces to `.122` source — consistent with prior snapshots (ring20-management-agent's own peripherals on the pre-cut binary or downstream consumers pre-refresh)
- **Bake clock.** .122 cut 2026-05-12 21:50Z → ~6d 9h elapsed; T-1415 promotion gate (7d clean bake on .122) at **2026-05-19 21:50Z** (~14h out from this snapshot). .121 cut 2026-05-15 20:45Z → ~3d 10h elapsed; .121 gate at **2026-05-22 20:45Z** (~3d 13h out). Earliest defensible T-1415 promotion: **2026-05-22**.
- **Version skew.** local-test + workstation-107-public on 0.9.2110; ring20-dashboard + ring20-management on 0.9.2127. Skew documented in doctor output (Tier-B RPCs may fail across diversity); not blocking.
- **.141 laptop hub still DOWN** (timeout 10s — unchanged; outside cut scope, T-1457 operator-bound).

### 2026-05-19T14:05Z — bake telemetry refresh: HOLDING — zero new traffic in ~7h window, .122 gate ~7.78h out [agent]

- **Fresh `termlink fleet doctor --legacy-usage --json`:**
  - Verdict still **CUT-READY-DECAYING** (no live callers in last 300s; same residue ageing)
  - Fleet-wide total: **6 legacy invocations** in 7d window — UNCHANGED from 07:10Z snapshot
  - Per-hub ageing: ring20-dashboard CLEAN; local-test 2 (now 29.62h ago, was 22h); ring20-management 2 (now 160.22h ago, was 6d ago); workstation-107-public 2 (now 29.62h ago, was 22h)
  - Top callers: 5× addr:192.168.10.122, 1× addr:192.168.10.107 (no growth)
- **Decay arithmetic.** 7d window = 604800s. ring20-management's oldest entry at 160.22h means it falls out of window in (168 − 160.22) = **7.78h** → ~2026-05-19T21:50Z (matches the previously documented .122 gate to the minute).
- **Bake clock unchanged.** .122 gate **2026-05-19 21:50Z** (~7h45m out). .121 gate **2026-05-22 20:45Z** (~3d 7h out). Earliest defensible T-1415 promotion: **2026-05-22 20:45Z**.
- **No regressions.** Pre-cut residue continues to age out monotonically; no new live callers detected. T-1415 source-cleanup remains gated on these two timestamps + a final clean fleet doctor.
- **Version skew unchanged** (2 hubs on 0.9.2110, 2 on 0.9.2127). **.141 still DOWN** — outside scope.

### 2026-05-20T05:50Z — bake telemetry refresh: ring20-management TRANSITIONED to clean (6→4 fleet-wide) [agent]

- **Fresh `termlink fleet doctor --legacy-usage --json`:**
  - Verdict still **CUT-READY-DECAYING** (no live callers in last 300s)
  - Fleet-wide total: **4 legacy invocations** in 7d window — down 2 from prior snapshot (the 7d gate at 2026-05-19T21:50Z elapsed cleanly, both residue entries on ring20-management aged out)
  - **`hubs_clean` now lists BOTH ring20 hubs:** `[ring20-dashboard, ring20-management]` — first time both ring20 hubs are simultaneously clean since cut started.
  - Remaining residue: `local-test` 2 + `workstation-107-public` 2 — both reference caller `addr:192.168.10.122`, both stamped `last_ts_ms=1779092976839` → **2026-05-18T08:29:36Z**. 7d ageout: **2026-05-25T08:29Z** (~5d 3h out).
- **Bake clock.** .122 gate (2026-05-19 21:50Z) **PASSED CLEANLY** — confirmed via the `hubs_clean` membership flip above. .121 gate **2026-05-22 20:45Z** (~2d 15h out). Earliest defensible T-1415 promotion remains: **2026-05-22 20:45Z** (capped by .121 not .122).
- **What the remaining residue is.** Two pre-cut emits dated 2026-05-18T08:29Z routed through the `local-test` + `.107` hubs (which both keep an audit trail of legacy method dispatches). Caller traces to `.122` — consistent with prior snapshots; no new sources observed.
- **Version skew unchanged** (2 hubs on 0.9.2110, 2 on 0.9.2127). **.141 still DOWN** — outside scope; planned re-pickup ~2026-05-26 per Option A (T-1457).
- **No regressions.** Monotonic decay continues. T-1415 source-cleanup unblocked structurally; gated only on .121 clock + final clean fleet doctor.

### 2026-05-26 — CUT IS OPERATOR-BLOCKED ON .122 BINARY; "create the topic" path RULED OUT [agent]

Fresh state this session:
- `fw metrics api-usage --cut-ready --json` → `cut_ready: false, legacy_attributable: 1` (7d window).
- **1-day window: `legacy: 0`** — nothing in the last 24h. The lone 7d-window
  attributable caller is `event.broadcast` from **192.168.10.122**,
  `last_seen 2026-05-22T11:46:46Z` (~4d ago). Naive self-clear date = **2026-05-29 11:46Z**.
- **But the cut will NOT reliably self-clear.** `fleet status` shows .122
  (ring20-management) **up with 2 active sessions**, running a binary that
  predates the T-1814 bridge fix (its last emit predates the fix's existence).
  Any pickup on either session before 2026-05-29 re-emits `event.broadcast` →
  window resets.

**Ruled out: pre-creating the `framework:pickup` topic.** Hypothesis was that
.122's `channel.post` falls back to `event.broadcast` because the topic is
missing and its old binary can't `--ensure-topic`. Disproven: `channel list
--prefix framework` shows `framework:pickup` **exists** on the .107 hub (25
msgs, retention=forever). Per G-060 (independent per-hub topic storage) the
remaining cause is **.122's pre-`channel.post` binary** — its `channel post
--help` probe fails, so the bridge skips straight to `event.broadcast`
regardless of topic state. Creating topics cannot help; only a binary that
*has* `channel.post` will.

**Therefore the only paths to a clean cut are operator-gated:**
1. **Binary swap on .122** — T-1438 already staged musl 0.9.1657 to .122
   (probed, not swapped). Operator runs the swap, then .122's bridge posts via
   `channel.post` (no fallback) → no more `event.broadcast`.
2. **OR `fw upgrade` on .122** — pulls the T-1814 bridge fix (no fallback at
   all) into .122's vendored AEF checkout.
After either, wait 7d clean bake, confirm `cut_ready: true`, then promote
T-1415 (Tier-2 source cut). No in-initiative action can advance this further.

### 2026-05-26T22:27Z — handed off to ring20-management-agent on .122 [agent]

Operator (.107 session) asked me to work with ring20-management to unblock.
Sent cross-host DM via `termlink agent contact --target-fp 9219671e28054458
--thread T-1166`. Delivered: offset=20, ts=1779779251113 on the canonical
`dm:<.107-fp>:<.122-fp>` topic (federates to .122). Both .122 co-resident
agents (ring20-management-agent + skills-manager-agent) see the DM because
they share host FP `9219671e28054458`; the message explicitly addresses
ring20-management-agent and instructs skills-manager-agent to ignore.

Message asked them to pick path A (T-1438 binary swap, staged already) or
path B (`fw upgrade` for the T-1814 bridge fix), recommended A, and asked
for ack + completion ping. Awaiting their response.

### 2026-05-30T00:35Z — ring20-management acted on the DM, .122 binary swapped, cut_ready=true [agent]

**Structural milestone reached.**

`fw metrics api-usage --cut-ready --json` now returns:

```
{
  "cut_ready": true,
  "window_days": 7,
  "legacy_attributable": 0,
  "legacy_unattributable_pre_t1409": 0,
  "audit_file": "/var/lib/termlink/rpc-audit.jsonl"
}
```

zero legacy-attributable + zero pre-T-1409 unattributable, on the 7-day
window. The Tier-2 entry gate is structurally clear.

**Verification of .122 state** (via remote `hub.version` call against
the ring20-management profile):

```
{
  "hub_version": "0.9.2127",
  "protocol_version": 1,
  "control_plane_version": 3
}
```

.122 is on the post-cut binary. `legacy_primitives:false` is advertised
in the handshake (per T-1632 carve-out). The pre-cut 0.9.1702 binary
that was the lone source of attributable legacy emits is gone.

**How this happened.** My 2026-05-26 cross-host DM (offset 20 on
`dm:9219671e28054458:d1993c2c3ec44c94`, sent from the .107 side) asked
ring20-management-agent for path A (T-1438 staged binary swap) or path
B (`fw upgrade` for the T-1814 bridge fix). They did the swap silently
— no T-1166-specific reply on the thread, but the binary version is
now 0.9.2127 and cut_ready flipped. That's the interactive
doorbell+mail arc paying off: one DM, one operator action, fourteen
days of bake-clock churn cleared.

**Ack DM sent** (offset 28 on the same topic, this session 2026-05-30):
"thank you, T-1166 entry gate cleared, T-1415 promoted to horizon=now
for human review, no further .122 operator action needed for the cut".

**Next move.** Source-cleanup is owner=human and lives in T-1415. The
remaining 4 unchecked Agent ACs in T-1166 (lines 42–45) are all marked
"DEFERRED TO T-1415" — the deferral target is now structurally
unblocked. T-1166's role (validate cut path, advertise via capability
handshake, publish migration guide, gate entry) is complete.

**T-1166 disposition.** Staying in `started-work` until T-1415 closes,
per the deferral contract — the four open agent ACs are explicitly
about source-deletion which is T-1415's deliverable. Promoting T-1415
to `horizon=now` so the human gets it surfaced.

### 2026-05-31T12:10Z — CUT GATE CLEAR — 7d clean window achieved [agent]

**`fw metrics api-usage --cut-ready --json` → `{cut_ready: true, legacy_attributable: 0, window_days: 7}`.**

Full trend snapshot (taken 2026-05-31T12:10Z):

| Window | total RPC | legacy | legacy_attributable | legacy_pct | gate (≤1.0%) |
|---|---:|---:|---:|---:|---|
| 1d  |   355,390 | 0 | 0 | 0.000% | PASS |
| 7d  | 2,164,139 | 0 | 0 | 0.000% | **PASS** ← cut gate |
| 30d | 5,430,768 | 2,544 | 2,544 | 0.047% | PASS |
| 60d | 5,502,498 | 7,873 | 4,472 | 0.143% | PASS |

**Last-seen legacy events (all >7 days old):**
- `event.broadcast` peer_ip=`192.168.10.122` (ring20-management): **2026-05-22T11:46Z** — 9 days ago. This was the last lingering G-061 framework-bridge fallback emission. .122 has now stopped — either pulled T-1814's `lib/pickup-channel-bridge.sh` fix, or its `channel.post` started succeeding (e.g. via the T-1438 staged binary swap).
- `inbox.status` peer_ip=`192.168.10.143`/`.121` (ring20 ring): last seen 2026-05-03T08:04Z (28 days ago). The migration to channel-aware pollers per T-1435 has effectively landed.

**Structural implications:**

1. **T-1166 cut gate (this task's primary entry-AC) is now MET** — the 7d ≤1% gate was originally the operator's cut-decision gate. It's been re-verified passing at 0.000%, with no legacy traffic in the entire 7d window. Previously the gate had been "almost ready" for weeks; this is the first definitively-clean reading.

2. **G-061 (gaps register: bus bridges to retired primitives) is MITIGATED in observable terms** — the last `event.broadcast` from `lib/pickup-channel-bridge.sh` fallback fired 9 days ago. The fix that stopped it is either:
   (a) T-1814's source-level removal of the fallback in `lib/pickup-channel-bridge.sh` + `publish-learning-to-bus.sh` (which lands on .122 only if it `fw upgrade`s its own framework checkout), OR
   (b) .122's `channel.post` started succeeding (T-1438 binary swap → `--ensure-topic` supported → topic-loss fallback no longer triggers).
   Either path silences the emitter; both are operationally fine. The 9-day clean signal is stronger than either single fix would predict on its own.

3. **T-1415 (post-cut source cleanup — delete dead handlers from `crates/termlink-hub/src/router.rs`) is now structurally unblocked.** T-1415 was the deferred follow-up holding pen for source deletion under the bake-window contract. With the bake window clean, T-1415 can advance.

**Next operator action (Tier-2 — human authority):** the original entry-gate Human AC requires explicit retirement timing approval. That was already ticked 2026-04-23. With the gate now definitively clean, the human can proceed to:
- Authorize T-1415 phase advance (source deletion + final CLI/MCP cleanup).
- Tag a release that includes T-1814's bridge fix as a CUT-MILESTONE marker.
- Update `.context/project/concerns.yaml` G-061 from `watching` → `resolved` (this Update establishes the evidence; the register edit is the human's sovereignty action).

Agent recommendation: GO on the cut. Evidence is solid (7d clean + 9d distance from last emission + T-1814 source fix landed both vendored AND upstream).

### 2026-06-10T19:15Z — cut_ready REGRESSED to false: .122 fired event.broadcast 2026-06-04 / 06 [agent]

**Fresh state:** `fw metrics api-usage --cut-ready --json` → `{cut_ready: false, legacy_attributable: 2, window_days: 7}`.

**Root cause (re-confirmed):** the lone emitter is .122 (ring20-management).
The 2 in-window calls are:

| ts | when | source |
|---|---|---|
| 1780696247121 | 2026-06-05T18:30Z | `192.168.10.122` event.broadcast |
| 1780750186281 | 2026-06-06T12:49Z | `192.168.10.122` event.broadcast |

Last call was **4 days ago** (2026-06-06). If .122 stays quiet, the older
call rolls out of the 7d window on **2026-06-12 18:30Z** (~47h) and the
younger on **2026-06-13 12:49Z** (~65h). `cut_ready` re-flips to true at
that point — *unless .122 fires again first*.

**Why this regressed.** Per the 2026-05-26 + 2026-05-30 entries:
ring20-management-agent took path A (T-1438 binary swap → 0.9.2127) but
did NOT take path B (`fw upgrade` on the framework checkout). The hub
binary has the `legacy_primitives_disabled` cut so it CAN'T emit legacy
itself, but the **framework checkout on .122** still carries the
pre-T-1814 `lib/pickup-channel-bridge.sh` that falls back to
`event.broadcast` when `channel.post` fails. So whenever a pickup is
emitted on .122 AND `channel.post` fails for any reason (topic missing,
hub transient, auth blip), the bridge re-fires `event.broadcast`.

**Verified on .107 local checkout:** `lib/pickup-channel-bridge.sh:95-101`
is the T-1814 fix — the legacy fallback is REMOVED, `channel.post` failure
degrades to a logged no-op. .107's framework is correct. The fix exists
upstream (origin/master f87f8e97). The bug is .122-specific: its
vendored framework checkout has not pulled the fix.

**Audit log snapshot over the full 60d window:** 212 `event.broadcast`
lines total — 197 from "(unknown)" (pre-T-1409 backlog, ages out
naturally), 15 attributable to .122 (the bridge fallback firings). Cadence
~1 fire every 3 days over the window. The structural risk is the bridge
keeps firing until .122's framework gets T-1814.

**Two paths forward:**

A) **Wait and hope** — if .122 stays quiet ~65h, cut_ready naturally
   re-flips on 2026-06-13 12:49Z. Risk: any single pickup-with-channel.post-blip
   on .122 in the interim resets the clock. Cost: 0.

B) **Durable fix — DM ring20-management-agent asking for `fw upgrade`** on
   .122 to pull the T-1814 bridge fix into its framework checkout. This
   permanently kills the fallback emission. Cost: 1 DM, matches the
   2026-05-26 precedent. Cost is small but it's a cross-host write to
   a remote operator session, so operator authorization preferred.

**Agent recommendation:** path B. Path A buys you a clean window for
maybe 2-3 days before the next stochastic blip resets it. We've burned
~16 days of bake clock to this same emitter over the last 6 weeks. The
fix is upstream, vendored, verified — it just needs to land on .122.

**Operator action surface (if approving path B):** the DM message body
should ask ring20-management-agent to run `fw upgrade` on the .122 AEF
framework checkout (pulls f87f8e97 → no fallback → channel.post failure
becomes a no-op). Wait 7d, then re-verify `cut_ready: true`. T-1415
remains the deferred source-cleanup task — it's structurally unblocked
and waiting on Tier-2 authorization regardless of this last-mile cleanup.

### 2026-06-10T19:20Z — path B executed: DM sent to ring20-management-agent asking for fw upgrade [agent under operator authorization]

User authorized path B. `/be-reachable start` established sender identity
`root-claude-dimitrimintdev` on .107. DM delivered via `termlink agent
contact --target-fp 9219671e28054458 --thread T-1166` to the canonical
`dm:<.107-fp>:9219671e28054458` topic — offset 44, ts 1781119429542.
The .122 host fp `9219671e28054458` is the shared host-key for the
ring20-management container (reference_shared_host_identity.md memory).
Both .122 co-resident agents (ring20-management-agent + skills-manager-agent)
will see the DM; message explicitly addresses ring20-management-agent
and asks for `fw upgrade` on the .122 AEF framework checkout.

**Expected timeline.** ring20-management-agent picks up the DM (poll
cadence varies); runs `fw upgrade`; bridge stops emitting; 7d clean bake
follows. If they take ~24-48h to act, the 7d window may already have
naturally cleared (rolls past 2026-06-06T12:49Z at 2026-06-13T12:49Z),
but the durable fix prevents the next stochastic blip from re-arming
the clock.

**Next checkpoint.** Re-run `fw metrics api-usage --cut-ready --json`
in ~24h to see if any new `event.broadcast` lines from .122 landed. If
zero new lines for 7d post-DM-action: cut_ready=true and T-1415
unblocked.
