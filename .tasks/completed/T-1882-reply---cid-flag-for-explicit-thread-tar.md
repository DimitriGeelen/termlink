---
id: T-1882
name: "reply: --cid flag for explicit thread targeting"
description: >
  reply: --cid flag for explicit thread targeting

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T16:23:19Z
last_update: 2026-05-30T16:27:09Z
date_finished: 2026-05-30T16:27:09Z
---

# T-1882: reply: --cid flag for explicit thread targeting

## Context

T-1881 made `conversation_id` visible in `/recent-dm` output. Operator can now
SEE distinct threads on a shared `dm:*` topic — but `/reply` still always
auto-extracts the highest-offset cid. When two parallel threads coexist
(cid_A=T-1880, cid_B=T-1881), the operator wanting to reply on the OLDER
thread has no lever: they'd have to shell out to `agent-respond.sh` directly.

This task adds `--cid <CID>` to `/reply` so the operator overrides the
auto-extracted cid with the one they actually want to target. Symmetric to
the existing `--ensure-cid` (mint new) — same level of operator-controlled
threading, just for an existing-but-not-latest cid.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-reply.sh --cid <CID> <peer> "<text>"` skips the auto-extraction step and uses the provided cid verbatim
- [x] `--cid` and `--ensure-cid` are mutually exclusive (exit 2 with a clear message when both passed — they're orthogonal levers)
- [x] `--dry-run --cid <CID>` resolves and prints `cid=<CID>` (proves bypass works without writing)
- [x] `.claude/commands/reply.md` documents `--cid` in the invocation table + a "Pair with /recent-dm" note explaining the typical flow (read cids → pick one → `--cid`)
- [x] Live smoke: `--dry-run --cid test-override-<ts>` on the self-DM topic resolves correctly without firing transport

<!-- All criteria are mechanically verifiable — no Human section. -->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# Help mentions --cid flag.
bash scripts/agent-reply.sh --help 2>&1 | grep -q -- '--cid'
# Mutual exclusion: --cid + --ensure-cid refuses with the right message (agent-reply exits 2 on purpose — swallow before grep).
out_excl="$(bash scripts/agent-reply.sh --cid foo --ensure-cid d1993c2c3ec44c94:d1993c2c3ec44c94 hi 2>&1 || true)"; echo "$out_excl" | grep -q "mutually exclusive"
# --dry-run + --cid bypass works and prints the override cid.
bash scripts/agent-reply.sh --cid test-override-T-1882 --dry-run d1993c2c3ec44c94:d1993c2c3ec44c94 hi 2>&1 | grep -q "cid='test-override-T-1882'\|cid=test-override-T-1882"
# Skill doc documents --cid.
grep -q -- '--cid' .claude/commands/reply.md
# Syntax sanity.
bash -n scripts/agent-reply.sh
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-30T16:23:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1882-reply---cid-flag-for-explicit-thread-tar.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a886616a
- **Timestamp:** 2026-05-30T16:27:10Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **skip-as-pass** (severe, deterministic) @ Verification:line 8
     - evidence: `bash scripts/agent-reply.sh --cid test-override-T-1882 --dry-run d1993c2c3ec44c94:d1993c2c3ec44c94 hi 2>&1 | grep -q "cid='test-override-T-1882'\|cid=test-override-T-1882"`

### 2026-05-30T16:27:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
