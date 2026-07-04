---
id: T-2339
name: "Inception — WS-over-Unix for co-located push consumers"
description: >
  arc-004 follow-on. Question (one, go/no-go): should co-located agents get WS push over a Unix socket instead of TCP+TLS-to-localhost? connect_tls_stream rejects Unix today. Explore only if a same-host workload demonstrates the localhost TLS path is a bottleneck. See docs/operations/push-transport-recipe.md + docs/reports/T-2309 scope finding.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-03T19:10:07Z
last_update: 2026-07-04T09:07:22Z
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

# T-2339: Inception — WS-over-Unix for co-located push consumers

## Problem Statement

Should co-located agents (same host as the hub) get WS push over the hub's **Unix socket**
instead of TCP+TLS-to-localhost? Filed from a T-2309 scope note claiming
`connect_tls_stream` rejects Unix, so same-host consumers would be paying TLS-handshake +
loopback-TCP overhead for no trust benefit. For: every `/be-reachable` push-waker session
on a hub host (the most common consumer topology in this fleet).

## Assumptions

- A1 (the filing premise): "WS push cannot ride the Unix socket today" — **REFUTED.** The
  premise was already stale at filing time: the T-2312 inception GO'd exactly this and
  T-2313 built it. `scripts/demo-ws-push-unix.sh` proves a live consumer on
  `channel subscribe <topic> --push` over the hub's Unix socket receives a DM push (~31 ms,
  no TLS, no hubs.toml profile, no token — Unix connections are peer-cred-trusted and
  pre-granted Execute scope, which satisfies `hub.ws_subscribe`).
- A2: "co-located consumers actually use the Unix path in practice" — **VERIFIED.** The
  push-waker (`be-reachable-pushwaker.sh`) invokes `channel subscribe … --push` with no
  `--hub` unless overridden; default address resolution for the local hub is the
  runtime_dir Unix socket. Co-located sessions ride Unix push today.

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

- **IW-1: Should co-located push consumers get WS-over-Unix instead of TCP+TLS-to-localhost?**
  confidence: 3
  disposition: dissolved
  rationale: Premise false — already shipped. T-2312 inception (GO) → T-2313 build;
  `scripts/demo-ws-push-unix.sh` proves ~31 ms Unix push with no TLS/token; the
  push-waker's default local-hub resolution already selects the Unix socket, so
  co-located consumers ride it today with zero configuration.

## Exploration Plan

Executed (source + artifact verification, no spikes needed): (1) locate the shipped
WS-over-Unix evidence — `docs/reports/T-2312-arc-004-ws-over-unix-inception.md` +
`scripts/demo-ws-push-unix.sh` (part of the arc-004 reproducer matrix, passing); (2)
confirm the consumer's default address resolution picks the Unix socket for the local hub
(`channel.rs` transport-addr fallback + `be-reachable-pushwaker.sh` passing no `--hub`);
(3) check nothing co-located is forced onto TCP+TLS-to-localhost. All three confirm the
capability exists and is the default.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** whether WS-over-Unix for co-located consumers is needed (one go/no-go question).
**OUT:** building any transport code under this ID (it already exists — T-2313); remote
(cross-host) consumers, which correctly stay on TCP+TLS; changes to the Unix peer-cred
trust model.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
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
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** NO-GO (dissolved — already shipped by T-2312/T-2313)

**Rationale (revised 2026-07-04 after artifact verification):** The question this inception
poses was answered and BUILT before it was filed. T-2312 ran the WS-over-Unix inception
(GO), T-2313 built it, and `scripts/demo-ws-push-unix.sh` is standing regression evidence:
a live consumer subscribed with `channel subscribe <topic> --push` over the hub's Unix
socket receives a DM push in ~31 ms — no TLS, no hubs.toml profile, no token (Unix
connections are peer-cred-trusted and pre-granted Execute scope, satisfying
`hub.ws_subscribe`). Co-located consumers get this path BY DEFAULT: the push-waker passes
no `--hub`, and local-hub address resolution selects the runtime_dir Unix socket. There is
nothing left to decide or build; a GO here would authorise duplicate work.

**Original DEFER draft (retained for audit — premise was stale):** ~~connect_tls_stream
currently rejects Unix sockets, so a co-located agent gets WS push over TCP+TLS to
127.0.0.1 … DEFER until a same-host workload shows the TLS path is a bottleneck.~~ — this
described the pre-T-2313 state; the "explore only if bottleneck" trigger is moot because
the Unix path already exists and is the default.

Human records the final no-go via `fw inception decide T-2339 no-go` (or the Watchtower
review form).

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

### 2026-07-04T09:07:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
