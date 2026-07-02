---
id: T-2312
name: "arc-004 WS-over-Unix push for co-located agents — should --push work over the Unix socket, not just TCP?"
description: >
  Inception: arc-004 WS-over-Unix push for co-located agents — should --push work over the Unix socket, not just TCP?

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-02T18:58:13Z
last_update: 2026-07-02T18:58:28Z
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

# T-2312: arc-004 WS-over-Unix push for co-located agents — should --push work over the Unix socket, not just TCP?

## Problem Statement

`channel subscribe <topic> --push` (arc-004 S3b / T-2309) gives sub-second DM push
(~90 ms, T-2310) — but **only over a TCP hub**. `connect_tls_stream` explicitly
rejects Unix targets (`WS-over-Unix is a follow-on`), so **co-located agents** —
those sharing a host with the hub and talking to it over the Unix socket — get
**no live push at all**; they always fall to the 1 s poll floor. **For whom:**
same-host agents (e.g. the multiple co-resident agents on .107). **Why now:** the
investigation for this inception found the hub *already* supports WS-over-Unix, so
the gap is a small, cheap client-only fix — worth closing while the arc is fresh.

## Assumptions

- **A1** — The hub already routes a Unix `GET` connection to the WS handler and
  authorises `hub.ws_subscribe` at the Execute scope Unix connections start with
  (**verified** — see Evidence; not an assumption anymore, retained for audit).
- **A2** — A raw `client_async` over the Unix stream (no TLS) completes the WS
  handshake the hub's `accept_async` expects. *To validate in a build spike.*
- **A3** — Skipping the token mint for Unix (peer-cred trust) is correct and does
  not weaken any boundary (same-UID Unix access is already full-trust).

## Open Questions

<!-- filed per T-2194/G-067 readiness gate; disposed before work-completed -->

- **IW-1: Should `--push` support the Unix socket, or is TCP-only the correct v1?**
  confidence: 2
  disposition: deferred
  rationale: Agent advises GO (client-only, low-effort, serves co-located agents); final go/no-go is the human's (sovereignty-gated).

- **IW-2: Does a raw client_async over the Unix stream (no TLS) complete the handshake the hub's accept_async expects?**
  confidence: 2
  disposition: deferred
  rationale: Hub handler is generic over AsyncRead+AsyncWrite (server.rs:830) — no TLS assumption; confirm with a build spike (A2).

- **IW-3: Is skipping hub.auth for Unix `--push` correct (peer-cred trust), and does anything downstream assume an authed token on the WS path?**
  confidence: 2
  disposition: deferred
  rationale: Unix connections start at Execute scope which satisfies ws_subscribe's Observe gate (server.rs:987-1010) — no token needed; verify no other WS RPC on the path requires a token.

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

Exploration is a hub-side code-read to determine one/two-sidedness (done):
1. **Read** the hub accept loop + `handle_connection` + `handle_ws_connection` +
   `maybe_handle_ws_subscribe` (done — `server.rs` 549/760/830/975). Result: the
   hub ALREADY supports WS-over-Unix; the sniff routes Unix `GET` to the generic
   WS handler and Execute-scope Unix connections pass the subscribe gate with no auth.
2. **Read** the client blocker (`connect_tls_stream`, `client.rs:112` — rejects Unix).
3. **Recommend** a client-only build path + spike to validate the raw handshake.
4. **Human go/no-go** via `fw task review T-2312` (pending — sovereignty-gated).

## Technical Constraints

- **No hub change:** confirmed the hub already handles WS-over-Unix; scope must stay
  client-side to keep the change bounded (any hub change would be out of scope).
- **No TLS over Unix:** the Unix socket is peer-cred-trusted, not TLS. The client
  WS handshake must run `client_async` over the raw Unix stream (no rustls layer).
- **No auth over Unix:** Unix connections are pre-granted Execute scope; the client
  must NOT attempt a `hub.auth` token mint (there is no TCP secret for a Unix hub).
- **Same no-miss invariant** as the TCP path: degrade-to-poll from the durable
  cursor stays the authoritative floor.

## Scope Fence

- **IN:** whether/how `channel subscribe --push` should work over the Unix socket;
  a client-only `connect_ws_unix` + routing Unix `--push` through it; a spike to
  validate the raw handshake; suggested build slices.
- **OUT:** any hub-side change (confirmed unnecessary); active reconnect-to-WS
  (separate follow-on, T-2311); WS-over-Unix for any consumer other than the CLI
  `--push` path; the build itself (post-GO, separate build task).

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
- The fix is **client-only** and bounded (no hub/protocol change). — met (hub
  already supports WS-over-Unix; only `connect_tls_stream`'s Unix rejection blocks it)
- The raw `client_async`-over-UDS handshake is validated by a spike. — pending build spike (IW-2)
- No boundary is weakened by skipping auth over Unix. — met (same-UID Unix is already full-trust)

**NO-GO if:**
- A build spike shows the raw Unix WS handshake needs a hub-side change after all
  (would break the "client-only, bounded" premise).
- Co-located-agent push is judged low-value because same-host agents rarely need
  sub-second cross-agent DM (they can share memory/files directly). — the human's call.

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

**Rationale:**

Investigation found the HUB SIDE ALREADY SUPPORTS WS-over-Unix: handle_connection (called by BOTH the Unix and TCP-after-TLS accept paths) runs the T-2305 first-byte sniff, so a Unix client sending 'GET ...' routes to the generic handle_ws_connection (which is generic over AsyncRead+AsyncWrite — no TLS assumption); and Unix connections start at PermissionScope::Execute, which satisfies the Observe requirement for hub.ws_subscribe — so NO hub.auth is needed. The ONLY blocker is the CLIENT: connect_tls_stream explicitly rejects Unix and always does TLS. WS-over-Unix is therefore a bounded CLIENT-ONLY change: add a connect_ws_unix (Unix connect + client_async over the raw UDS, no TLS), route --push through it for Unix addrs (skipping the token mint), and drop the Unsupported rejection. Value: co-located agents on shared hosts (e.g. .107 multi-agent) get the same ~90ms push instead of forced poll. Low effort, no hub/protocol change. GO to scope; final decision is the human's.

**Evidence:**

- `crates/termlink-hub/src/server.rs:760` `handle_connection` — called by BOTH the
  Unix accept path (line ~618) and the TCP-after-TLS path — runs the T-2305
  first-byte sniff (`is_ws = first == 'G'`, ~785), routing Unix `GET` connections
  to `handle_ws_connection`. **Hub already accepts WS over Unix.**
- `server.rs:830` `handle_ws_connection<S>` is generic over
  `AsyncRead + AsyncWrite` — **no TLS assumption**; it runs `accept_async(stream)`
  over whatever stream (raw Unix or TLS).
- `server.rs:616-625` — Unix accept passes `Some(PermissionScope::Execute)` as the
  initial scope. `server.rs:987-1010` `maybe_handle_ws_subscribe` allows subscribe
  when scope satisfies `Observe` — Execute does — so **Unix WS subscribe needs no `hub.auth`**.
- `crates/termlink-session/src/client.rs:112-122` `connect_tls_stream` — the sole
  blocker: returns `Unsupported` for Unix and always wraps TLS. A sibling
  `connect_ws_unix` (raw `client_async` over the Unix stream, no TLS, no token) is
  the whole client-side change.
- Full analysis: `docs/reports/T-2312-arc-004-ws-over-unix-inception.md`.

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

### 2026-07-02T18:58:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
