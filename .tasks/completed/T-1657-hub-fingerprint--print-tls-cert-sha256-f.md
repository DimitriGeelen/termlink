---
id: T-1657
name: "hub fingerprint — print TLS cert sha256 fingerprint for peer verification (PL-021 ergonomics)"
description: >
  hub fingerprint — print TLS cert sha256 fingerprint for peer verification (PL-021 ergonomics)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T14:25:44Z
last_update: 2026-05-17T14:36:11Z
date_finished: 2026-05-17T14:36:11Z
---

# T-1657: hub fingerprint — print TLS cert sha256 fingerprint for peer verification (PL-021 ergonomics)

## Context

When a peer first connects to a hub, TOFU records the cert fingerprint in `~/.termlink/known_hubs`. To verify the pin is correct (especially after a hub rotation per PL-021), the operator on the giving side needs the canonical sha256 fingerprint of the live `<runtime_dir>/hub.cert.pem`. Today there is no built-in command — operators reach for `openssl x509 -in /var/lib/termlink/hub.cert.pem -noout -fingerprint -sha256` and reformat the output.

A `termlink hub fingerprint` command reads the live PEM, computes the DER sha256, and prints the canonical `sha256:<64-hex>` form. Same shape as the existing `termlink_session::tofu::cert_fingerprint()` so the output is directly comparable against `KnownHubStore` entries on peers.

Mirrors T-1656's "always read live, never cache" design — peer-verification handoffs use the live cert path, not a stale cache.

## Acceptance Criteria

### Agent
- [x] `HubAction::Fingerprint` variant added to `cli.rs` enum + dispatched in main.rs
- [x] `cmd_hub_fingerprint(json)` reads `termlink_hub::tls::hub_cert_path()`, parses the first BEGIN CERTIFICATE PEM block, computes DER sha256 via `termlink_session::tofu::cert_fingerprint()`
- [x] Stdout default: prints `sha256:<64-hex>` (live: `sha256:d1bd50f5cb03c4fd11689b77c4d9d6a3d6f8f83ff947a23c2dd586c43abb359f`)
- [x] `--json` flag: `{"path":"<live-cert>","fingerprint":"sha256:..."}` — live JSON matches expected shape
- [x] Error path: missing cert → exit 1 with "no hub.cert.pem at /tmp/no-hub-xyz/hub.cert.pem — is the hub running?"
- [x] Error path: PEM has no BEGIN CERTIFICATE block → covered by `fingerprint_no_certificate_block_errors` unit test (real-PEM with PRIVATE KEY block only)
- [x] Unit test: `fingerprint_matches_tofu_format` — hand-built PEM + DER round-trip; verified `sha256:` prefix and 64-hex length
- [x] Unit test: `fingerprint_missing_cert_errors` — no cert under staged TERMLINK_RUNTIME_DIR; error msg contains both substrings
- [x] `cargo test -p termlink --bin termlink -- fingerprint` passes (3/3 plus 1 unrelated)
- [x] `cargo check --workspace` passes
- [x] Live-smoke against local hub: output `sha256:d1bd50f5...` exactly matches `openssl x509 -noout -fingerprint -sha256` canonical form
- [x] Commit with T-1657 prefix (3c99d7fc)

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
cargo check --workspace
cargo test -p termlink --bin termlink -- fingerprint
grep -q "    Fingerprint {" crates/termlink-cli/src/cli.rs

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

### 2026-05-17T14:25:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1657-hub-fingerprint--print-tls-cert-sha256-f.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-63170ffe
- **Timestamp:** 2026-05-17T14:36:11Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#5 (Agent)** — Error path: missing cert → exit 1 with "no hub.cert.pem at /tmp/no-hub-xyz/hub.cert.pem — is the hub running?"
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tmp/no-hub-xyz/hub.cert in: Error path: missing cert → exit 1 with "no hub.cert.pem at /tmp/no-hub-xyz/hub.cert.pem — is the hub running?"`

### 2026-05-17T14:36:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
