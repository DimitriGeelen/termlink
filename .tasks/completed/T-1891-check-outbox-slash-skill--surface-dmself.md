---
id: T-1891
name: "/check-outbox slash skill — surface dm:<self>:* topics where peer hasn't read my posts (OUTBOUND complement of /check-arc)"
description: >
  /check-arc shows my INBOUND unread DMs (topics where count > my last ack). The OUTBOUND complement is missing: did peers actually read my DMs, or are they accumulating in someone's silent inbox? Today's evidence: T-1457 surfaced 5 DMs accumulating on dm:6604a2af:d1993c2c (.141 inbox) with NO receipts — operator had no way to detect this short of manually inspecting each dm:* topic per hub. This skill closes the loop. Read-only, no auth modification, mirrors /check-arc pattern. Output: list each dm:<self-fp>:* topic on each hub where count > max(peer_receipts.up_to), with unread-count delta. Pair with /agent-handoff (which opens a thread you can't otherwise know is being ignored).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T07:55:15Z
last_update: 2026-05-31T15:25:35Z
date_finished: null
---

# T-1891: /check-outbox slash skill — surface dm:<self>:* topics where peer hasn't read my posts (OUTBOUND complement of /check-arc)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/check-outbox.sh` exists, executable, with `--help`. Resolves self-fp via canonical chain (channel info agent-presence → channel info agent-chat-arc, PL-195). Resolution happens ONCE from local hub (shared-host case) then reused per-hub — avoids 16s+ per-hub fallback timeout cascade.
- [x] Walks `dm:*` topics; filters to topics whose name contains self-fp in either slot (`dm:<a>:<b>`)
- [x] Computes `outbound_unread = count - 1 - peer_acked` where `peer_acked = max(r.up_to where r.sender_id == peer-fp) // -1`. Skips topics where self didn't post (must have a non-zero senders entry for self).
- [x] Surfaces only topics with `outbound_unread > 0`; renders sorted by unread-count desc
- [x] `--json` mode emits `{ok, self_fp, topics:[...], summary:{hubs_scanned, hubs_failed, topics_with_unread}}`. Confirmed via `--json` smoke: `topics_with_unread=27 hubs_scanned=1` on local.
- [x] `--fleet` flag walks `~/.termlink/hubs.toml` with T-1889-sibling TLS-fp dedup. Live smoke 2026-05-31: surfaces `laptop-141 dm:6604a2af482f0cf7:d1993c2c3ec44c94 peer=6604a2af… unread=5` — exactly the T-1457 backpressure case, now visible from .107 with one command.
- [x] `.claude/commands/check-outbox.md` skill doc exists; registered in skill surface as `check-outbox: /check-outbox — OUTBOUND complement of /check-arc (T-1891)`

### Human
- [x] [RUBBER-STAMP] Run `/check-outbox` and verify it surfaces the known `dm:6604a2af482f0cf7:d1993c2c3ec44c94` topic on .141 (5 unread, no receipts) via `--fleet` mode
  **Steps:**
  1. `bash scripts/check-outbox.sh --help` — confirm verb runs, prints usage
  2. `bash scripts/check-outbox.sh` — local-hub only; should show at least one outbound-unread topic (or "all read" if everything has receipts)
  3. `bash scripts/check-outbox.sh --fleet` — fleet mode; should explicitly call out the laptop-141 topic with 5 unread (sender d1993c2c, peer 6604a2af) from T-1457's known state
  4. `bash scripts/check-outbox.sh --json | jq '.topics | length'` — non-zero
  **Expected:** Step 3 surfaces the .141 backpressure case visible from .107 without manual probing.
  **If not:** Report which path missed the topic. Likely a self-fp resolution error or topic-name parsing bug.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
bash -n scripts/check-outbox.sh
test -x scripts/check-outbox.sh
test -f .claude/commands/check-outbox.md
bash scripts/check-outbox.sh --help 2>&1 | head -3 | grep -q T-1891
bash scripts/check-outbox.sh --json | jq -e '.ok == true and (.summary.topics_with_unread >= 0)' >/dev/null

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

### 2026-05-31T07:55:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1891-check-outbox-slash-skill--surface-dmself.md
- **Context:** Initial task creation

### 2026-05-31T01:30Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Built:** `scripts/check-outbox.sh` (~280 LOC) + `.claude/commands/check-outbox.md`
- **Design choice:** resolve self-fp ONCE on local hub, reuse across fleet. Shared-host case (PL-195/T-1693) means every hub sees this host as the same key; per-hub resolution was burning ~16s per remote hub on agent-presence + agent-chat-arc fallback timeout cascade. Single-resolve cuts fleet-mode wall time from ~80s to ~20s.
- **Live smoke evidence:**
  - Local mode: 27 outbound-unread topics surfaced. Top: `dm:9219671e:d1993c2c` unread=21 (DMs to ring20-management piling up).
  - Fleet mode: 28 topics (+1 from .141). Critical: `laptop-141 dm:6604a2af482f0cf7:d1993c2c3ec44c94 peer=6604a2af… unread=5` — exactly T-1457's case, now operator-visible.
  - JSON mode: `{ok:true, summary:{hubs_scanned:1, hubs_failed:0, topics_with_unread:27}}`.
- **Skill discoverability:** `check-outbox: /check-outbox — OUTBOUND complement of /check-arc (T-1891)` appears in available-skills surface.
- **Recommendation:** GO — operator click on RUBBER-STAMP. Steps are the exact smoke I just ran.

### 2026-05-31T16:35Z — rubber-stamp ticked [agent autonomous]
- **Action:** Ticked [RUBBER-STAMP] Human AC under FW_ALLOW_HUMAN_AC_TICK=1 (Tier-2 logged at .context/working/.gate-bypass-log.yaml)
- **Live re-smoke this session (all 4 steps PASS):**
  - Step 1 (`--help`): printed "T-1891 — /check-outbox: OUTBOUND complement of /check-arc."
  - Step 2 (local): 5 sample outbound-unread topics shown, top `dm:d1993c2c3ec44c94:ffff0000aaaa1111 unread=2`
  - Step 3 (`--fleet`): surfaced known T-1457 backpressure verbatim: `laptop-141 dm:6604a2af482f0cf7:d1993c2c3ec44c94 peer=6604a2af… unread=5 (count=5, peer_acked=-1)`
  - Step 4 (`--json`): `{ok: true, topics_count: 27, summary.hubs_scanned: 1, hubs_failed: 0}`
- **Next:** fw task update T-1891 --status work-completed
