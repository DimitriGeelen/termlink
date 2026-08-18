---
id: T-2789
name: "RCA cross-UID TermLink delivery from 0503-Codex (dimitri-mint-dev) to the root-owned AEF hub"
description: >
  Codex agent running as dimitri-mint-dev in /opt/0503-codex-cli-playground cannot reach AEF session tl-812b4f38 on the root-owned hub (TERMLINK_RUNTIME_DIR=/var/lib/termlink); root-side ping succeeds at ~1ms. Produce evidence-backed RCA plus at least two least-privilege fix options with security trade-offs and a preferred architecture. Constraints from the operator: no world-writable sockets, no broad chmod/chown, no copied secrets, no unaudited bypass, no interactive sudo for normal governed collaboration, and delivery must remain distinct from receipt.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-18T11:49:38Z
last_update: 2026-08-18T11:50:22Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2789: RCA cross-UID TermLink delivery from 0503-Codex (dimitri-mint-dev) to the root-owned AEF hub

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

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

- **IW-1: Which layer actually refuses the Codex agent — session discovery, or filesystem permission?**
  confidence: 3
  disposition: answered
  rationale: Filesystem, and ONLY filesystem. `/var/lib/termlink` is 0700 root:root (traverse denied live) and `hub.sock` is 0755 so connect() lacks write (rc=1 live). The competing hypothesis — an inherited `TERMLINK_RUNTIME_DIR` disabling multi-dir scan via `discovery.rs:44-47` — was MEASURED and ruled out: `strings /proc/2992705/environ` on the running codex process (user dimitri-mint-dev) shows `TERMLINK_RUNTIME_DIR=/var/lib/termlink`, i.e. correctly aimed at the dir it cannot enter. Read from the running process, not `sudo -u … env`, which would have reported a fresh login shell rather than what the agent actually inherited.

- **IW-2: Does TermLink already ship a capability-scoped credential, so that granting the
  Codex agent access does not mean handing it the full `hub.secret`?**
  confidence: 3
  disposition: answered
  rationale: The TYPE exists and is verified (`auth.rs:270-346`, `server.rs:644-712`, regression test `router.rs:2809`) but it is NOT a privilege boundary — every production mint site runs client-side signing with the client's own `hub.secret` and choosing its own scope (`remote.rs:770,6417`, `mcp/tools.rs:7770`, and `channel.rs:428,472` hardcoding Execute). No server-side issuance, no subject field, no revocation. Possession of the secret ≡ Execute.

- **IW-3: Is the local UNIX-socket control plane the right transport for this at all, or is
  the authenticated TCP hub the intended cross-boundary path?**
  confidence: 3
  disposition: answered
  rationale: TCP — it is the only plane that starts unauthenticated (`server.rs:873`), whereas a UNIX connection is granted `PermissionScope::Execute` unconditionally at `server.rs:813`. But TCP is not sufficient today, because the credential it accepts is minted from a secret the client must hold (IW-2). This is what refutes the socket-ACL option: an ACL selects WHO connects and cannot constrain WHAT they may do.

- **IW-4: Which layer owns each of the operator's five proof obligations (unauthorised
  refused / authorised delivers / receipt observable / no secret in logs / wrong-worktree
  refused)? Some may be TermLink's, some AEF's, some neither.**
  confidence: 3
  disposition: answered
  rationale: Split three ways — see report §3. Obligations 1 and 3 hold today (TCP zero-scope; `--await-ack`/`awaiting-ack` per T-2286/T-2287, class named by PL-247). Obligation 2 is blocked (no grant narrower than full Execute exists). Obligation 4 is convention (R3/G-011), not structural. Obligation 5 is AEF's T-559 project-boundary hook, NOT TermLink's — it fired during this investigation and does not apply to a bare binary invocation.

- **IW-5: Is the UID boundary this exercise defends actually enforced by the host?**
  confidence: 3
  disposition: answered
  rationale: No. `id dimitri-mint-dev` shows membership of `sudo` AND `docker`; docker-group membership is root-equivalent without a password. The account already holds root-equivalent authority by two independent routes. This does not moot the request — ambient vs available authority still matters for an autonomous process — but it resizes Option D from "security boundary" to "containment of agent behaviour" unless the host is hardened first. Question was not filed at inception; it surfaced from the live `id` check in §1.6.

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
- [ ] RCA names the refusing layer with live evidence, not inference (`docs/reports/T-2789-cross-uid-delivery-rca.md` §1.1, §1.6)
- [ ] At least two fix options carry an explicit "authority actually granted" column, so
      options that look narrow but are not are visibly distinguished (§2)
- [ ] Every IW question carries a disposition and a `file:line` or measured-output rationale
- [ ] The report states what was NOT done and why (§7) — in particular that no live
      cross-UID delivery proof was produced, because producing one requires granting the
      authority the report recommends against
- [ ] No permission, group, secret, or config on this host was modified by this task

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

test -f docs/reports/T-2789-cross-uid-delivery-rca.md
# The two comments the whole RCA rests on must be cited by exact file:line, not paraphrased.
out=$(cat docs/reports/T-2789-cross-uid-delivery-rca.md 2>&1 || true); grep -q "server.rs:813" <<< "$out"
out=$(cat docs/reports/T-2789-cross-uid-delivery-rca.md 2>&1 || true); grep -q "server.rs:873" <<< "$out"
# The client-self-mints-Execute site — the finding that makes "scoped token" unavailable today.
out=$(cat docs/reports/T-2789-cross-uid-delivery-rca.md 2>&1 || true); grep -q "channel.rs:428" <<< "$out"
# Report must state what was not done (§7) rather than implying the proof obligations were met.
out=$(cat docs/reports/T-2789-cross-uid-delivery-rca.md 2>&1 || true); grep -q "No live cross-UID delivery proof" <<< "$out"
# Every IW disposed (5 filed, incl. IW-5 which surfaced mid-investigation).
out=$(grep -c "disposition: answered" .tasks/active/T-2789-rca-cross-uid-termlink-delivery-from-050.md 2>/dev/null || echo 0); test "$out" -ge 5
# Invariant: this task must NOT have loosened the very permissions it recommends against changing.
out=$(stat -c '%a' /var/lib/termlink 2>&1 || true); grep -q "^700$" <<< "$out"
out=$(stat -c '%a' /var/lib/termlink/hub.secret 2>&1 || true); grep -q "^600$" <<< "$out"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:** Recommendation is the deliverable of this inception, not an input; set at decide time.

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

**Rationale**: Recommendation is the deliverable of this inception, not an input; set at decide time.

**Date**: 2026-08-18T12:22:40Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-18T11:50:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-18T12:22:40Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation is the deliverable of this inception, not an input; set at decide time.
