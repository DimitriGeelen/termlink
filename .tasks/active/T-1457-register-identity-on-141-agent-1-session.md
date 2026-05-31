---
id: T-1457
name: "Register identity on .141 agent-1 session (chat-arc peer-addressability gap)"
description: >
  Register identity on .141 agent-1 session (chat-arc peer-addressability gap)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-03T20:52:09Z
last_update: 2026-05-31T07:12:25Z
date_finished: null
---

# T-1457: Register identity on .141 agent-1 session (chat-arc peer-addressability gap)

## Context

Initial framing was wrong — see Decisions #1 below for correction.

The .141 host **has** a working identity key; FP `6604a2af482f0cf7` posts heartbeats successfully to .141's local agent-chat-arc (33 posts, last 21:37 UTC 2026-05-03). So the WRITE side works.

The actual gap is on the inbound/READ side:

1. `termlink remote exec laptop-141 tl-gibzucwp 'termlink whoami'` returns no `Identity FP:` line — the session metadata lacks the T-1436 identity_fingerprint field.
2. Posting to `dm:6604a2af482f0cf7:d1993c2c3ec44c94` on .141's hub succeeds (offset=1, smoke tested 2026-05-03T20:54Z). The message LANDS.
3. But no subscriber reads it — there is no active Claude on .141 listening to its dm:* topic. The .122 counterpart works because a peer Claude IS attached and reading chat-arc + dm:*.

**Why this matters for the rollout:** "vendored agents in the field" means peer Claudes that can read AND write. .141 has the write half (heartbeat-via-key) but not the read half (no listening agent). Address-ability via `dm:` requires (a) a peer Claude attached to a session on .141, AND (b) that session having T-1436 identity binding so dm topic subscription resolves automatically.

Counterpart on .122 (`tl-vtvvv2tj` / `9219671e28054458`) demonstrates the working pattern — today's interlocutor on `dm:9219671e:d1993c2c` (offsets 6, 7 on ring20-management hub).

## Acceptance Criteria

### Agent
- [x] FP confirmation: 6604a2af482f0cf7 posts as .141's chat-arc participant — verified 2026-05-03T20:54Z (33 posts, last 21:37 UTC)
- [x] Direct dm: post smoke-tested — `agent contact --hub laptop-141 --target-fp 6604a2af482f0cf7` lands on offset=1 of dm:6604a2af482f0cf7:d1993c2c3ec44c94 — verified 2026-05-03T20:54Z
- [ ] `termlink remote exec laptop-141 tl-gibzucwp 'termlink whoami'` shows non-empty `Identity FP: 6604a2af482f0cf7` (requires session re-registration with --identity)

### Human
- [ ] [REVIEW] Decide whether .141 needs a peer Claude attached at all, or whether heartbeat-only target is the desired end state for that host
  **Steps:**
  1. Read this task body + the .122 working pattern
  2. Decide: is .141 a "vendored agent" (needs peer Claude) or a "vendored host" (heartbeat-only is fine)?
  **Expected:** Clear scope decision. If "vendored agent", proceed with operator action below.
  **If not (heartbeat-only target):** Close this task as out-of-scope — current state is the desired end state.

- [ ] [RUBBER-STAMP] Operator action on .141 (only if peer Claude is required)
  **Steps:**
  1. Open SSH or WSL session on .141 as user dimitri
  2. Stop the existing `tl-gibzucwp` agent-1 session (`pkill -f 'termlink register'` or wait for natural exit)
  3. Re-register with explicit identity: `termlink register --name agent-1 --identity-key ~/.termlink/identity.key --tags 'role:agent,host:dimitrixpro'`
  4. Verify: `termlink whoami` shows `Identity FP: 6604a2af482f0cf7`
  5. Attach a Claude Code session that subscribes to chat-arc + dm:6604a2af:* topics
  **Expected:** From .107, `termlink agent contact --hub laptop-141 --target-fp 6604a2af482f0cf7 --message "ping"` produces a reply within ~30s.
  **If not:** Re-check identity key path; verify session metadata via `termlink remote list laptop-141` shows non-`-` FP column.

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
out=$(./target/release/termlink remote exec laptop-141 tl-gibzucwp 'termlink whoami' 2>&1); echo "$out" | grep -E "^Identity FP: [0-9a-f]{16}" >/dev/null
out=$(./target/release/termlink channel members --hub laptop-141 agent-chat-arc 2>&1); { echo "$out" | grep -vE "^d1993c2c|^0000" | grep -cE "^[0-9a-f]{16}" || true; } | (read n; [ "$n" -ge 1 ])

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

## Decisions

### 2026-05-03T20:54Z — Reframe: write-side works, read-side is the gap
- **Chose:** Reframe the task from "register identity on .141" to "decide if peer Claude is needed on .141, and if so, register session metadata + attach Claude"
- **Why:** Smoke test revealed FP 6604a2af482f0cf7 already posts to .141's chat-arc successfully — the write-side is fine. The actual missing piece is a Claude reading dm:* topics, which requires both a session-metadata identity binding (T-1436) AND an attached peer agent. Without the peer Claude, even a perfect identity binding gives nothing — messages would still land in dm: with no reader.
- **Rejected:** Original framing (treat as pure session-registration task) — would have produced session metadata but no readability improvement.

## Updates

### 2026-05-03T20:52:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1457-register-identity-on-141-agent-1-session.md
- **Context:** Initial task creation

### 2026-05-03T20:52:53Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Fix requires session restart on .141 — out of scope for autonomous touch this session; deferring with full reproducer in body.

### 2026-05-31T07:05:16Z — status-update [task-update-agent]
- **Change:** status: issues → started-work

### 2026-05-31T00:40Z — fresh-evidence-for-operator-decision [agent autonomous]
Refreshed evidence so the [REVIEW] taste-call is actionable today.

**.141 chat-arc on laptop-141 hub (write-side):**
- count=466 posts, 3 distinct senders
- 6604a2af482f0cf7 (.141 host): 418 posts — heartbeat-only emitter healthy
- d1993c2c3ec44c94 (.107 self-key): 45 posts — .107-driven cross-posts visible
- 0000000000000000: 1 (null/legacy)

**.141 inbox dm:6604a2af482f0cf7:d1993c2c3ec44c94 (read-side):**
- count=5 envelopes
- receipts=[] — NO reader has ever ack-ed
- Five DMs from .107 → .141 accumulated without being read; structural backpressure-of-silence

**Fleet listeners on laptop-141 hub:**
- 0 listeners reported by `scripts/agent-listeners-fleet.sh`
- No `/be-reachable` heartbeat from any agent on .141

**Implication for operator [REVIEW] decision:**
- "vendored-agent" path: one-line `/be-reachable start` on .141 + a Claude reading dm:* would convert host from broadcast-target to interactive peer. The 5 accumulated DMs would become the inbox.
- "heartbeat-only" path: current state IS the desired end state. Close T-1457 as scoped-out; document .141 as "vendored host" for chat-arc fan-in only.
- Memory `[.141 pickup partial]` already classifies current state as "Hub UP but no peer-agent listener... broadcast-only" — picking heartbeat-only just formalizes the status quo.

**Agent AC 3 (session re-registration on .141)** remains unsatisfied — requires operator SSH/WSL action on .141 host (cannot drive from .107).
