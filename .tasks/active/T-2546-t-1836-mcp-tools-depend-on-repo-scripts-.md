---
id: T-2546
name: "T-1836 MCP tools depend on repo scripts absent from consumer installs (portability)"
description: >
  Portability lens (T-2468, Directive #4). Verified in code: ~18 T-1836 MCP tools (resolve_t1836_script, tools.rs:29098) shell out via Command::new(bash) (tools.rs:29121) to scripts under ${TERMLINK_SCRIPTS_DIR:-/opt/termlink/scripts}. The default is this repo's dev-host checkout path AND the scripts do not ship with an installed (brew/cargo) binary, so every non-repo consumer sees all 18 tools fail (loudly, with a set-TERMLINK_SCRIPTS_DIR hint) until they clone the repo and point the env var at it. Crux question (human product-scope): are these operator-host canary/heartbeat tools in-scope for consumer installs at all? If YES -> fix by embedding scripts (include_str! -> tempdir) or in-process rewrite. If NO (operator-host-only by design, per PL-185 shell-out decision) -> no code change; document the scope. Owner human: product-scope + distribution-architecture decision.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-08T18:30:55Z
last_update: 2026-08-08T18:33:35Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2546: T-1836 MCP tools depend on repo scripts absent from consumer installs (portability)

## Problem Statement

TermLink's charter Directive #4 (Portability) forbids provider/language/environment
lock-in. Verified in code: the T-1836 MCP tool family (~18 tools — the
listener-heartbeat / fleet-canary trio and siblings, added "shell-out per PL-185")
resolves its scripts via `resolve_t1836_script` (`crates/termlink-mcp/src/tools.rs:29098`),
whose default directory is **`/opt/termlink/scripts`** — *this repository's dev-host
checkout path* — and then runs them with `Command::new("bash")`
(`tools.rs:29121`). Two coupled portability facts:

1. The default `/opt/termlink/scripts` is a specific build-host absolute path baked
   into a distributed binary.
2. More fundamentally, **the scripts are not distributed with the installed binary
   at all** — a Homebrew/cargo consumer has no `scripts/` dir on disk, so *every one
   of these 18 MCP tools fails* (loudly, with a `set TERMLINK_SCRIPTS_DIR` hint)
   until the consumer clones the repo and points the env var at it.

The failure is loud (not silent) — `resolve_t1836_script` returns a `json_err` with
a remediation hint — so this is a *degraded-capability* portability gap, not a
crash. **Why now:** the T-2468 purpose review is sweeping every Constitutional
Directive; Portability was the last un-run lens and this is its one genuine
consumer-affecting finding.

## Assumptions

- A-1: `resolve_t1836_script` defaults to `/opt/termlink/scripts` and the tools
  shell out via `bash`. — VERIFIED (`tools.rs:29098-29121`).
- A-2: The `scripts/` directory is not copied to any standard location by an
  install (no Homebrew formula / release step stages it). — VERIFIED: the release
  workflow copies only the binary (`.github/workflows/release.yml:53`) and the brew
  formula only `bin.install`s it (`homebrew/Formula/termlink.rb:39`). `scripts/`
  ships nowhere; a consumer install has zero scripts on disk.
- A-3: These ~18 tools were *intentionally* shell-outs to repo scripts (PL-185),
  i.e. designed as operator-host tooling run from a checkout — not as
  general-consumer MCP surface. — PLAUSIBLE from the PL-185 note; the human owns
  whether that intent still holds.
- A-4: The core protocol/hub/session path has NO comparable cross-OS portability
  defect. — VERIFIED by the hunt (runtime_dir resolution chain, /proc soft-degrade,
  setsid fallback all properly guarded).

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Are the T-1836 canary/heartbeat MCP tools in-scope for consumer
  (non-repo) installs at all, or are they operator-host-only by design?**
  confidence: 2
  disposition: deferred
  rationale: The PL-185 "shell-out to repo scripts" design implies operator-host
  intent, but that is a product-scope call the human owns. This is the CRUX — the
  answer decides whether ANY code change is warranted. If operator-host-only →
  documentation, not code. If consumer-scoped → IW-2.

- **IW-2: If consumer-scoped, which distribution strategy — embed scripts in the
  binary (`include_str!` → tempdir at call time), stage them via the installer
  (brew formula + release step copies to a resolved prefix), or rewrite the ~18
  tools in-process (eliminate the bash shell-out entirely)?**
  confidence: 1
  disposition: deferred
  rationale: Each has distinct cost/reversibility (embed = self-contained binary,
  moderate; install = cross-repo brew/release change; in-process = large refactor,
  retires the `bash` dependency too). Part of a GO build scope, not this inception.

- **IW-3: Independent of scope, should the default resolution at least try a
  portable chain (exe-relative → XDG data dir) before the `/opt/termlink/scripts`
  dev-host fallback, mirroring `runtime_dir()`'s chain?**
  confidence: 2
  disposition: deferred
  rationale: A small additive improvement (keep env override + loud-fail) that
  helps the "repo present but binary run from elsewhere" case even under the
  operator-host answer. Low-risk; candidate for a bounded GO even if IW-1 says
  operator-host-only.

## Exploration Plan

1. Verify A-1 (default path + bash shell-out). — DONE (`tools.rs:29098-29121`).
2. Verify A-2: read the Homebrew formula + `.github/workflows/release.yml` to
   confirm `scripts/` is NOT staged into any install prefix. (GO build prereq.)
3. Human resolves IW-1 (the crux product-scope call).
4. If consumer-scoped → evaluate IW-2 strategies + prototype the cheapest correct
   one; else → document the operator-host scope + close.

## Technical Constraints

- **Distribution:** the release flow (CLAUDE.md CI/Release) ships the `termlink`
  binary + checksums via GitHub Releases → Homebrew; there is no script-staging
  step today. Any "install the scripts" fix is cross-repo (brew formula lives with
  the release tooling) and touches the GitHub-mirror release path.
- **PL-185:** the shell-out-to-repo-scripts design was deliberate; a rewrite-in-
  process fix reverses that decision for ~18 tools and must preserve their exact
  JSON envelopes + timeout/kill-on-drop semantics.
- **Unix scope:** the crate is unix-by-design (unix sockets, `libc::getuid`); the
  `bash` (vs `sh`) dependency is a within-unix nuance, not a Windows blocker (the
  binary already precludes Windows session-control).

## Scope Fence

**IN scope (this inception):** confirm the finding in code (done), frame the
consumer-vs-operator-host scope question, and lay out the three distribution
strategies with cost/reversibility so the human can decide.
**OUT of scope:** the actual fix (a GO build task — embed/install/rewrite or a
docs-only scope note), the brew-formula/release change (cross-repo), and any
rewrite of the ~18 tools' internals.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO (fix in code) if:**
- The human judges the T-1836 tools in-scope for consumer installs (IW-1), AND
- A bounded strategy is chosen (IW-2): embed-scripts is the smallest self-contained
  fix; OR IW-3 alone (portable resolution chain) is approved as a low-risk additive
  improvement even under the operator-host answer.

**NO-GO (docs-only) if:**
- The human judges these operator-host-only tooling by design (PL-185 intent holds)
  → no code change; instead document the operator-host scope (these MCP tools
  require a repo checkout + `TERMLINK_SCRIPTS_DIR`) so the loud-fail hint is
  expected, not a bug. Close as respected-boundary.

**INTERMEDIATE (IW-3 only) if:**
- Scope stays operator-host but the dev-host-path default is worth softening → land
  only the portable resolution chain (exe-relative → XDG → `/opt/termlink/scripts`),
  keeping env override + loud-fail. Small bounded build; defers the bigger IW-2.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:** Verified tools.rs:29098-29121: default script dir is the repo path /opt/termlink/scripts and scripts are not distributed with the binary, so 18 MCP tools are dead on any non-repo install (loud-fail, not silent). Whether that is a defect depends on a human product-scope call the agent cannot make: are the T-1836 canary/heartbeat MCP tools intended for consumer installs, or operator-host-only (the PL-185 shell-out design implies the latter)? DEFER pending that scope decision — if operator-host-only, this is documentation not code.

**Related low-severity observation (NOT a co-fix — logged here per one-bug-one-task
so it isn't lost):** `crates/termlink-cli/src/commands/push.rs:9` hardcodes
`INBOX_DIR = "/tmp/termlink-inbox"` (written on the *remote* host at `:73/:90`) — a
new `/tmp` path of the PL-021 volatile-tmp class and a remote-host-layout
assumption. LOW value because `remote push` is **deprecated** (`push.rs:23`
`print_deprecation_warning("remote push", "channel post")`) and the payload is
transient. Left as an observation; if `remote push` is ever un-deprecated, file it
as its own task then.

**Portability lens net result (T-2468):** the core protocol/hub/session path is
CLEAN — `runtime_dir()` has a proper resolution chain (env → XDG → TMPDIR/macOS →
`/tmp/termlink-$UID`), `/proc/<pid>` reads soft-degrade to `None`, and `setsid`
spawn has an `sh` fallback. The only genuine consumer-affecting portability defect
is this T-1836 script-shell-out layer.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
