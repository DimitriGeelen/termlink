---
id: T-1290
name: "Investigate ring20-management hub.secret rotation root cause (G-011 upstream)"
description: >
  Inception: ring20-management (CT 200 / .122) keeps regenerating its hub.secret across
  restarts despite T-933/T-945/T-1028/T-1031 shipping persist-if-present. Every
  regeneration cascades into TOFU + auth-signature failures on every client (G-011).
  The persist mechanism itself is structurally sound (verified by T-1284 + the wider
  fleet running fine), so .122 specifically is doing one of: (a) running an old binary
  predating persist-if-present, (b) systemd restart landing in a different runtime_dir
  per boot (volatile tmpfs?), or (c) something else not yet enumerated. Investigate,
  identify, fix root cause. Closing this upstream eliminates the heal cycle entirely
  and obviates much of T-1291's value.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [auth, infrastructure, ring20-management, G-011]
components: []
related_tasks: [T-1051, T-1284, T-1137, T-933, T-945, T-1028, T-1031, T-1291]
created: 2026-04-26T11:36:22Z
last_update: 2026-04-26T11:49:41Z
date_finished: null
---

# T-1290: Investigate ring20-management hub.secret rotation root cause (G-011 upstream)

## Problem Statement

The ring20-management hub at 192.168.10.122:9100 (running inside CT 200 on proxmox .180) regenerates its hub.secret across restarts. Each rotation breaks every client's pinned auth (TOFU mismatch + invalid signature). The fleet has shipped persist-if-present (T-933/T-945/T-1028/T-1031) and CLAUDE.md explicitly documents that **under normal operation hubs preserve secrets across restarts and clients never need to re-pin** — yet .122 keeps doing it. This is the upstream cause of the recurring G-011 cascades; without this fix, every other hardening (T-1284 value-comparison, T-1054/T-1055 heal commands, T-1291 bootstrap manifest) is symptom-management.

Observed evidence (2026-04-26 session): after one CT reboot, both the TLS fingerprint AND the HMAC secret had rotated. Cleared TOFU successfully → next call hit `Token validation failed: invalid signature`. Tier-2 autoheal (`fleet reauth --bootstrap-from ssh:`) also failed because this container has no SSH key to .122. End result: the only path to fix the root cause (T-1137 logrotate dispatch) was blocked by the very rotation we want to prevent.

## Assumptions

- A-1: Hub binary on .122 is recent enough to include persist-if-present (T-933+). If false, the fix is "upgrade the binary". Validate first.
- A-2: hub.secret on .122 IS being written to disk somewhere; the question is whether that path survives container restart. If false, persist-if-present is silently failing on this host.
- A-3: Other fleet hubs (ring20-dashboard .121, self-hub) do NOT rotate on restart — i.e. .122 is special, not the persist mechanism being broken everywhere.
- A-4: The runtime_dir on .122 is on persistent storage, not tmpfs/zram (CT 200 lives on .180 which is ALREADY on zram for /var/log — possible the same pattern affects /var/lib).

## Exploration Plan

1. **Inspect .122 hub directly** (operator has console access via .180): binary version, configured runtime_dir, presence/contents of `<runtime_dir>/hub.secret`, mount type of runtime_dir parent (`mount | grep <path>`). Time-box: 15 min.
2. **Reproduce the rotation deterministically**: stop the hub on .122, capture `sha256sum hub.secret`, restart, capture again. If different → persist-if-present is broken on this host. If same → rotation only happens on CT-level reboot, narrowing it to volatile-FS scenario. Time-box: 10 min.
3. **Cross-check peer hubs**: same procedure on ring20-dashboard .121 + local self-hub. If they preserve, A-3 is confirmed and the bug is .122-specific. Time-box: 10 min.
4. **Map the failure to one of three CLAUDE.md scenarios** (old binary / different runtime_dir / operator regen) or document a new fourth scenario.
5. **Recommendation**: scoped fix matching the identified scenario.

## Technical Constraints

- Investigation requires either termlink auth to .122 (currently broken — chicken-and-egg) or operator console access to .180 → CT 200. Console path is the only reliable channel today.
- Inspecting hub.secret value on disk requires root on the container; do NOT log it to any shared store, just compare hashes.
- Cannot easily replay historical rotations — only reproduce going forward.

## Scope Fence

**IN:** Find why .122 specifically rotates secrets despite persist-if-present. Identify which of the three known scenarios (or a fourth) applies. Recommend a scoped fix.

**OUT:** Building T-1291 (declarative heal manifest) — separate captured task. Generalizing the fix to all hubs (only do that if the bug turns out to be in shared code, not .122's deploy). Touching auth protocol semantics (T-1284 closed that). Solving the chicken-and-egg of "how do we dispatch fixes when auth is broken" — that's T-1291's domain.

## Acceptance Criteria

### Agent
- [x] Problem statement validated — directly observed in 2026-04-26 session: TLS+secret double-rotation on .122 after CT reboot, while self-hub and .121 preserve.
- [ ] Assumptions tested — A-3 confirmed (other hubs preserve); A-1/A-2/A-4 pending spike 1 (operator console on .122).
- [x] Recommendation written with rationale — preliminary GO in `docs/reports/T-1290-ring20-mgmt-secret-rotation-inception.md`, contingent on spike 1 confirmation that `.122`'s runtime_dir is on tmpfs (`/tmp/termlink-0`).

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO (preliminary — pending spike 1 confirmation)

**Rationale:** Convergent evidence points to scenario (b) in its degenerate form: ring20-management hub on .122 is running with `runtime_dir=/tmp/termlink-0` (legacy default, tmpfs inside the LXC container) instead of the persistent `/var/lib/termlink` configured via `TERMLINK_RUNTIME_DIR` in the T-931 systemd unit. On every CT reboot the tmpfs wipes; persist-if-present then has nothing to find and regenerates secret + cert. Two distinct peer hubs on different code generations both preserve secrets, ruling out a fleet-wide regression. PL-021 (T-1067) already documented the both-secret-AND-cert pattern as "container nuke / clean runtime_dir."

**Evidence:**
- self-hub (.102) `hub.secret` mtime 2026-04-12, unchanged across many container restarts; runtime_dir = `/var/lib/termlink`
- ring20-dashboard (.121) auth still valid 2026-04-26 — last pin held; persist works
- ring20-management (.122) — both TLS fingerprint and HMAC secret rotated since prior pin; `tofu clear` + `Token validation failed` confirms simultaneous regeneration of both, characteristic of a wiped runtime_dir
- T-935 / T-931 history (2026-04-12) shipped the migration `/tmp/termlink-0` → `/var/lib/termlink` via systemd `Environment=TERMLINK_RUNTIME_DIR=...` — `.122` deploy may predate or omit this
- PL-021 prior learning: "When a remote hub rotates BOTH secret and TLS cert (container nuke / clean runtime_dir)..."

**Fix:** Same one-line migration documented at `docs/operations/termlink-hub-runtime-migration.md` (T-935): install systemd unit on CT 200 with `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink`, ensure dir exists at mode 700, restart hub once, re-pin from clients (one-off heal, never again).

**Spike 1 confirmation needed:** Operator console on .180 → `pct enter 200` → `ls -la /tmp/termlink-0/ /var/lib/termlink/` + `mount | grep termlink` + check systemd unit. If `/tmp/termlink-0/hub.secret` exists and is the live one, hypothesis confirmed and we move to GO. If runtime_dir is already `/var/lib/termlink` and still rotates, we have a deeper bug and recommendation flips.

Full artifact: `docs/reports/T-1290-ring20-mgmt-secret-rotation-inception.md`.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-26T11:48:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
