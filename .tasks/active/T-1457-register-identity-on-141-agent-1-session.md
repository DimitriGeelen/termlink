---
id: T-1457
name: "Register identity on .141 agent-1 session (chat-arc peer-addressability gap)"
description: >
  Register identity on .141 agent-1 session (chat-arc peer-addressability gap)

status: issues
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-03T20:52:09Z
last_update: 2026-05-03T20:52:53Z
date_finished: null
---

# T-1457: Register identity on .141 agent-1 session (chat-arc peer-addressability gap)

## Context

`termlink remote exec laptop-141 tl-gibzucwp 'termlink whoami'` returns no `Identity FP:` line. The `agent-1` session on .141 is registered without an identity key, so it cannot post to `agent-chat-arc` under its own identity (T-1427 strict-reject would reject the post with -32014). Today the .141 heartbeat lands as `d1993c2c…` (the .107 driver's identity, via `termlink remote exec`-driven posts), not as a .141-resident identity. This makes .141 a chat-arc TARGET (heartbeat lands there because hub state is local) but not a chat-arc PARTICIPANT (no peer agent on .141 can be addressed via `dm:<sorted_a>:<sorted_b>` because there's no FP).

Concretely: `termlink agent contact --hub laptop-141 --target-fp <??>` fails because there is no `<??>` to give. T-1429 Phase-1 contact assumes the peer has an identity key registered (T-1436 metadata).

Counterpart on .122 (`tl-vtvvv2tj` / `9219671e28054458`) demonstrates the working pattern — that session was today's interlocutor on the dm:9219671e:d1993c2c topic (offsets 6, 7 on ring20-management hub).

**Why this matters for the rollout:** "vendored agents in the field" means peer Claudes that can read AND write chat-arc / dm:* — not just sessions that have heartbeat fired at them. .141 currently fails the WRITE side under T-1427 enforcement.

## Acceptance Criteria

### Agent
- [ ] `termlink remote exec laptop-141 tl-gibzucwp 'termlink whoami'` shows a non-empty `Identity FP:` line
- [ ] A direct heartbeat-equivalent post from the .141 session (not driven from .107) lands on `agent-chat-arc` with that FP as `sender_id`
- [ ] `termlink agent contact --hub laptop-141 --target-fp <new-fp> --message "ping"` succeeds (does not return -32014)

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

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-03T20:52:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1457-register-identity-on-141-agent-1-session.md
- **Context:** Initial task creation

### 2026-05-03T20:52:53Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Fix requires session restart on .141 — out of scope for autonomous touch this session; deferring with full reproducer in body.
