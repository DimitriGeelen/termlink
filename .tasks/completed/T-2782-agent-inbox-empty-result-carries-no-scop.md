---
id: T-2782
name: "agent inbox empty result carries no scope — reads as no mail when it means no locally-tracked topics on this hub"
description: >
  termlink agent inbox returned unread_topics:[] while a full reply sat unread on a peer hub, causing two redundant escalations sent over the top of an answer. The scope limitation is documented in the MCP tool DESCRIPTION but absent from every OUTPUT path, so the answer reads as complete. Same shape as T-2680 (charter-drift canary reporting 214 checked / 0 off-charter while only scanning six known families) — fixed there by making every output path carry an explicit scope disclaimer. Apply that convention here.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-17T10:27:42Z
last_update: 2026-08-17T10:54:04Z
date_finished: 2026-08-17T10:54:04Z
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

# T-2782: agent inbox empty result carries no scope — reads as no mail when it means no locally-tracked topics on this hub

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Every** output path carries the scope — the empty-cursor early-out, the populated path, and the "scanned but nothing unread" path. The empty path is the one that matters most: an empty inbox is precisely when an operator stops looking, so it is the costliest place for a bare `[]`
- [x] The envelope gains machine-readable scope fields, not just prose: at minimum `hub` (which hub was actually read) and `topics_scanned` (how many cursor-store topics were considered). A caller must be able to tell "0 unread of 9 tracked" from "0 tracked, nothing was looked at" — those are different facts and today both render as `[]`
- [x] The empty path additionally carries an actionable `hint` naming the two blind spots by their remedy: this is one hub (no `--fleet`, unlike `check-outbox`), and only `subscribe --resume`-tracked topics are enumerated — so `/recent-dm <peer>` or a direct per-hub `channel subscribe` is what actually answers "did they reply?"
- [x] Affirmative rather than silent on the healthy path, per the T-2076 convention already used by `claims-summary --only-stuck` (`All topics healthy (0/N stuck)`): a clean inbox states what was checked, e.g. `No unread across 9 tracked topic(s) on <hub>`
- [x] MCP and CLI stay in parity — **delivered, but NOT as this AC was written, and the difference is deliberate.** I wrote "both surfaces gain the same fields" before reading the CLI's JSON contract. `agent inbox --json` emits a bare ARRAY, and two consumers parse it: `scripts/substrate-worker-pickup.sh:290` and `docs/operations/substrate-orchestrator-recipe.md`, the latter documenting the array shape explicitly (T-2153). Wrapping it in an envelope is the tidier design and would break both. So the CLI carries the scope on **stderr** in `--json` mode (stdout byte-identical, machine contract untouched) and inline on the human path; the MCP envelope, which already exists, gains the fields additively. Same information on both surfaces, different transport, chosen by what each surface has promised its callers
- [x] Regression test pins the property that actually failed: an empty result **must** carry the scope fields. A test asserting only `unread_topics == []` would have passed throughout this defect's life and is not a test of it
- [x] Load-bearing by mutation: removing the scope fields makes the new test(s) fail; restoring passes. Not argued — run and reported

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

cargo test -p termlink -p termlink-mcp inbox_scope --quiet
cargo test -p termlink-mcp inbox_envelope --quiet
out=$(cargo test -p termlink-mcp inbox 2>&1 || true); grep -q "empty_inbox_envelope_states_what_was_examined ... ok" <<< "$out"
out=$(cargo test -p termlink inbox_scope 2>&1 || true); grep -q "inbox_scope_note_names_both_blind_spots_and_a_remedy ... ok" <<< "$out"
grep -qF 'eprintln!("{}", inbox_scope_note(cursors.len(), hub));' crates/termlink-cli/src/commands/channel.rs
grep -qF 'fn inbox_envelope_json' crates/termlink-mcp/src/tools.rs

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

**Symptom.** `termlink agent inbox` returned `{ok:true, unread_topics:[]}` while a full,
actionable reply from ring20-manager sat unread on hub `.122`. Read as "no mail", it led to
two further escalations sent over the top of an answer that was already waiting — roughly
half an hour of wasted round-trips, and a peer told twice that they had not responded when
they had.

**Root cause.** The verb has two independent blind spots and neither appeared in its result:

1. It reads ONE hub. There is no `--fleet` (its sibling `check-outbox` has one), and topics
   are per-hub state with no federation primitive (G-060) — so the DM thread did not merely
   read as unread-free on `.107`, it does not exist there. Verified by grep of
   `channel list --json`: zero matches.
2. It enumerates only topics present in the local cursor store, which `subscribe --resume`
   populates. A topic I had only ever POSTED to was never resumed, so it was never tracked.
   Verified: 9 cursor entries, none matching the topic.

**Why structurally allowed.** Both limitations were already written down — in the clap doc
comment (`"Still cursor-SCOPED — ... not a whole-hub view"`) and, almost verbatim, in the MCP
tool's `description`. So this was never an undiscovered fact; it was a **documented fact that
never reached the answer**. A caller who had not read the help got a result that looked
complete, and there is no point in the call path where the help is in front of you. That is
the T-2680 shape exactly: `{checked:214, live_off_charter:0}` was true, and read as "all 214
trace to the charter" when only six families had been examined. The cost lands hardest on the
empty result, because an empty inbox is the precise moment an operator stops looking.

**Prevention** (distinct from the fix): three regression tests that pin the property which
actually failed, not the one that always worked. `unread_topics` was correct throughout this
defect's life — a test asserting `unread_topics == []` would have passed every day. The new
tests assert that an empty result *carries its scope*, that the populated path carries the
same (so the disclaimer cannot quietly disappear once mail exists), and that the text names
**both** blind spots plus a command that actually answers "did they reply?". Mutation-proven:
dropping the fields from the MCP envelope fails 2 tests; emptying the CLI note fails 2 more.

**Not fixed here, deliberately.** The verb is still one-hub and still cursor-gated — this
makes it *honest*, not complete. A `--fleet` mode is a real capability change with its own
cost (per-hub timeouts, the TLS-fp dedup `check-outbox --fleet` already needed) and belongs
in its own task rather than smuggled in behind a truthfulness fix.

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

### 2026-08-17T10:27:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2782-agent-inbox-empty-result-carries-no-scop.md
- **Context:** Initial task creation

### 2026-08-17T10:54:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
