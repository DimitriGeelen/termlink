# T-3258 — guard-layer severity classification: DRAFT for operator approval

**Status: PROPOSAL ONLY. Nothing in the tree is classified advisory** (`grep -rlE '^#\s*guard-layer:\s*source\s+advisory' scripts tests` → 0 files). The mechanism shipped in T-3258 is inert until the operator approves entries here and a follow-up task edits the markers.

## What a class changes
- `# guard-layer: source advisory  # <reason>` → ADVISORY. No word → BLOCKING (default).
- Only `run-guard-layer.sh --gate release` (the `test` job in `release.yml`) distinguishes them: advisory FAIL/ERROR is printed in its own section with its reason and counted in the summary, but does not set the exit code.
- Push CI (`doc-lint.yml`) runs the default gate, where every red still counts. **Demoting a member does not hide it anywhere.** It only stops that member from holding a release.
- `cargo test --workspace` and fixture suites (`tests/*fixtures*.sh`, which prove the guards can fire) are proposed BLOCKING as a class; neither is listed row by row.

## Rule used for the proposal
1. **BLOCKING** if a red means a defect in what is being released: shipped code, shipped docs, the install path. Also BLOCKING if the member protects the *evidence* that shipped work was verified (the P-011 vacuity family), or the detection/demotion machinery itself.
2. **ADVISORY** if a red describes process, ledger or host state: a human backlog, time-driven hygiene, cross-branch or live-bus state. Such a red can appear with no code change and cannot be fixed by the release.
3. **DECIDE** where the two pull against each other; the tension is stated in the row.

Counts: **41 BLOCKING · 12 ADVISORY · 6 DECIDE** across 59 static members.

## Risk cases, read these first
- **`check-framework-tracking-drift`**: looks like bookkeeping, but it catches clean-clone breakage (`fw` fails, and pre-commit guards fail open in worktrees; T-2806/T-2807). Proposed BLOCKING.
- **`voi-prompt`**: its header says it "never blocks — it only reports" (`scripts/voi-prompt.sh:287,314`), yet its `--check` exits 1 inside the layer, which is why Doc Lint is red today. Either it is ADVISORY, or its exit contract changes. The current state contradicts itself. The underlying question (T-3200) is the operator's to answer; this report does not answer it.
- **`check-go-propagation`**: proposed ADVISORY because, after R8 route A, it is red only on T-3012, a human decision. The cost of ADVISORY is that the GO-leak (C-35) stops holding releases. That is acceptable only because push CI still shows it red.
- **`check-arc-claim-drift`**: "a closed arc's capability is no longer proven" sounds BLOCKING. But its provers need a live binary or hub and SKIP in CI (T-3238/T-3239), so the class barely matters in CI today. Decide on principle.
- **`check-vendor-divergence`**: no release impact, but the release gate may be the only forcing function before a re-vendor silently deletes a local fix.

## Per-member proposal
| member | what it guards | proposed | one-line reason |
|---|---|---|---|
| `check-absence-assertion.sh` | check-absence-assertion.sh — a `## Verification` leg that asserts something is ABSENT | **BLOCKING** | absence legs that pass on a missing file — vacuous gate |
| `check-alloc-sink-clamps.sh` | check-alloc-sink-clamps.sh (T-2527, G-019 prevention for the T-2523/T-2526 class) | **BLOCKING** | guards daemon OOM class in shipped code |
| `check-arc-claim-drift.sh` | T-3066 — a closed arc asserts a capability. Is anything still checking it? | **DECIDE** | a closed arc's capability no longer proven — sounds BLOCKING, but provers need a binary/hub and SKIP in CI (T-3238/3239) |
| `check-arc-slice-drift.sh` | T-3083 — an arc's SLICE REGISTER goes stale and nothing detects it. | **ADVISORY** | arc register bookkeeping |
| `check-audit-warning-acknowledgement.sh` | T-3167 — the missing half of arc-008's success condition. | **DECIDE** | arc-008's success condition — process, but its reds were real defects in R7 (T-3240) |
| `check-budget-ladder-drift.sh` | check-budget-ladder-drift.sh — does the DOCUMENTED context-budget ladder match the LIVE ga | **DECIDE** | can go red from a re-vendor with nothing fixable locally (vendored text, G-062) — candidate ADVISORY |
| `check-busy-spin.sh` | check-busy-spin.sh (T-2672, G-019 prevention for the T-2658/T-2636/T-2640/T-2670/T-2671 | **BLOCKING** | guards CPU-burn on dead hub in shipped CLI/MCP |
| `check-canary-log-hygiene.sh` | check-canary-log-hygiene.sh (T-2685, G-019 prevention for the T-2683 F2 class) | **BLOCKING** | crontab stream split — detection layer integrity |
| `check-canary-log-isolation.sh` | check-canary-log-isolation.sh (T-2761, G-019 prevention for the T-2402-sibling class) | **BLOCKING** | detection layer integrity |
| `check-charter-sentence-drift.sh` | check-charter-sentence-drift.sh (T-2484, G-019 charter-fork prevention) | **BLOCKING** | three shipped copies of the charter sentence |
| `check-decisions-register.sh` | T-2969 — local detector for .context/project/decisions.yaml corruption. | **BLOCKING** | corruption detector; rarely red, cheap to fix |
| `check-drain-sink-caps.sh` | check-drain-sink-caps.sh (T-2531, G-019 prevention for the T-2518/2524/2525/2529 class) | **BLOCKING** | guards unbounded peer-driven drains in shipped daemons |
| `check-env-var-docs.sh` | check-env-var-docs.sh (T-2220) — Level-C prevention for env-var-name doc drift. | **BLOCKING** | documented env var names must exist in code |
| `check-episodic-parse.sh` | T-2805 — episodic-store readability check. | **DECIDE** | 29 legacy/corrupt files existed at T-2805; if any remain red, ADVISORY until migrated |
| `check-error-code-docs.sh` | scripts/check-error-code-docs.sh | **BLOCKING** | documented error codes must match control.rs |
| `check-error-code-emission.sh` | check-error-code-emission.sh (T-2699, T-2698 G1) | **BLOCKING** | refusal taxonomy vs code: wire-protocol claims |
| `check-error-swallowing-predicate.sh` | check-error-swallowing-predicate.sh (T-2792, G-019 prevention for the T-2791 class) | **BLOCKING** | guards swallowed errors in shipped code |
| `check-fabric-card-parse.sh` | check-fabric-card-parse.sh — every .fabric/components/*.yaml must parse. | **BLOCKING** | corruption detector |
| `check-fleet-recipient-agreement.sh` | do the two fleet-wide broadcast paths resolve the same recipients? | **BLOCKING** | two broadcast paths disagreeing is a shipped-behaviour bug |
| `check-framework-tracking-drift.sh` | check-framework-tracking-drift.sh — T-2814. | **BLOCKING** | RISK CASE — looks like bookkeeping, but DANGLING/UNTRACKED means `fw` breaks in a clean clone and pre-commit guards fail open (T-2806/T-2807) |
| `check-go-propagation.sh` | check-go-propagation.sh — GO-recorded inceptions that never propagated their scope. | **ADVISORY** | fires on HUMAN decision backlog (GO triage); red means 'the operator has items', not 'the code is wrong' |
| `check-guard-severity-markers.sh` | check-guard-severity-markers.sh (T-3258) — every `advisory` guard-layer marker | **BLOCKING** | the demotion mechanism itself; must never be advisory |
| `check-handover-staleness.sh` | check-handover-staleness.sh (T-2883, G-019 prevention for the T-2882 class) | **ADVISORY** | session hygiene; time-driven |
| `check-hubs-parse-agreement.sh` | do the two hubs.toml parsers agree? | **BLOCKING** | two parsers disagreeing is a shipped-behaviour bug |
| `check-human-ac-escalation.sh` | scripts/check-human-ac-escalation.sh — T-3186 | **ADVISORY** | human-AC backlog escalation — human-owned by definition |
| `check-human-ac-steps-heading.sh` | Human AC "Steps" heading canonical-form check (T-2859). | **ADVISORY** | approval-page rendering of task files |
| `check-installed-binary-drift.sh` | is the fix you landed actually RUNNING (installed binary vs HEAD)? | **ADVISORY** | host state (is the fix running HERE) — says nothing about the artifact being released |
| `check-instructed-verb-resolves.sh` | T-3142 — an instruction names a verb that does not exist. | **BLOCKING** | an instruction naming a non-existent verb (T-3142) — operators act on it |
| `check-mcp-parity-census.sh` | check-mcp-parity-census.sh (T-2747, herdr rank 13 — coverage census for the MCP/CLI parity | **BLOCKING** | new MCP tool without parity decision; allowlist ratchet already absorbs the backlog |
| `check-pickup-deferred-freshness.sh` | T-2801 — stranded / stale auto-deferred pickup envelope check. | **ADVISORY** | inbound-queue hygiene; age-driven, goes red with no code change |
| `check-planted-default-gate.sh` | check-planted-default-gate.sh (T-2855, G-019 prevention for the class 832 named) | **BLOCKING** | planted defaults defeat a gate |
| `check-platform-lock.sh` | check-platform-lock.sh (T-2693, T-2690 G3 — Directive #4 Portability) | **BLOCKING** | Directive #4 — macOS binaries are published |
| `check-preflight-doc-set-drift.sh` | T-2188 — /preflight check-count drift canary. | **BLOCKING** | /preflight doc vs script check count |
| `check-receiver-ack-lag.sh` | subscribers that never acknowledge (sender-keyed; reads live bus state) | **ADVISORY** | reads LIVE bus state, permanently red on broadcast rails until T-3256; SKIPs in CI anyway |
| `check-release-artifact-drift.sh` | check-release-artifact-drift.sh (T-2751, G-019 prevention for the install-path drift class | **BLOCKING** | install.sh/release.yml/formula names — the release itself |
| `check-run-record-parse.sh` | T-3092 — a run record can be left UNPARSEABLE and nothing detects it. | **BLOCKING** | corruption detector |
| `check-silent-exit.sh` | check-silent-exit.sh (T-2666, G-019 prevention for the T-2663 silent-text-exit class) | **BLOCKING** | guards silent non-zero exits in the shipped CLI (Directive #2) |
| `check-stranded-finalized-tasks.sh` | scripts/check-stranded-finalized-tasks.sh (T-2833) | **ADVISORY** | task-ledger latch (vendored defect T-2833); deadlocks commits, not releases |
| `check-strict-star.sh` | check-strict-star.sh (T-2703, closing T-2702 finding F1) | **BLOCKING** | guards the substrate's decisive architecture invariant (strict star; no spoke-to-spoke connections) in shipped code |
| `check-task-frontmatter.sh` | check-task-frontmatter.sh (T-2794) | **BLOCKING** | corruption detector |
| `check-task-id-collisions.sh` | T-2800 — cross-branch task-ID collision + duplicate-work check. | **ADVISORY** | cross-branch state; can fire from someone else's branch |
| `check-task-template-idioms.sh` | check-task-template-idioms.sh (T-2777) | **BLOCKING** | template idioms propagate into every new task |
| `check-tier0-approval-latch.sh` | T-3139 — the Tier-0 approval surface latches PENDING and never clears. | **BLOCKING** | Tier-0 approval surface integrity (T-3139) |
| `check-unbounded-rpc-call.sh` | check-unbounded-rpc-call.sh (T-2669, G-019 prevention for the T-2641 hang class) | **BLOCKING** | guards hang class in shipped code |
| `check-unpaired-capture.sh` | capture-form verification legs with no failure assertion | **BLOCKING** | capture legs with no failure assertion — vacuous gate |
| `check-vacuous-verification.sh` | verification legs that cannot fail | **BLOCKING** | legs that cannot fail make P-011 green by construction |
| `check-vendor-divergence.sh` | T-2812 — unregistered local modifications to vendored framework code. | **DECIDE** | no release impact, but red means a re-vendor will DELETE a local fix; blocking the release is the only forcing function it has |
| `check-verification-heading-shadow.sh` | check-verification-heading-shadow (T-2877) — does the P-011 gate actually see | **BLOCKING** | P-011 not seeing the block; also the layer's long pole (~29% wall, T-3090) |
| `check-verification-misfile.sh` | check-verification-misfile.sh (T-2831) | **BLOCKING** | P-011 vacuous pass (T-2830) — protects the evidence behind every closed task |
| `check-version-derivation.sh` | check-version-derivation.sh (T-2746, G-019 prevention for the T-1458 / T-2744 class) | **BLOCKING** | a missing build.rs ships a plausible wrong version |
| `test-comms-selftest.sh` | test-comms-selftest.sh (T-2482) -- host-independent unit tests for the staged | **BLOCKING** | hermetic unit tests of a prover |
| `test-diagnose-unconsumed.sh` | test-diagnose-unconsumed.sh (T-2479) -- host-independent unit tests for the | **BLOCKING** | hermetic unit tests |
| `test-fleet-rearm-wakers.sh` | T-2404 — hermetic unit + dry-run tests for fleet-rearm-wakers.sh. | **BLOCKING** | hermetic unit tests |
| `test-mcp-desc-budget.sh` | test-mcp-desc-budget.sh (arc-005 mcp-slimming, T-2406) — anti-regrowth guard for | **BLOCKING** | anti-regrowth guard on shipped MCP descriptions |
| `test-pushwaker-ready-loop.sh` | T-2402 Stage 3 — integration test for the idle-gated ring loop. | **BLOCKING** | integration test for shipped waker loop |
| `test-session-selftest.sh` | test-session-selftest.sh (T-2485) -- host-independent unit tests for the | **BLOCKING** | hermetic unit tests |
| `fabric-workflow-link.sh` | fabric-workflow-link.sh — validate workflow steps registered as fabric components. | **DECIDE** | fabric bookkeeping — candidate ADVISORY |
| `invocation-usage.sh` | T-2996 (value-review C-45): report per-tool invocation counts. | **ADVISORY** | a usage REPORT (T-2996 C-45 telemetry), not a correctness assertion |
| `voi-prompt.sh` | T-3175 — ask about an unset voi_score, remember the answer, re-ask after N runs. | **ADVISORY** | FLAG: header says it 'never blocks — it only reports' (voi-prompt.sh:287,314) yet `--check` exits 1 in the layer; it is the OPERATOR's open question (T-3200). Either ADVISORY, or change its exit contract — do not leave it contradicting itself |

## Approval → implementation
Approve or amend per row. A follow-up build task then edits each approved member's marker to `# guard-layer: source advisory [existing args]  # <reason from this table>`. `check-guard-severity-markers.sh` refuses a marker with no reason, so each reason must be written out, not implied.

## Operator rulings (2026-09-29, T-3211 SQ walk-through)
- **Part 1 APPROVED in full:** all 12 ADVISORY rows (voi-prompt, check-go-propagation, check-human-ac-escalation,
  check-human-ac-steps-heading, check-handover-staleness, check-pickup-deferred-freshness,
  check-stranded-finalized-tasks, check-task-id-collisions, check-arc-slice-drift, check-installed-binary-drift,
  check-receiver-ack-lag, invocation-usage).
- **New requirement:** more than two classes — the operator named "warn, advisory, fail, etc." Tier vocabulary to
  be settled before implementation (proposal in the walk-through; see below once ruled).
- **Tier vocabulary APPROVED (three tiers):**
  - **FAIL** — blocks release; push CI red. (default for every member with no word)
  - **WARN** — never blocks release; push CI stays GREEN with the warnings surfaced as GitHub annotations and in the
    summary; a WARN member red for more than 14 days ESCALATES (the release-publication canary fires on it), so a
    warning cannot silently become ignored.
  - **INFO** — never blocks, never red; printed only (reports/statistics).
  - Part 1 mapping: `invocation-usage` -> INFO; the other 11 approved rows -> WARN.
- **Part 2 APPROVED (all six as recommended):** `check-arc-claim-drift` -> FAIL; `check-audit-warning-acknowledgement`,
  `check-budget-ladder-drift`, `check-episodic-parse`, `check-vendor-divergence`, `fabric-workflow-link` -> WARN.
- Part 3 (41 BLOCKING -> FAIL rows): pending.
