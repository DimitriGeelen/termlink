---
id: T-2771
name: "Local IPC authorization is uid-coupled while remote is identity-based — fragmentation
  is the default outcome"
description: >
  A unix domain socket carries filesystem permissions, so local hub access is decided
  by whichever uid started the hub and its umask, never by TermLink's own auth. Remote
  TCP uses HMAC and is user-agnostic. Net inversion: a remote host on another machine
  can authenticate in while a local agent on the same box cannot — and an agent that
  cannot reach the hub silently starts its own, which is why .107 currently runs three.
  Origin: AEF agent (/opt/999-Agentic-Engineering-Framework), 2026-08-16.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-16T16:42:47Z
last_update: '2026-08-18T18:58:40Z'
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:38Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2771: Local IPC authorization is uid-coupled while remote is identity-based — fragmentation is the default outcome

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: `SO_PEERCRED` (option 2) or loopback TCP + HMAC (option 3)?**
  confidence: 2
  disposition:
  rationale: Option 2 keeps the unix-socket fast path and makes local authorization use
  the hub's own model. Option 3 is the only one that yields literally ONE auth path, which
  is the stronger Directive #4 story, but it gives up the local fast path and opens a port
  on every host. Lean 2; the decision hinges on IW-2.

- **IW-2: Does `SO_PEERCRED` create a NEW portability gap while closing a uid gap?**
  confidence: 3
  disposition: dissolved
  rationale: DISSOLVED by reading the code (T-2772 session, 2026-08-16). The premise was
  wrong: option 2 is not a future change, it is ALREADY SHIPPED and has been all along.
  `PeerCredentials::from_raw_fd` (termlink-session/src/auth.rs:28-47) already branches
  `#[cfg(target_os = "linux")]` → `SO_PEERCRED`, `#[cfg(target_os = "macos")]` →
  `LOCAL_PEERCRED`, and returns `io::ErrorKind::Unsupported` elsewhere; the hub calls it
  via `decide_unix_peer` (hub/src/server.rs:704). So there is no portability gap to open —
  the cross-platform abstraction predates the question. The `Unsupported` arm is
  Directive #4-correct on a third platform too: it becomes `Reject{uid_mismatch: None}`,
  which fails closed rather than silently trusting. Note this INVERTS the original framing:
  the AEF report described local access as decided purely by filesystem permissions, and
  that is only the FIRST of two gates.

- **IW-3: What does authorization by uid actually GRANT?**
  confidence: 3
  disposition: answered
  rationale: ANSWERED from code (hub/src/server.rs, accept loop): a same-uid Unix peer is
  granted `Some(PermissionScope::Execute)` — the highest scope, unconditionally, with the
  in-code comment "Unix same-UID connections get full access (no auth needed)". So the
  current model is binary and coarse: same uid ⇒ everything, different uid ⇒ nothing. That
  answers the shape question posed here — uid is a BOUNDARY, not an identity, and it maps
  no local peer to a distinct principal. Consequence for the T-2769 merge question: the two
  do NOT merge. T-2769 needs per-agent identity for claim ownership, which uid cannot
  supply, because every agent running as the same user is one principal here (F2 measured
  exactly that: two projects on .107 sharing fingerprint d1993c2c3ec44c94). Design them
  separately.

- **IW-6: Should the same-uid grant be `Execute` (everything) at all?** *(new, T-2772)*
  confidence: 1
  disposition:
  rationale: Falls out of IW-3's answer and was not visible before it. Any process running
  as the hub's uid gets full `Execute` — including force-release of another agent's claims
  and topic deletion (`CHANNEL_DELETE`, Execute per T-2421). On a host where several agent
  runtimes deliberately share one uid, that is every agent holding operator-tier authority
  over every other. Whether that is intended or merely inherited from the single-user
  origin is a human question; it also bounds how much T-2769 can achieve, since an
  authenticated `claimer` is defeated by any co-uid peer that can simply force-release.

- **IW-4: Is option 1 acceptable as an interim, and how do we stop it becoming "the fix"?**
  confidence: 2
  disposition:
  rationale: `UMask=0002` + a shared group unblocks this host today and is genuinely
  useful. The risk is documentation drift: it is deployment config, so every host that
  does not apply it still fragments silently, and a runbook line reading "fixed" would be
  false for a fresh install. If adopted, it must be recorded as a MITIGATION with the gap
  left open — the T-2483 allowlist pattern (a ledger of an open question) rather than a
  closure.

- **IW-5: How would we ever detect recurrence?**
  confidence: 1
  disposition:
  rationale: Preflight Check 6 caught the ghost on THIS host because it compares the
  pidfile PID to the unit MainPID. It would not notice a hub owned by another uid under a
  different `runtime_dir` (e.g. `/tmp/termlink-1000`) — that hub is invisible to a check
  looking at one path. Any fix should come with detection for "more than one live hub on
  this host, across runtime_dirs", or the class stays silent the next time it appears in a
  new shape.

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
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
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:** MEASURED on .107: systemd unit runs User=root with StateDirectoryMode=0700, hub.sock lands root:root 0755, and three hubs are live (root's, dimitri-mint-dev's under /tmp/termlink, systemd's). Nobody misconfigured anything — fragmentation is the DEFAULT outcome once two agent runtimes run as different users, now the normal case with Claude Code and Codex side by side. Silent fragmentation is a Directive #2 violation by design rather than by bug, and it defeats charter verb 1 (discover) and verb 2 (exchange durable messages) locally while both still work across hosts. Self-demonstrating evidence: this session could not DM the AEF agent to co-write this task, because they are absent from agent-presence and .107's hub is unreachable from here — two agents on one machine unable to use the agent-to-agent comms tool to discuss the bug in agent-to-agent comms. Three candidate fixes: (1) UMask=0002 + shared termlink group + StateDirectoryMode=0770 — smallest and unblocks today, but it is DEPLOYMENT CONFIG, not a TermLink fix; every host that does not apply it still fragments, so it must not be recorded as the fix. (2) authorize local clients via SO_PEERCRED inside the hub so local and remote share one model — structurally right and my recommendation, with a Directive #4 caveat: SO_PEERCRED is Linux, macOS needs LOCAL_PEERCRED/getpeereid, and README claims macOS first-class, so it needs T-2693 platform-lock treatment or it trades a uid gap for a portability gap. (3) drop the unix socket for loopback TCP + the same HMAC — genuinely one auth path but gives up the local fast path and opens a port. Sequence behind T-2770, which contains the damage by making hub start refuse on EACCES instead of starting a rival hub.

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

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: MEASURED on .107: systemd unit runs User=root with StateDirectoryMode=0700, hub.sock lands root:root 0755, and three hubs are live (root's, dimitri-mint-dev's under /tmp/termlink, systemd's). Nobody misconfigured anything — fragmentation is the DEFAULT outcome once two agent runtimes run as different users, now the normal case with Claude Code and Codex side by side. Silent fragmentation is a Directive #2 violation by design rather than by bug, and it defeats charter verb 1 (discover) and verb 2 (exchange durable messages) locally while both still work across hosts. Self-demonstrating evidence: this session could not DM the AEF agent to co-write this task, because they are absent from agent-presence and .107's hub is unreachable from here — two agents on one machine unable to use the agent-to-agent comms tool to discuss the bug in agent-to-agent comms. Three candidate fixes: (1) UMask=0002 + shared termlink group + StateDirectoryMode=0770 — smallest and unblocks today, but it is DEPLOYMENT CONFIG, not a TermLink fix; every host that does not apply it still fragments, so it must not be recorded as the fix. (2) authorize local clients via SO_PEERCRED inside the hub so local and remote share one model — structurally right and my recommendation, with a Directive #4 caveat: SO_PEERCRED is Linux, macOS needs LOCAL_PEERCRED/getpeereid, and README claims macOS first-class, so it needs T-2693 platform-lock treatment or it trades a uid gap for a portability gap. (3) drop the unix socket for loopback TCP + the same HMAC — genuinely one auth path but gives up the local fast path and opens a port. Sequence behind T-2770, which contains the damage by making hub start refuse on EACCES instead of starting a rival hub.

**Date**: 2026-08-18T12:10:07Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-18T12:10:02Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: MEASURED on .107: systemd unit runs User=root with StateDirectoryMode=0700, hub.sock lands root:root 0755, and three hubs are live (root's, dimitri-mint-dev's under /tmp/termlink, systemd's). Nobody misconfigured anything — fragmentation is the DEFAULT outcome once two agent runtimes run as different users, now the normal case with Claude Code and Codex side by side. Silent fragmentation is a Directive #2 violation by design rather than by bug, and it defeats charter verb 1 (discover) and verb 2 (exchange durable messages) locally while both still work across hosts. Self-demonstrating evidence: this session could not DM the AEF agent to co-write this task, because they are absent from agent-presence and .107's hub is unreachable from here — two agents on one machine unable to use the agent-to-agent comms tool to discuss the bug in agent-to-agent comms. Three candidate fixes: (1) UMask=0002 + shared termlink group + StateDirectoryMode=0770 — smallest and unblocks today, but it is DEPLOYMENT CONFIG, not a TermLink fix; every host that does not apply it still fragments, so it must not be recorded as the fix. (2) authorize local clients via SO_PEERCRED inside the hub so local and remote share one model — structurally right and my recommendation, with a Directive #4 caveat: SO_PEERCRED is Linux, macOS needs LOCAL_PEERCRED/getpeereid, and README claims macOS first-class, so it needs T-2693 platform-lock treatment or it trades a uid gap for a portability gap. (3) drop the unix socket for loopback TCP + the same HMAC — genuinely one auth path but gives up the local fast path and opens a port. Sequence behind T-2770, which contains the damage by making hub start refuse on EACCES instead of starting a rival hub.

### 2026-08-18T12:10:07Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: MEASURED on .107: systemd unit runs User=root with StateDirectoryMode=0700, hub.sock lands root:root 0755, and three hubs are live (root's, dimitri-mint-dev's under /tmp/termlink, systemd's). Nobody misconfigured anything — fragmentation is the DEFAULT outcome once two agent runtimes run as different users, now the normal case with Claude Code and Codex side by side. Silent fragmentation is a Directive #2 violation by design rather than by bug, and it defeats charter verb 1 (discover) and verb 2 (exchange durable messages) locally while both still work across hosts. Self-demonstrating evidence: this session could not DM the AEF agent to co-write this task, because they are absent from agent-presence and .107's hub is unreachable from here — two agents on one machine unable to use the agent-to-agent comms tool to discuss the bug in agent-to-agent comms. Three candidate fixes: (1) UMask=0002 + shared termlink group + StateDirectoryMode=0770 — smallest and unblocks today, but it is DEPLOYMENT CONFIG, not a TermLink fix; every host that does not apply it still fragments, so it must not be recorded as the fix. (2) authorize local clients via SO_PEERCRED inside the hub so local and remote share one model — structurally right and my recommendation, with a Directive #4 caveat: SO_PEERCRED is Linux, macOS needs LOCAL_PEERCRED/getpeereid, and README claims macOS first-class, so it needs T-2693 platform-lock treatment or it trades a uid gap for a portability gap. (3) drop the unix socket for loopback TCP + the same HMAC — genuinely one auth path but gives up the local fast path and opens a port. Sequence behind T-2770, which contains the damage by making hub start refuse on EACCES instead of starting a rival hub.
