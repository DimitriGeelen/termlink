---
id: T-2607
name: "Identity/trust plane silently relocates to volatile /tmp when HOME unset (portability + no-silent-failures)"
description: >
  Verb-portability hunt F1 (HIGH): tofu.rs known_hubs_path + offline_queue + ack_retry fall back to /tmp when HOME unset; tofu ignores TERMLINK_IDENTITY_DIR

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T11:14:16Z
last_update: 2026-08-11T11:15:40Z
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

# T-2607: Identity/trust plane silently relocates to volatile /tmp when HOME unset (portability + no-silent-failures)

## Context

Found by the T-2468 **Portability** (Constitutional Directive #4 — "no
provider/language/environment lock-in") adversarial hunt — finding F1 (HIGH),
verified in code.

The client **identity/trust plane** silently relocates to a volatile,
world-writable directory when `HOME` is unset:

```rust
// crates/termlink-session/src/tofu.rs:19  — the TRUST STORE (known_hubs / TOFU pins)
let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
PathBuf::from(home).join(".termlink").join("known_hubs")
```

Same `HOME → "/tmp"` fallback pattern in the sibling identity artifacts:
- `crates/termlink-session/src/offline_queue.rs:117` (durable outbound queue)
- `crates/termlink-session/src/ack_retry.rs:297` (await-ack retry DB)
- MCP mirrors: `crates/termlink-mcp/src/tools.rs:7322, 7372, 10601`

**Two coupled defects (one root cause — the identity plane has no coherent,
fail-loud dir-resolution policy):**

1. **Silent relocation to volatile world-writable `/tmp` (HIGH).** When `HOME`
   is unset — a systemd unit without `Environment=HOME` / `User=`, an `env -i`
   invocation, or a minimal container — the **TOFU trust store, offline queue,
   and ack-retry DB all silently move to `/tmp/.termlink/`**. Consequences on
   the trust plane specifically: (a) pins are lost on every reboot on a volatile
   `/tmp` host → recurring TOFU-violation flaps (exactly the PL-021 / G-058
   volatile-`/tmp` class CLAUDE.md warns about, but on the *client trust* plane
   instead of the *hub* plane); (b) `/tmp/.termlink/known_hubs` is in a shared,
   world-readable/writable location → cross-user pin tamper / collision. No error
   is surfaced — violates Reliability #2 ("no silent failures") AND Portability #4.

2. **`tofu.rs` ignores `TERMLINK_IDENTITY_DIR` (MEDIUM, subset).** `offline_queue.rs:114`
   already honors a `TERMLINK_IDENTITY_DIR` override *before* the HOME/`/tmp`
   fallback (so the queue, identity key, etc. can be co-located and test-isolated).
   `tofu.rs::known_hubs_path()` and `ack_retry.rs` do **not** consult it, so the
   trust store cannot be pinned to the same dir as the rest of the identity plane
   — an inconsistency that makes fix #1 harder to configure cleanly.

**Why file (not build autonomously):** this is a **security/trust-plane
contract change** on a **shared helper with many callers**. A hard-error on
`HOME`-unset would break `env -i`/minimal-container invocations that currently
"succeed" by writing to `/tmp`; making `known_hubs_path()` return a `Result`
ripples to every caller. Choosing *what* HOME-unset should do (see Decisions) is
a deliberate design call, not a mechanical patch. Sibling of the PL-021 hub-side
volatile-/tmp work; coordinate the dir-resolution policy so hub and client planes
tell one consistent story.

## Acceptance Criteria

### Agent
- [ ] A decision is recorded (see Decisions) on the `HOME`-unset behavior for the
      identity/trust plane, AND on unifying `TERMLINK_IDENTITY_DIR` resolution
      across `tofu.rs` / `offline_queue.rs` / `ack_retry.rs` (+ the MCP mirrors).
- [ ] The trust store (`known_hubs_path`) and the ack-retry DB honor
      `TERMLINK_IDENTITY_DIR` before any HOME-based path — matching the existing
      `offline_queue::default_queue_path` convention, so the whole identity plane
      resolves to one dir.
- [ ] The silent `HOME → "/tmp"` fallback is replaced per the decision: either a
      loud refusal (the trust store is NOT written to a world-writable volatile
      dir without an explicit opt-in), or an XDG-style `$XDG_STATE_HOME`/documented
      fallback — NOT a bare `unwrap_or("/tmp")`. Whatever is chosen is identical
      across all identity-plane path helpers (no per-file divergence).
- [ ] A load-bearing test proves it: with `HOME` unset (and `TERMLINK_IDENTITY_DIR`
      unset) the resolver yields the chosen outcome (error / XDG path), NOT
      `/tmp/.termlink/...`. Prove load-bearing by temp-reverting to the
      `unwrap_or("/tmp")` form and confirming the test FAILS.
- [ ] `cargo test -p termlink-session` passes (and `cargo check -p termlink-mcp`
      if the MCP mirrors are touched).

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

### OPEN — what should the identity/trust plane do when `HOME` is unset?

Three defensible options; pick one deliberately (this is why the task is filed,
not auto-built — it is a security-plane contract change).

- **Option A — XDG fallback then loud error (recommended).** Resolution order:
  `TERMLINK_IDENTITY_DIR` → `$XDG_STATE_HOME/termlink` → `$HOME/.termlink` →
  **hard error** ("cannot resolve a private identity dir: set HOME or
  TERMLINK_IDENTITY_DIR"). Never `/tmp`. Honors both directives: no silent
  relocation (Reliability), no environment coupling to a specific layout
  (Portability), and no world-writable trust store. Cost: `env -i` invocations
  that today "work" via `/tmp` now fail loudly — but that prior success was the
  bug (they were writing trust pins to a shared volatile dir).
- **Option B — same as A but a documented `/tmp/.termlink-$UID` last resort
  instead of erroring.** Keeps backward-compat for HOME-less invocations but
  moves the fallback to a `$UID`-namespaced (non-world-writable) path and logs a
  loud `tracing::warn!` that pins will not survive reboot. Weaker — still writes
  trust material to volatile storage, just less dangerously.
- **Option C — env-override only (`TERMLINK_IDENTITY_DIR` everywhere), leave the
  `HOME`-unset `/tmp` fallback as-is.** Smallest change; fixes only defect #2
  (the inconsistency). Does NOT fix the HIGH silent-relocation defect #1.
  Rejected as insufficient unless the owner decides #1 is acceptable risk.

Recommendation leans **Option A** (strongest on both directives; the trust plane
should never silently land in `/tmp`). Owner to confirm A/B/C before coding.

### Scope / one-root-cause note
Defects #1 (silent `/tmp`) and #2 (`tofu` ignores `TERMLINK_IDENTITY_DIR`) share
ONE root cause — the identity plane has no single fail-loud dir-resolution
policy — so they are fixed together here (a shared `resolve_identity_dir()`
helper is the natural shape). The MEDIUM `/proc` macOS finding from the same hunt
is a DIFFERENT root cause (no macOS `sysctl` fallback for the whoami PID-ancestor
walk) and is filed separately.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-11T11:14:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2607-identitytrust-plane-silently-relocates-t.md
- **Context:** Initial task creation

### 2026-08-11T11:15:40Z — status-update [task-update-agent]
- **Change:** horizon: now → next
