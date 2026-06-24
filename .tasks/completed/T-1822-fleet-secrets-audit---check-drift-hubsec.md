---
id: T-1822
name: "fleet secrets-audit --check-drift <hub.secret-path> — compare IP-keyed cache vs authoritative hub.secret (G-011 item 1, T-1820 follow-up #3)"
description: >
  fleet secrets-audit --check-drift <hub.secret-path> — compare IP-keyed cache vs authoritative hub.secret (G-011 item 1, T-1820 follow-up #3)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-28T07:52:04Z
last_update: 2026-05-28T08:05:23Z
date_finished: 2026-05-28T08:05:23Z
---

# T-1822: fleet secrets-audit --check-drift <hub.secret-path> — compare IP-keyed cache vs authoritative hub.secret (G-011 item 1, T-1820 follow-up #3)

## Context

T-1820 shipped `fleet secrets-audit` covering perms / format / orphan
detection (G-011 item 4). T-1821 shipped MCP parity. This task closes
**G-011 item 1**: drift detection between an IP-keyed cache file
(`~/.termlink/secrets/<IP>.hex`) and the authoritative `<runtime_dir>/hub.secret`.

The 2026-04-20 incident (recorded in PL-041 and CLAUDE.md R3) was exactly
this: the giving-end's IP-keyed cache had been stale for ~1 day after a
hub restart; the giving side appeared clean while the peer saw auth-mismatch.
The existing audit catches *perms* hygiene but cannot tell you whether
a cache file's *content* still matches the authoritative source.

Scope: add `--check-drift <authoritative-path>` flag. When set, the audit
reads the authoritative file's hex content and, for each `.hex` file
scanned, classifies as one of `ok-mirror` (content matches), `warn-drift`
(content differs), or leaves status unchanged if neither file is a valid
64-char hex payload (warn-format already flags those).

## Acceptance Criteria

### Agent
- [x] CLI flag `--check-drift <PATH>` added to `FleetAction::SecretsAudit` in cli.rs
- [x] `classify_secret_file` extended with optional `authoritative_hex: Option<&str>` parameter
- [x] New status `warn-drift` added — fires when content differs AND both files are valid 64-char hex; otherwise no drift verdict (perms/format issues take priority)
- [x] `scan_secrets_dir` reads authoritative file once (if --check-drift set), passes hex to classifier
- [x] Authoritative file itself validated (format + perms warning if mode > 0o600); on read failure, the audit reports the error and skips drift-check (perms/format/orphan still run)
- [x] JSON output includes new top-level field `authoritative` when --check-drift set: `{path, mode, size, status, reasons}`
- [x] Severity priority documented and tested: warn-perms > warn-format > warn-drift > info-orphan > ok-mirror > ok
- [x] Exit 1 when any warn-drift present (mirror semantics: warn-perms || warn-format || warn-drift)
- [x] 5 new classifier unit tests: ok-mirror match, warn-drift differs, drift skipped on format-bad, perms outranks drift, case-insensitive comparison
- [x] `cargo check -p termlink` passes (package name is `termlink`, not `termlink-cli`)
- [x] `cargo test -p termlink secrets_audit` passes (6 existing + 5 new = 11/11)
- [x] Live-run on .107 against `/var/lib/termlink/hub.secret` — authoritative ok-mirror, 3 peer-cache files correctly flagged warn-drift (semantic note: broad-mode comparison; see Recommendation)

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
cd /opt/termlink && cargo check -p termlink 2>&1 | tail -5
cd /opt/termlink && cargo test -p termlink secrets_audit 2>&1 | tail -15

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

## Recommendation

**GO** — ship as-is.

Closes G-011 item 1 (drift detection between IP-keyed cache and authoritative
`<runtime_dir>/hub.secret`). The classifier extension is backward-compatible
(default `None` produces the existing T-1820 verdicts unchanged) so no
risk to currently-shipped surveillance.

**Live result on .107** (authoritative = `/var/lib/termlink/hub.secret`):

```
# authoritative: ok-mirror 0o600 /var/lib/termlink/hub.secret
warn-drift   0o600 /root/.termlink/secrets/laptop-141.hex      [content differs]
info-orphan  0o600 /root/.termlink/secrets/proxmox4.hex        [not referenced]
warn-drift   0o600 /root/.termlink/secrets/ring20-dashboard.hex [content differs]
warn-drift   0o600 /root/.termlink/secrets/ring20-management.hex [content differs]
```

**Semantic note (broad-mode behavior).** When `--check-drift` is given, the
audit compares EVERY cache in the dir against the named authoritative. On
.107 this produces 3 warn-drift hits for peer-hub caches — those caches
SHOULD differ from .107's own hub.secret because they point at remote
hubs. The verdict is correct given the contract ("is this cache the
mirror of this authoritative?"); the operator-burden is choosing the
right scope. For the canonical PL-041 use case (single-hub host, one
IP-keyed cache that's expected to match this host's hub.secret) broad-mode
produces the right verdict on its own.

**Follow-ups filed as needed:**
1. `--target-cache <PATH>` scoping flag — file as T-1823 if broad-mode false-positives become operator-friction signal
2. MCP parity (`check_drift: Option<String>` param in T-1821) — small extension, file as T-1824
3. `--auto-detect-self-hub` flag — derive authoritative path from `TERMLINK_RUNTIME_DIR` env var + filesystem probe (deferred — needs design)

## Updates

### 2026-05-28T07:52:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1822-fleet-secrets-audit---check-drift-hubsec.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a02aba7f
- **Timestamp:** 2026-05-28T08:06:28Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#12 (Agent)** — Live-run on .107 against `/var/lib/termlink/hub.secret` — authoritative ok-mirror, 3 peer-cache files correctly flagged warn-drift (semantic note: broad-mode comparison; see Recommendation)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=var/lib/termlink/hub.secret in: Live-run on .107 against `/var/lib/termlink/hub.secret` — authoritative ok-mirror, 3 peer-cache files correctly flagged warn-drift (semantic note: bro`

### 2026-05-28T08:05:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
