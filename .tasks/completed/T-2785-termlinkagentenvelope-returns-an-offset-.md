---
id: T-2785
name: "termlink_agent_envelope returns an offset with no hub — G-060 makes that a plausible wrong answer"
description: >
  termlink_agent_envelope returns an offset with no hub — G-060 makes that a plausible wrong answer

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-17T14:47:31Z
last_update: 2026-08-17T15:09:26Z
date_finished: 2026-08-17T15:09:26Z
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

# T-2785: termlink_agent_envelope returns an offset with no hub — G-060 makes that a plausible wrong answer

## Context

`termlink_agent_envelope` is the forensic deep-fetch verb: *"what exactly was at offset X
with all fields?"*. Its params are **`{offset}` and nothing else** (`tools.rs:9175-9178`).
The topic is hardcoded to `agent-chat-arc` (`:25565`) and the hub is whatever
`hub_socket_path()` resolves to locally (`:25561`).

Per **G-060**, a topic name is per-hub state with no federation primitive — the same name
on two hubs is unrelated state. Measured on this fleet right now: `agent-chat-arc` holds
**8951 messages on ring20-dashboard** and **3577 on ring20-management**. So the same
offset denotes *different envelopes* depending on the hub, and this tool answers with a
confident `found: true` while naming neither the hub nor the topic it read.

That is the Directive #2 shape: not an error, a **plausible wrong answer**. An agent
handed a peer's citation ("see offset 3400 on agent-chat-arc") calls this tool, gets a
different message from its own local log, and nothing in the response reveals the
substitution. It is the same defect class T-2782/T-2783 just closed for `agent inbox` — a
read verb whose output does not state its scope — in the sibling verb.

Two consequences, both fixed here:
- **No hub selection.** The caller cannot ask the hub the citation came from.
- **No topic selection.** The main forensic target on this fleet is a `dm:` thread, and
  this verb structurally cannot fetch one.

**Not in scope:** the paging cost. This walks the whole topic in 1000-message pages to
find one offset (9 round-trips on `.121`) where a cursor-seek would take one. Real, but a
separate performance concern from the correctness defect; recorded in Evidence.

## Acceptance Criteria

### Agent
- [x] `AgentEnvelopeParams` gains optional `hub` (address; `None` = local hub) and optional
      `topic` (`None` = `agent-chat-arc`, preserving current default behaviour)
- [x] The success response always carries `hub` and `topic` naming what was actually read,
      so an offset is never reported without the scope that makes it meaningful
- [x] The `found: false` response also carries `hub` and `topic` — a miss is exactly when
      the caller most needs to know whether they asked the wrong hub
- [x] The tool description states the G-060 scope (offset is meaningless without a hub)
      instead of implying a fleet-wide `agent-chat-arc`
- [x] Tests lock all three: hub echoed on hit, hub echoed on miss, topic default preserved
- [x] Systemic-scope measurement recorded in Evidence: how many MCP tools resolve
      `hub_socket_path()` vs how many expose a hub param — if the gap is broad, it is
      FILED as a separate finding, not silently fixed here
- [x] `cargo test -p termlink-mcp` passes with 0 failures

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
-->

## Evidence

### Systemic-scope measurement (AC-6)

Measured on `crates/termlink-mcp/src/tools.rs`, 2026-08-17:

| measure | count |
|---|---|
| `#[tool(name = "termlink_…")]` declarations | 262 |
| call sites resolving `hub_socket_path()` (local hub) | 176 |
| params structs exposing `pub hub: Option<String>` | 8 |
| `CHANNEL_SUBSCRIBE` call sites (per-hub topic reads) | 70 |

The gap is **broad but not uniformly a defect**. Many of the 176 are legitimately
local-only: hub lifecycle (`hub_start`/`hub_stop`/`hub_status`), session control, and
`queue_status` (a local SQLite read by design, T-2051). The defect class is narrower and
specific: **a read verb over per-hub topic state that reports a result without naming the
hub.** The 70 `CHANNEL_SUBSCRIBE` sites bound that class from above.

Per AC-6 this is **filed, not silently fixed here** — see T-2786. Fixing 70 call sites
under one task would be exactly the unscoped build G-020 exists to prevent, and each verb
needs its own judgement about whether a `hub` param is meaningful for it.

### Mutation proof

The stamp is load-bearing, not decorative. Removing `hub`/`topic` from the miss branch of
`envelope_response_json`:

```
test tools::tests::envelope_miss_still_names_the_hub_and_topic ... FAILED
test result: FAILED. 0 passed; 1 failed
```

Restored; suite returns green.

### Full-suite run (AC-7)

`cargo test -p termlink-mcp`, 2026-08-17, exit 0:

```
test result: ok. 934 passed; 0 failed   (lib)
test result: ok.  99 passed; 0 failed   (mcp_integration)
test result: ok.  28 passed; 0 failed   (parity, 804.30s)
```

The `## Verification` block runs `--lib` only. That is a deliberate, stated trade-off:
parity.rs takes 804s and re-running it in the P-011 gate would stall completion. The
fast check is not a substitute for the full run — the full run is the evidence above.

### Not fixed here, deliberately

- **Paging cost.** The verb still walks the whole topic in 1000-message pages to find one
  offset — 9 round-trips on `.121`'s 8951-message `agent-chat-arc` where a cursor-seek
  would take one. A performance concern, orthogonal to the correctness defect. The bus
  already has the primitive (`Bus::envelope_at`, T-2109); wiring it through is separate.
- **Remote auth surface.** The remote path uses `connect_remote_hub_mcp(hub, None, None,
  "observe")`, i.e. profile-resolved credentials only — no `secret_file`/`secret`
  overrides. That matches how a forensic reader should work (the hub is already a
  configured profile) and keeps credentials out of tool params.

## Verification

# The four T-2785 scope tests exist and pass.
out=$(cargo test -p termlink-mcp envelope 2>&1 || true); grep -q "0 failed" <<< "$out"
out=$(cargo test -p termlink-mcp envelope_miss_still_names_the_hub_and_topic 2>&1 || true); grep -q "1 passed" <<< "$out"
out=$(cargo test -p termlink-mcp same_offset_on_two_hubs 2>&1 || true); grep -q "1 passed" <<< "$out"
# Params carry hub + topic.
out=$(sed -n '/pub struct AgentEnvelopeParams/,/^}/p' crates/termlink-mcp/src/tools.rs); grep -q "pub hub: Option<String>" <<< "$out"
out=$(sed -n '/pub struct AgentEnvelopeParams/,/^}/p' crates/termlink-mcp/src/tools.rs); grep -q "pub topic: Option<String>" <<< "$out"
# The description states the G-060 scope rather than implying a fleet-wide topic.
out=$(grep -A2 'name = "termlink_agent_envelope"' crates/termlink-mcp/src/tools.rs); grep -q "G-060" <<< "$out"
# Follow-up for the systemic class is filed, not dropped.
out=$(cat .tasks/active/T-2785-termlinkagentenvelope-returns-an-offset-.md); grep -q "T-2786" <<< "$out"
# Whole lib green (934 tests, ~2s). The FULL crate suite — lib + mcp_integration +
# parity — was run once for AC-7 and recorded under Evidence; it is deliberately NOT
# re-run here because parity.rs alone takes 804s and would stall the P-011 gate.
out=$(cargo test -p termlink-mcp --lib 2>&1 || true); grep -q "0 failed" <<< "$out"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

### 2026-08-17T14:47:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2785-termlinkagentenvelope-returns-an-offset-.md
- **Context:** Initial task creation

### 2026-08-17T15:09:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
