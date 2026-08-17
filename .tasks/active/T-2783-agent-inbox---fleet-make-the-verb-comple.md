---
id: T-2783
name: "agent inbox --fleet: make the verb complete, not just honest"
description: >
  T-2782 made agent inbox state its scope; it is still one-hub and cursor-store gated. This is the capability half: walk hubs.toml like check-outbox --fleet (per-hub timeout, TLS-fp dedup), and consider discovering dm:* topics that carry the local fingerprint regardless of whether the cursor store tracks them — the case that actually hid the ring20 reply. Deliberately separated from T-2782 so a truthfulness fix was not blocked on a capability change.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-17T10:56:18Z
last_update: 2026-08-17T12:00:51Z
date_finished: null
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

# T-2783: agent inbox --fleet: make the verb complete, not just honest

## Context

T-2782 made `agent inbox` **honest** — every output path now states its scope. It is still
**incomplete**: one hub, and only topics the local cursor store tracks. This task closes the
capability half.

**Scoping correction made while writing these ACs.** The task was filed as "`--fleet`", with
untracked-topic discovery as a secondary "consider". That ordering is backwards. The reply
that actually went missing (T-2781, ring20-concierge on `.122`) lived on topic
`dm:88743a9ad59fda39:d1993c2c3ec44c94`, which **was never in the cursor store** — it is a
topic this identity had only ever *posted* to, and `subscribe --resume` is what writes a
cursor row. `cursor_store::list_for_fingerprint(&fp)` returns nothing for it, so
`cmd_channel_inbox` never examines it. **A `--fleet` walk would still have missed it**, on
`.122` as surely as on `.107`, because the enumeration source — not the hub — is what
excluded it. Discovery is therefore the primary fix and `--fleet` the secondary one.

This is **PL-350** ("when two stores can answer the same question, something must JOIN them
or the answer is silently partial"): the local cursor store and the hub's own `channel.list`
both know which topics concern this identity, and `inbox` consults only the first. The join
is cheap — `channel.list` is *already fetched* on the existing path (`channel.rs:8994`) and
already carries `name` + `count` + `latest_offset` for every topic. The missing piece is a
name match against this identity's fingerprint, not a new round trip.

**Blind spot ordering, restated for the implementer:**

1. **Cursor-store gated** (primary — this is the one that cost two redundant escalations).
   A `dm:*` topic carrying our fingerprint is *by construction* addressed to us. If it has no
   cursor row, the correct reading is "everything in it is unread", not "it does not exist".
2. **One hub** (secondary). Topics are per-hub state with no federation (G-060), so a thread
   on a peer's hub is invisible. `check-outbox --fleet` already solved this shape.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Untracked `dm:*` topics carrying this identity's fingerprint are discovered and
      reported.** `cmd_channel_inbox` joins the already-fetched `channel.list` against the
      cursor store: a topic whose name matches `dm:*` and contains this identity's
      fingerprint but has **no** cursor row is treated as cursor=0 (all unread) rather than
      skipped. Regression test reproduces the T-2781 shape exactly — a dm topic with posts
      and no cursor row must appear in the result.
      **Done on BOTH surfaces** (`discover_untracked_dm_topics` /
      `discover_untracked_dm_topics_mcp`). Scope grew during build: MCP was not in the
      original ACs, and leaving it would have left the incident's actual cause in place on
      the surface agents actually read their mail through, while the tree looked fixed.
- [x] **Untracked rows are visibly distinguished from tracked ones**, in both human and JSON
      output. "Never resumed, so all N unread" and "resumed and N behind" are different
      operational facts and must not render identically; conflating them would re-create a
      plausible-wrong-answer at the row level while fixing it at the verb level.
      Human: `[DISCOVERED — never subscribe --resume`d …]`. JSON: `tracked: false`.
      Rows also carry `hub`, without which a merged fleet result would read as one thread
      reported twice (G-060).
- [x] **Receipt frontier is honoured for untracked topics too.** The T-2757 join
      (`fetch_receipt_frontier`) runs for discovered topics as well, so a topic already
      acked via receipts does not resurface as spuriously unread. Without this, discovery
      trades a false negative for a false positive.
      Proven live, not just in unit test: the discovered ring20 row rendered
      `cursor=0 superseded by receipt=23`, i.e. 23 already-acked offsets correctly excluded.
- [x] **`--fleet` walks every profile in `~/.termlink/hubs.toml`**, bounded per hub
      (`timeout`, PL-189), deduping profiles that resolve to the same TLS fingerprint — the
      same rule `scripts/check-outbox.sh --fleet` uses. A hub that fails is **reported**, not
      silently dropped (an omitted hub is precisely this task's bug class recurring one layer
      up). Live run skipped `workstation-107-public` as a duplicate of `local-test`, and
      named `laptop-141` as unreadable.
- [x] **The T-2782 scope note is derived from actual scope, never asserted.** It currently
      hardcodes `one hub only (no --fleet)` and `only topics recorded by subscribe --resume`.
      Both claims become FALSE the moment this task lands, so the note must be regenerated
      from what was really read (which hubs, how many tracked + how many discovered). Test
      pins that a `--fleet` run's note does not contain the single-hub claim. Shipping the
      capability while leaving the note stale would reintroduce the exact T-2680 defect
      T-2782 existed to fix — in the sentence written to prevent it.
      Both surfaces updated; MCP's note now states plainly that IT still has no `--fleet`
      rather than implying parity the CLI has and it does not.
- [x] **The machine contract is preserved.** `agent inbox --json` still emits a **bare array**
      on stdout (`scripts/substrate-worker-pickup.sh:290` and the T-2153 orchestrator recipe
      both parse it); all scope output stays on stderr. Test pins stdout shape.
- [x] **Mutation-proven load-bearing.** Removing the discovery join makes a named test go red;
      removing the fleet TLS-fp dedup makes a named test go red. Both restored with zero
      residue. Evidence recorded in this file with the date it was run.
- [x] `cargo test --workspace` exits 0 with 0 failures, and `bash scripts/run-guard-layer.sh`
      reports all members clean.

## Evidence

### Mutation proof — RAN 2026-08-17 (`bash tests/mutate-2783.sh`)

Four mutations, each removing exactly one guarded behaviour. **All four detected; clean
restore (`git diff --stat` empty).**

| # | Mutation | Result |
|---|---|---|
| M1 | CLI discovery join returns nothing | RED — 2 tests failed |
| M2 | MCP discovery join returns nothing | RED — 3 tests failed |
| M3 | fleet TLS-fp dedup keeps every profile | RED — 2 tests failed |
| M4 | scope note emptied (the T-2782 guard) | RED — 3 tests failed |

M4 is included deliberately: this task rewrote T-2782's note, and a rewrite is exactly when
a guard quietly stops being load-bearing.

### Live fleet run — RAN 2026-08-17

`./target/debug/termlink channel inbox --fleet` against the real fleet. **The T-2781 topic
appeared**, which is the whole point of the task:

```
(skipping profile 'workstation-107-public' — same hub as an already-read profile)
5 topic(s) with unread content:
  [ring20-dashboard]  agent-chat-arc — 7338 unread (latest=8949, cursor=1611, no receipt)
  [ring20-management] agent-chat-arc — 1966 unread (latest=3577, cursor=1611, no receipt)
  [ring20-management] dm:88743a9ad59fda39:d1993c2c3ec44c94 — 5 unread
        (latest=28, cursor=0 superseded by receipt=23) [DISCOVERED — …]
  [ring20-management] dm:9219671e28054458:d1993c2c3ec44c94 — 2 unread
        (latest=125, cursor=44 superseded by receipt=123)
  [ring20-dashboard]  dm:33df8954b2a9b70d:d1993c2c3ec44c94 — 1 unread
        (latest=1, cursor=0, no receipt) [DISCOVERED — …]
scope: read 3 hub(s) from hubs.toml: local-test, ring20-dashboard, ring20-management;
9 cursor-tracked topic(s) + 2 discovered dm topic(s) addressed to this identity.
WARNING: 1 hub(s) could NOT be read, so mail there is neither counted nor ruled out:
laptop-141 (Hub rpc_call (channel.list) failed). …
```

Both DISCOVERED rows are threads that had to be found **by hand** earlier the same session
because the verb could not see them. The single-hub run on `.107` correctly reports
`0 discovered` — there are no dm topics carrying this fingerprint on that hub at all, which
is precisely why `--fleet` was needed *and* why `--fleet` alone would not have sufficed.

**It earned its keep immediately.** The run surfaced offsets 27-28 on the ring20 topic,
posted after the manual sweep — a reply confirming a live plaintext credential had been
shredded. Found by the tool, not by hand.

### Not fixed here, deliberately

- **MCP has no `--fleet`.** The tool takes no `hub` parameter at all, so this is a larger
  change (profile walk + per-hub auth inside the MCP server). Its scope note now says so
  explicitly instead of implying parity.
- **Non-dm untracked topics are still not enumerated.** Discovery is restricted to `dm:`
  because a broad topic is addressed to nobody in particular; auto-adding every untracked
  topic would make the digest a firehose and teach its reader to ignore it (T-2709). Stated
  in the scope note on every output path rather than left to be rediscovered.
- **Own posts count toward unread on a receipt-frontier topic** (the ring20 row's 5 includes
  posts made by this identity). Pre-existing behaviour of the frontier model, not a
  regression from this change; noted here so the next reader does not re-derive it.

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

# --- AC1/AC2/AC3: discovery, marking, receipt join (CLI + MCP) ---
out=$(cargo test -p termlink discover 2>&1 || true); grep -q "discovers_the_untracked_dm_topic_that_hid_the_reply ... ok" <<< "$out"
out=$(cargo test -p termlink discover 2>&1 || true); grep -q "discovered_rows_are_marked_untracked_and_start_at_cursor_zero ... ok" <<< "$out"
out=$(cargo test -p termlink discover 2>&1 || true); grep -q "discovered_topic_already_acked_via_receipts_is_not_reported_unread ... ok" <<< "$out"
out=$(cargo test -p termlink-mcp mcp_discover 2>&1 || true); grep -q "mcp_discovers_the_untracked_dm_topic_that_hid_the_reply ... ok" <<< "$out"
out=$(cargo test -p termlink-mcp mcp_discover 2>&1 || true); grep -q "mcp_discovery_agrees_with_cli_on_the_same_input ... ok" <<< "$out"

# --- AC4: --fleet walk, dedup, and loud reporting of unreadable hubs ---
out=$(cargo test -p termlink fleet_dedups 2>&1 || true); grep -q "fleet_dedups_profiles_that_resolve_to_the_same_hub ... ok" <<< "$out"
out=$(cargo test -p termlink fleet_keeps 2>&1 || true); grep -q "fleet_keeps_profiles_whose_fingerprint_probe_failed ... ok" <<< "$out"
out=$(cargo test -p termlink scope_note_reports 2>&1 || true); grep -q "scope_note_reports_unreadable_hubs_loudly ... ok" <<< "$out"

# --- AC5: the scope note is derived, and a fleet read never claims single-hub scope ---
out=$(cargo test -p termlink fleet_scope_note 2>&1 || true); grep -q "fleet_scope_note_does_not_claim_single_hub_scope ... ok" <<< "$out"
out=$(cargo test -p termlink inbox_scope 2>&1 || true); grep -q "inbox_scope_note_names_both_blind_spots_and_a_remedy ... ok" <<< "$out"

# --- AC6: machine contract — stdout stays a bare ARRAY, scope on stderr only ---
# substrate-worker-pickup.sh:290 and the T-2153 orchestrator recipe both parse this.
grep -qF 'eprintln!("{}", inbox_scope_note(&scope));' crates/termlink-cli/src/commands/channel.rs
out=$(./target/debug/termlink channel inbox --json 2>/dev/null || true); python3 -c "import json,sys; d=json.load(sys.stdin); assert isinstance(d, list), f'stdout must be a bare array, got {type(d).__name__}'" <<< "$out"

# --- AC7: mutation proof — every mutation must be DETECTED, none may stay green ---
out=$(bash tests/mutate-2783.sh 2>&1 || true); grep -q "MUTATION NOT DETECTED" <<< "$out" && exit 1 || true
out=$(bash tests/mutate-2783.sh 2>&1 || true); test "$(grep -c 'detected' <<< "$out")" -eq 4

# --- AC8: whole tree green ---
out=$(cargo test -p termlink -p termlink-mcp 2>&1 || true); grep -q "0 failed" <<< "$out"
out=$(cargo test -p termlink -p termlink-mcp 2>&1 || true); grep -q "FAILED" <<< "$out" && exit 1 || true
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -qE "PASS — [0-9]+/[0-9]+ members clean" <<< "$out"

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

### 2026-08-17T10:56:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2783-agent-inbox---fleet-make-the-verb-comple.md
- **Context:** Initial task creation

### 2026-08-17T12:00:51Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
