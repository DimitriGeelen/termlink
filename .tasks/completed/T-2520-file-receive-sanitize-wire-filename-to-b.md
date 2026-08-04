---
id: T-2520
name: "file receive: sanitize wire filename to basename (path-traversal / arbitrary-write fix)"
description: >
  Receiver-side FileInit.filename / ArtifactManifest.filename is deserialized from the wire and passed unsanitized into out_path.join() -> fs::write at 4 sites (file.rs:507,537,858,975). An absolute or ..-bearing filename escapes the -o output dir (arbitrary file write). The protocol type already documents the invariant (events.rs:499 'basename only, no path') but the receiver never enforced it. Reduce to a trusted basename before join.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/file.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T09:39:27Z
last_update: 2026-08-04T09:57:06Z
date_finished: 2026-08-04T09:57:06Z
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

# T-2520: file receive: sanitize wire filename to basename (path-traversal / arbitrary-write fix)

## Context

Campaign firing #39 (T-2468 subtract-and-deepen review, file-send/receive framing lens).
The receiver of `termlink file receive <target> -o <dir>` (and the `termlink_file_receive`
MCP tool) trusts the wire-supplied filename. `FileInit.filename` (legacy chunk/inline path)
and `ArtifactManifest.filename` (current artifact path) are deserialized from the peer's
payload and passed unmodified to `out_path.join(name)` → `std::fs::write`. Rust `Path::join`
treats an absolute component as replacing the base (`/out`.join(`/etc/cron.d/x`) → `/etc/cron.d/x`)
and follows `..` components, so an adversarial/compromised sender chooses where the file lands
— arbitrary file write outside the `-o` dir (overwrite cron/authorized_keys/rc → priv-esc/RCE).
The protocol type already documents the invariant it never enforced: `events.rs:499 /// Original
filename (basename only, no path).` This fix makes the receiver honor that documented contract.
SHA verification is NOT a mitigation — filename and content are independent fields; a
content-verified transfer still writes to an attacker-chosen path.

## Acceptance Criteria

### Agent
- [x] A `sanitize_recv_filename` helper reduces a wire filename to its trusted final basename (via `Path::file_name`), returning `None` for `""`, `.`, `..`, or path-only inputs.
- [x] All four receive-side join sites (file.rs:507, 537, 858, 975) route the wire filename through the helper before `out_path.join`; a rejected/path-bearing name falls back to the existing synthesized default, never escapes `out_path`.
- [x] A regression test `sanitize_recv_filename_confines_to_basename` asserts traversal (`../escape`), absolute (`/etc/x`), and nested (`a/b/c.txt`) inputs all reduce to a bare basename (or `None`) and never yield a parent/absolute path. Proven load-bearing (FAILS without the fix).
- [x] `cargo test -p termlink --bins sanitize_recv_filename` passes; `cargo build --release -p termlink` succeeds (exit 0).

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
cargo test -p termlink --bins sanitize_recv_filename > /tmp/.t2520-test.out 2>&1 && grep -q "test result: ok" /tmp/.t2520-test.out
grep -q "fn sanitize_recv_filename" crates/termlink-cli/src/commands/file.rs

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

## RCA

**Symptom:** A peer sending a file to a `termlink file receive -o <dir>` receiver can write the
file anywhere the receiving process has permission, not just under `<dir>`, by putting an absolute
path or `..` components in the wire filename (`FileInit.filename` / `ArtifactManifest.filename`).

**Root cause:** The receive side passed the deserialized wire filename directly to
`out_path.join(name)`. `Path::join` replaces the base on an absolute component and follows `..`,
so the sender — not the receiver — controls the destination. No basename reduction existed on the
receive side (the only `file_name()` call was on the *send* side, line 200, which is cosmetic and
does not constrain what an adversarial sender puts on the wire).

**Why structurally allowed:** The protocol type documented the invariant (`events.rs:499` "basename
only, no path") but nothing *enforced* it — a documented-but-unchecked contract. Serde populates the
`String` field verbatim; the field name reads innocuous ("filename"); and the send-side sanitization
created a false sense that names were basename-only by the time they reached the receiver. The
file-transfer primitive is also flagged for retirement (T-1166), which likely lowered scrutiny.

**Prevention:** `sanitize_recv_filename` centralizes the basename reduction so the four write sites
cannot diverge, plus a unit test (`sanitize_recv_filename_confines_to_basename`) that pins traversal/
absolute/nested inputs to a confined basename. Any future receive site that forgets to sanitize is a
visible omission against this now-existing helper.

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

### 2026-08-04T09:39:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2520-file-receive-sanitize-wire-filename-to-b.md
- **Context:** Initial task creation

### 2026-08-04T09:40:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-04T09:57:05Z — status-update [task-update-agent]
- **Change:** owner: human → agent

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1644e4b2
- **Timestamp:** 2026-08-04T09:57:37Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — A regression test `sanitize_recv_filename_confines_to_basename` asserts traversal (`../escape`), absolute (`/etc/x`), and nested (`a/b/c.txt`) inputs all reduce to a bare basename (or `None`) and neve
  - **AC-verify-mismatch** (narrow, heuristic) — `path=a/b/c.txt in: A regression test `sanitize_recv_filename_confines_to_basename` asserts traversal (`../escape`), absolute (`/etc/x`), and nested (`a/b/c.txt`) inputs `

### 2026-08-04T09:57:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
