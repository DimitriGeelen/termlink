---
id: T-2731
name: "chat-arc-recent reports ok:true with 0 posts on a degraded partial read"
description: >
  chat-arc-recent reports ok:true with 0 posts on a degraded partial read

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/agent-chat-arc-recent.sh, tests/chat-arc-recent-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T09:20:09Z
last_update: 2026-08-15T09:27:59Z
date_finished: 2026-08-15T09:27:59Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2731: chat-arc-recent reports ok:true with 0 posts on a degraded partial read

## Context

Found by using the tool, not by reading it. Checking messages this session,
`termlink_agent_chat_arc_recent` returned:

```json
{"ok": true, "summary": {"total_posts": 0, "hubs_scanned": 4, "hubs_failed": 1,
 "failed_hubs": [{"hub": "laptop-141", "reason": "network"}],
 "fallback_hubs": ["ring20-management", "ring20-dashboard"]}}
```

`ok:true` and `total_posts:0` read as "the fleet is quiet". They do not mean
that. One hub was unreachable and two more served a **partial head-read**,
because their stale binaries do not implement seek-to-tail (T-1872 fallback
path). So of four hubs scanned, exactly one produced a complete answer. The
returned zero is consistent with a quiet fleet AND with a busy fleet whose
traffic is entirely on the three degraded hubs — the read cannot distinguish
them, but nothing in its verdict says so.

The information is present — `hubs_failed`, `failed_hubs`, `fallback_hubs` are
all in the envelope, and human mode prints `failed:` / `fallback:` lines. Two
things still make it easy to miss:

1. **No single field states completeness.** A caller must know that
   `fallback_hubs` non-empty implies partial data — domain knowledge encoded
   nowhere in the envelope. Every caller re-derives it or forgets to.
2. **Human mode still asserts the absence.** After printing the degradation
   lines it prints `(no posts matched filters)`, which claims the filters
   matched nothing — a statement about the *fleet*, when the truthful statement
   is about the *data retrieved*.

This is the session's recurring class (T-2726, T-2729, T-2680, T-2709) sitting
on the comms path: **a verdict resting on an assumption about its input —
here, that the read was complete — which no longer holds.** It has come close
to misleading me twice in two sessions; I only avoided it by reading the
summary fields by hand.

Absence of evidence from a degraded read is not evidence of absence. The output
should be incapable of being read as the latter.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The JSON summary carries a single unambiguous completeness field
      (`read_complete`) that is false when ANY hub failed OR any hub served the
      partial fallback path — so a caller can gate on one field without knowing
      what `fallback_hubs` implies
- [x] The summary names WHY completeness is false, machine-readably
      (`degraded_reasons`), rather than requiring the caller to diff two lists
- [x] On a degraded read with zero results, human mode does NOT print a bare
      "no posts matched filters" — it states that the retrieved data is empty
      AND that absence is not established, naming the hubs responsible
- [x] On a complete read, output is unchanged — no new warning noise on the
      healthy path (an alert that fires always is PL-219 alert fatigue)
- [x] Fixture tests cover: complete read, failed-hub read, fallback-hub read,
      and both-at-once — and FAIL against the pre-fix script

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
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
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# 9 assertions; 5..9 are T-2731's. Hermetic — mock termlink, no hub, no network.
bash tests/chat-arc-recent-fixtures.sh

# The whole guard layer, which this suite belongs to.
bash scripts/run-guard-layer.sh

# The degraded path must never fall back to the bare absence claim. Both
# strings must exist: one for the degraded branch, one for the healthy branch.
grep -q "absence is NOT established" scripts/agent-chat-arc-recent.sh
grep -q "no posts matched filters" scripts/agent-chat-arc-recent.sh

## RCA

**Symptom.** `termlink_agent_chat_arc_recent` returned `{ok:true,
total_posts:0}` for a 48h window across 4 hubs while 1 hub was unreachable and
2 served a partial head-read. Read as "the fleet is quiet"; it actually meant
"1 of 4 hubs gave a complete answer, and it was empty".

**Root cause.** The envelope carried the raw ingredients of degradation
(`hubs_failed`, `failed_hubs`, `fallback_hubs`) but no field stating the
conclusion. Knowing that a non-empty `fallback_hubs` implies missing recent
posts is domain knowledge about T-1872's fallback that lives nowhere in the
output, so each caller must re-derive it. Human mode compounded it: having
printed the degradation lines, it still printed `(no posts matched filters)` —
a claim about the fleet where only a claim about the retrieved data was
supportable.

**Why structurally allowed.** `ok` conflates two questions — "did the command
run?" and "is the answer complete?" — and only ever answered the first. Nothing
tested the zero-results path at all: the existing 4 fixtures all assert on
*present* posts, so the branch that renders emptiness was unexercised. A branch
no test enters is a branch whose wording can drift into a false claim without
anything noticing.

**Prevention.** `read_complete` (single boolean) + `degraded_reasons` (named
causes) in the summary; the human branch splits on degradation. Fixtures 5–9
cover complete / fallback / failed / degraded-human / healthy-human, and
**4 of the 5 fail against the pre-fix script** (the 5th asserts the healthy path
is *unchanged*, so passing both is correct). Suite is in the guard layer, so CI
runs it per push. Origin note: found by *using* the tool during the routine
message check, not by reading it — the two prior sessions each came within one
step of recording "fleet is quiet" from this exact output.

## Follow-up — the fix is shipped but NOT live on the MCP surface

Verified after the fix: calling `termlink_agent_chat_arc_recent` still returns
the pre-fix envelope, and its `stdout` field is verbatim old-format. **The MCP
server executes a different copy of this script than the repo working tree**
(installed `TERMLINK_SCRIPTS_DIR`, not the worktree). So the surface where this
defect was actually encountered still has it.

That is the G-069 shipped≠live class, and this task does not close it — the
remediation is operator-side (point the MCP server's scripts dir at the current
tree, or reinstall + restart it; the running binary here is 0.11.720 against a
0.11.13xx tree, which preflight Check 4/5 already WARNs about). Recorded rather
than silently counted as done: the code fix is complete and tested, the
deployment is not, and the ACs above claim only the former.

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

### 2026-08-15T09:20:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2731-chat-arc-recent-reports-oktrue-with-0-po.md
- **Context:** Initial task creation

### 2026-08-15T09:27:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
