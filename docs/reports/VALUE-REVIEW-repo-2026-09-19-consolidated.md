# VALUE REVIEW — Consolidated & Deduplicated Findings (runs 1–5)

**Date:** 2026-09-19 · **Task:** T-2974 (consolidation) · **Source runs:** T-2971 runs 1–5
**Inputs read in full:** run2 report, run4 report, run5 report, run5 judge verdicts.
**Inputs skimmed for corroboration:** run1 evidence (O-rows), run3 evidence (§B–L rows).
Run1 and run3 are evidence-only artifacts (no classifications) — cited here only where they
settle or corroborate a fact.

> **Nothing here is authorization.** Every row awaits item-by-item human approval per the
> source runs' own Phase-6 gates. Class values are the source runs' verdicts, merged;
> conflicts are shown side by side, never averaged.

**Cross-run recurrence = confidence signal.** A finding independently produced by 2–3
blind runs is marked ×2 / ×3 in the Runs column.

**KEEP items:** the runs recorded 12 (run2), ~10 (run4), 13 (run5) KEEP verdicts; per the
consolidation rules these are not itemized here. No DELETE was verdicted in run5; run2/run4
DELETE rows appear with their gates noted.

---

## 1. Consolidated findings table

Legend: Class = source-run verdict(s). Reading = non-use diagnosis A–E where given.
Conf = highest-evidence run's rating (conflicts shown, not resolved). Route = agent |
human/sovereign.

| ID | Item | Location | Class | Reading | Runs (sources) | Conf | Proposal (condensed) | Size | Rev./Risk | Expected effect | Route |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **C-01** | 28 off-charter analytics MCP tools; T-2548 decision loop not closed, no removal task, IW-1 external-consumer check never discharged | `crates/termlink-mcp` (categories `agent_stats`/`agent_thread_health`/`agent_rankings`/`channel_engagement`); `.context/checks/charter-drift-allowlist`; T-2548 | run2: DELETE (pending SQ-1) · run4: DELETE · run5: INVESTIGATE — **see §3 D-1** | B + D + E (run4); D stacked with E (run5) | ×3 — run2 F1; run4 D1; run5 F-19/R5-19; run1 O-36 + run3 I-4/I-5 (evidence) | run4 **HIGH** vs run2 MEDIUM vs run5 HIGH-on-facts/LOW-on-verdict | (1) Reconcile the T-2548 record (run3 I-4 verifies `Decision: GO` gated on IW-1; run5 read the `## Decision` section as empty — fix whichever representation is wrong); (2) discharge IW-1 (needs cross-project check or the per-tool invocation counter, C-45); (3) then create the GO-build removal task: remove 24–25 non-KEEP tools, keep the 3 KEEP-listed + split the 5 `channel_engagement` tools per the allowlist NOTE; re-run charter-drift + parity census | Medium | Yes (`git revert`, p4 convention) / hidden external caller breaks silently if IW-1 skipped | Tool count 260→~232; allowlist 28→3; ~3,341 tokens (12.7% of MCP description payload) reclaimed per agent session (run5 E-57) | **human/sovereign** (T-2548; SQ set) |
| **C-02** | Deprecated-but-still-compiled MCP tools (run2: 40; run4: 46) still advertised to every client; 48-day soak elapsed | `tools.rs:1077-1217`; CLI hidden twins | DELETE ×2 | E | ×2 — run2 F2; run4 D2 + I6 | run2 **HIGH** vs run4 MEDIUM (deletion step lacks recorded go-ahead) | Present the soak + zero-caller evidence for the separate human go-ahead `p4-surface-reduction.md` itself requires, then delete; reconcile the 40-vs-46 count during scoping | Medium | Yes (git revert) / no telemetry rules out hidden external caller | Tool count →~214–220; parity-census denominator drops | **human/sovereign** (doc-mandated go-ahead) |
| **C-03** | Test/demo/probe residue: 20/45 live topics + 40/66 identity keys; 8 selftest topics, 6+ `aef-*` 1-message `forever` topics, demo topics | hub topic/identity state | DELETE (ops hygiene) | E | run2 F3; run3 D-2 (evidence corroborates topic list) | **HIGH** | Sweep/bound residue topics; delete stale keys; **confirm `health:ring20-fedprobe` arc status before touching that one** (→ C-14) | Small | Yes / low | Live topics 45→~25 | agent |
| **C-04** | CLAUDE.md ≈69.2k tokens (276,749 B, 3,283 lines; ~1,908 lines canary narrative) auto-loaded every session — measured mechanism behind Sept product-work collapse (1/1023 file-touches to `crates/`) | `CLAUDE.md` | REFACTOR ×2 | — | ×2 — run2 F4; run4 R1 (+ run4 Q5) | **HIGH** (both) | Split: small always-loaded governance core + per-script headers / linked `docs/operations/*` reference; one-line pointer table stays. Must preserve the T-2015 clobber-safe boundary; content preserved, not deleted | run2: needs-design · run4: Medium | Partially / sloppy extraction loses operator remediation text | 276,749 B → <80,000 B; preload ~69k → ~20k tokens; budget-gate-truncated commits trend down | **human/sovereign** (approach ratification — T-2015 boundary; run4 Q5) |
| **C-05** | MCP tool-description payload ≈26.4k tokens / 261 strings; arc-005 "mcp-slimming" status ambiguous | MCP descriptions; arc-005 | REFACTOR | — | run2 F5; run5 E-56 **contradicts staleness reading** — see §3 D-4 | run2 **HIGH** | Formally close or re-commit arc-005 (bookkeeping); shorten remaining outliers; C-01/C-02 removals deliver the biggest single cut (−12.7%) | Medium (mostly bookkeeping per run5) | Yes / none | Payload → ~15–23k tokens; arc audit warning stops | agent |
| **C-06** | `tools.rs` monolith: 46,458 LOC, 390 commits/180d (#1 churn), 1,403 fns, 9.2% parity coverage | `crates/termlink-mcp/src/tools.rs` | REFACTOR ×3 | — | ×3 — run2 F6; run4 R2; run5 F-07/R5-07 | run4/run5 **HIGH** (cost) vs run2 MEDIUM; run5 MEDIUM on readiness | **Sequence is load-bearing (run5):** (a) raise parity coverage via T-2748 ratchet on highest-churn portions FIRST — REFACTOR CHECK 2 fails at 9.2%; (b) then mechanical `mod` split along the 29 help categories. Do not touch deliberate T-2069 helper duplication (→ C-26) | Large, gated behind Medium coverage work | Partially / **high if sequence skipped** — silent regressions in low-coverage high-churn code | No mcp file >3–10k LOC; coverage % rises before split; churn spreads | agent |
| **C-07** | `commands/channel.rs`: 20,435 LOC, 166 commits/180d; cli+mcp = 68% of all code | `crates/termlink-cli/src/commands/channel.rs` | REFACTOR | — | run5 F-08/R5-08 | **HIGH** | Split **jointly** with C-06 (CLI groups mirroring MCP categories — same `channel.*` surface); same coverage gating caveat | Large | Partially / medium | Churn spreads across files | agent |
| **C-08** | `cli.rs` (7,001 L/356 commits) + `main.rs` (1,931 L/351 commits) secondary hotspots | `crates/termlink-cli/src/` | REFACTOR (investigate-first) | — | run5 F-09/R5-09 | MEDIUM | Churn-cause breakdown (one-line registrations vs logic) before any structural work | Small (investigation) | N/A / low | N/A until investigated | agent |
| **C-09** | V3 claim verbs trust spoofable `claimer`/`by`/`to_owner` (G-086/G-087 HIGH); no active task carries the substance | claim/renew/release/transfer handler sites (4+) | ADD(REPAIR) | A | ×2 — run2 F7; run4 I5 (no-task angle) | **HIGH** (run2) / MEDIUM (run4 I5) | Bind claim-verb identity to the verified sender fingerprint as `channel.post` already does; add negative-auth tests (none exist). Cheap **only while V3 has no users** — time-sensitive | Medium | Yes / nil blast radius today | G-086/G-087 resolved; negative-auth tests added | agent |
| **C-10** | Presence heartbeat can lie: `LIVE + armed` doesn't reflect real session availability; doorbell wake doesn't become a turn when recipient busy (G-083 HIGH) | heartbeat/waker rail | ADD(REPAIR) | A + B | run2 F8 | **HIGH** | Adopt T-2876's DELIVERED/BLOCKED/ENQUEUED vocabulary so heartbeat reflects true availability | needs-design | Yes / — | waker-liveness log shows fewer rail-dark entries within a month | agent |
| **C-11** | Push-wake rail dark NOW: 36 firings, 0 live wakers carry `pty_session` (named instance `penelope`); verb-2 degraded to ~15s poll floor fleet-wide | `.waker-liveness-canary.log` | ADD(REPAIR, operational) | — | run5 F-03/R5-03 | **HIGH** | Relaunch affected agents via `scripts/tl-claude.sh start --reachable …` (arm-at-relaunch; headless agents can't be retrofitted, PL-237) | Small per agent | Yes / low (brief presence gap, DM-durable) | 0 new FIRING entries 7 days; `pty_session` on relaunched agents | agent |
| **C-12** | Binary staleness firing 74–75 consecutive days, unactioned: hub serves 0.11.1766 vs floor 0.11.1944 (`arc-live-probe` SHIPPED-BUT-NOT-LIVE); **and** the preflight comparator can't distinguish "174 versions behind" from "1 product commit behind"; 3 installed binaries at 2 versions (PATH-dependent) | `.substrate-preflight-canary.log`; preflight Check 4; `/root/.cargo/bin` etc. | ADD(REPAIR) ×3 | A | ×3 — run2 F9; run4 A4; run5 F-02/R5-02 + E-49 | **HIGH** (all three) — but **diagnosis differs, see §3 D-3** | Two complementary fixes: (1) operational — rebuild/reinstall, restart hub through systemd unit (G-070), resolve the 3-binary install ambiguity; (2) comparator — compare against last tagged release or build-vs-source mtime, not the governance-inflated commit counter | Small (both) | Yes / none cited | `arc-live-probe` exit 0; `[WARN] binary` lines stop; log stays empty absent real drift — restores a channel deaf ~2.5 months | agent |
| **C-13** | `/canaries` masks genuinely-FIRING canaries as HEALTHY: predicate `FIRING = log mtime >= heartbeat mtime`; an ad-hoc run refreshes heartbeat without appending → flips FIRING→HEALTHY. Reported `firing:0` while ≥4 logs carried same-day findings | `scripts/canary-status.sh:20-21` | ADD(REPAIR) | — | run5 F-01/R5-01 (run5's central finding) | **HIGH** | Key FIRING on content newer than last **acknowledged/cleared** marker; a heartbeat touch must never clear a fire; change only the *clear* condition, never the *fire* condition; pin with a fixture. Run5 Q4 flags this as gate-adjacent — treat as bug fix (strengthens, never weakens) | Small | Yes / over-fire on heartbeat-only refresh (mitigated by clear-only change) | `/canaries firing` tracks non-empty-log count exactly, 7 daily runs | agent (flagged, run5 Q4) |
| **C-14** | `health:ring20-fedprobe`: largest topic (1,666–1,672 rec), `forever` retention, ~88 rec/day, single sender, 0 receipts, 38.8% of bus traffic; invisible to both guards built for the class | hub channel config | run4: REFACTOR · run5: INVESTIGATE · run2: DELETE-adjacent w/ confirm gate — **see §3 D-5** | D (run5) | ×3 — run2 F3(part)+contradiction 6; run4 R5; run5 F-21/R5-21; run3 J-7 (evidence) | run4 MEDIUM vs run5 LOW | Extract subscriber cursors + confirm the fed-probe arc's status; then set bounded retention per the recommended-settings table | Small | Yes / probe history rarely needed indefinitely | Topic ≤N records; bus-wide count −~35% | agent |
| **C-15** | A `forever` topic under the 50k ceiling with sustained daily growth is structurally invisible to both T-2252 and T-2562 | forever-archival / topic-growth checks | ADD(EXTEND) | — | run2 F10 | MEDIUM | Rate-based secondary trigger (records/day over N days) in one existing check | Small | Yes / low | Fixture gains a `rate_flagged` case | agent |
| **C-16** | Agent-dispatch gate: documented per-session, behaves per-project-lifetime; blocked attempts increment the counter; blocked all three runs live | `.agentic-framework/agents/context/check-agent-dispatch.sh` (+`post-compact-resume.sh:42-48`) — **vendored (G-062)** | ADD(REPAIR — file upstream) ×3 | A | ×3 — run2 F11; run4 A1; run5 F-11/R5-11 | **HIGH** (run2/run4, observed live) vs run5 MEDIUM | Do not patch locally. File upstream (T-2304/T-2469/T-2813 pattern); record in `.vendor-divergence.yaml`. Fix: reset on genuine cold start / key to real per-conversation id (over-eager reset risk noted, run4) | Small | N/A / runaway-session risk if reset is naive | Fresh session dispatches without `BLOCKED`; no manual `fw dispatch reset` | agent |
| **C-17** | Dispatch-gate remediation text cites dead CLI path (`fw termlink dispatch` yields no usage) | `check-agent-dispatch.sh` remediation string | ADD(REPAIR) | A | run4 A7 (also run4 contradiction 9) | **HIGH** | Fix text to a working invocation, or make `fw termlink dispatch --help` work | Trivial | Yes / none | `--help` prints usage | agent |
| **C-18** | Guard-layer runner exceeds documented "seconds" budget (~20 min run4; >600 s run5); `--json` all-or-nothing — interrupted run yields 0 bytes; gates every push/PR against a 10-min CI cap; layer completed 109 PASS/4 FAIL/0 ERROR in run5 text mode | `scripts/run-guard-layer.sh`; `doc-lint.yml`, `release.yml` | ADD(REPAIR) ×2 | A | ×2 — run4 A2 (+run2 I5 CI-cap angle); run5 F-06/R5-06 | **HIGH** (run5, raised from JUDGE's MEDIUM after a second completing run; run4 HIGH) | Stream/flush per-member JSON results incrementally; profile the 113–191 members; split fast/slow CI jobs; fix the "(seconds)" doc claim regardless | Small–Medium | Yes / real defects pass through if CI silently soft-times-out today | `--json` yields partial output when interrupted; documented budget matches measured runtime | agent |
| **C-19** | 7 firing canaries, **0** corresponding active tasks — detection layer works, response layer manual and one-human-wide | `.context/working/.*-canary.log` ↔ `.tasks/active/` | ADD(WIRE) | B (response half) | run4 A3 | **HIGH** | Auto-file a task on firing (mirror framework-pickup's self-filing pattern), de-duplicated by canary name — de-dup is load-bearing | Medium | Yes / naive filer spams duplicates per cron cycle | ≥1 active task naming each firing canary within 24h of next cron | agent |
| **C-20** | Hook-counter corruption: unlocked truncate+write race; two readers of same file disagree (1 vs 7); counter is the **denominator** of the T-1626 decay alarm → false silence; "L-023 recurring" | `lib/hook-telemetry.sh::_fw_telemetry_increment` — **vendored (G-062)** | ADD(REPAIR — file upstream) ×2 | A | ×2 — run4 A5; run5 F-04/R5-04 | **HIGH** (both) | `flock` the read-modify-write; file upstream, local wrapper only as explicit stopgap | Small | Yes / negligible contention | Readers agree; integrity canary stops firing 30 days | agent |
| **C-21** | 71 partial-complete tasks blocked on human ACs, median **125 days**, no staleness surface; G-008 register stale (recorded 64, measured 71, 157 days apart) | `.tasks/active/*` `owner: human`; G-008 | run4: ADD(SURFACE) · run5: INVESTIGATE (governance) — **see §3 D-7** | C + A (run4; T-2859 renderer defect vendored) | ×3 — run4 A6; run5 F-22/R5-22; run2 I6/SQ-4 | run4 MEDIUM; run5 HIGH-on-fact | Agent half: add a staleness surface (extend `/canaries` or handover banner) for partial-complete over threshold, mirroring the `revisit_at` G-053 pattern; update G-008's figure. Human half: triage cadence / AC narrowing → §2 | Small–Medium | Yes / none | 56 tasks >30 d old surfaced at session start; count tracked per review | agent (surface) + **human/sovereign** (cadence) |
| **C-22** | 11 unacknowledged inbound peer filings on `framework:pickup` (offsets 106–120: 4 bug reports, 2 proposals, 5 learnings); G-063 literal recurrence, open 105 days; +1 STRANDED envelope P-078 (9 days, no breadcrumb — already tracked T-2960) | `.framework-pickup-canary.log`; `.context/pickup/auto-deferred/` | ADD(REPAIR + WIRE) | B | run5 F-05/R5-05 + E-50; run3 D-7 (evidence) | **HIGH** | Two owners: (1) triage + ack the 11 now (small, bounded); (2) G-063 structural consumer is new-subsystem territory — **inception-size it**, don't build ad hoc | Small / Large | Yes / scope creep without inception | Pickup canary clear of the 11 after ack | agent (triage) + inception for consumer |
| **C-23** | Project-boundary gate false-positives on the project's own hub runtime dir (`cd` into `/var/lib/termlink` blocked; identical absolute-path read succeeds) | `check-project-boundary` hook (framework) | ADD(REPAIR) | — | run5 F-10/R5-10 | MEDIUM | Recognize declared hub `runtime_dir` as in-bounds **for reads**, or normalize `cd` vs absolute-path through one check. Narrows a false positive; does not weaken the boundary | Small | Yes / low | No false blocks on read-only access to declared runtime paths | agent |
| **C-24** | Cluster of stale/contradicted CLAUDE.md claims: `FRAMEWORK.md` ref; "(seconds)"; "per session"; `api-usage --json`; dispatch remediation text; canary tally "all eighteen" vs 21+ | `CLAUDE.md` various | REFACTOR (5+ one-line fixes) | — | run4 R6; run2 contradiction 7 | **HIGH** | Fix or remove each contradicted claim | Small | Yes / none | 0 of the claims contradicted on next verification pass | agent |
| **C-25** | Governance-artifact volume: 105 MB vs ~6 MB product source; `.context/handovers` 59 MB / 1,706 files; 24:1 touch ratio | `.context/handovers`, `.tasks`, episodic, audits | REFACTOR | — | run4 R4 | MEDIUM | Apply the project's own T-2562 retention discipline to its own trees — compress/**archive** >90 days (never delete) | Small–Medium | Yes (archive) / forensic loss if archive unreachable | handovers 59 MB → <20 MB active | agent |
| **C-26** | 67 duplicated `_mcp`/CLI helper pairs (T-2069 convention); duplication already produced one 45-day undetected parity drift | `termlink-mcp/tools.rs` ↔ `termlink-cli/commands/*` | REFACTOR | — | run4 R3 (+run4 Q3) | MEDIUM | If T-2069 revisited (sovereign): extract shared pure fns. **Else (no sovereignty needed):** add a check that both twins changed in the same commit | Medium | Yes / shared crate reintroduces the coupling T-2069 forbids | Pairs 67→0, or 0 undetected drifts in 90 days | **human/sovereign** (convention) · agent (paired-commit check) |
| **C-27** | `orchestrator.route` — hub-implemented, zero client surface, zero calls; name overlaps non-goal #4; federation tripwire treats it as live residual path | `hub/src/router.rs:78,1186` | INVESTIGATE | B + weak E | run5 F-12/R5-12 (+run5 Q2) | MEDIUM | Task/design-history check: shelved scaffolding → DELETE becomes runnable; wanted → ADD(WIRE). Don't delete code the tripwire needs | Small | N/A / wrong either way if skipped | N/A until investigated | **human/sovereign** (non-goal #4 ruling) |
| **C-28** | `dialog.presence` — hub-implemented (T-1286/T-243), documented purpose, zero client surface, zero calls | `hub/src/router.rs:140`, `channel.rs:28` | INVESTIGATE | B | run5 F-13/R5-13 | MEDIUM | Check T-1286/T-243 history; purpose served elsewhere → DELETE, else small ADD(WIRE) — tracker already exists | Small | Yes if WIRE / low (inert) | N/A until investigated | agent |
| **C-29** | `artifact.get`/`put` — fully wired, hub-visible, zero calls in 34.2 d | hub+cli+mcp | INVESTIGATE | none established (N-4) | run5 F-14/R5-14 | MEDIUM (zero measured; reason not) | Extend window to 90+ days before any verdict; no code action | N/A | — / premature DELETE removes a built capability on a short sample | Re-check at 90+ d | agent |
| **C-30** | `kv.*` usage structurally invisible: session-daemon calls never pass the hub audit sink; `kv.get` has 0 hub implementation | hub/cli/mcp; `hub/src/server.rs` audit sites | INVESTIGATE + ADD(instrument) | **D — decisive** | run5 F-15/R5-15 | **HIGH** (diagnosis) | No value verdict until instrumented: session-daemon counter or route `kv.*` through the shared audit sink | Small–Medium | Yes / low | Future review classifies kv.* on real data | agent |
| **C-31** | `session.*` lifecycle methods — zero hub-observed calls; same blind spot as kv.* plausibly applies (not individually verified) | session daemon RPCs | INVESTIGATE + ADD(instrument) | D (strong by analogy) | run5 F-16/R5-16 | MEDIUM | Run the same per-method hub/cli/mcp wiring check used for N-1…N-4 first | Small | N/A / low | Confirms/refutes D reading | agent |
| **C-32** | `event.emit/subscribe/topics/state_change/error` — zero calls despite CLI/MCP surface; `event.subscribe` IS hub-implemented yet zero-call (argues against pure D) | event family | INVESTIGATE | unclear | run5 F-17/R5-17 | LOW | Per-method `LEGACY_METHODS` + wiring check on each member; family may not share one fate | Small | N/A / low | Retirement-family vs underused-but-wanted resolved | agent |
| **C-33** | `event.broadcast` — recorded intent to retire (`LEGACY_METHODS`, "targeted for retirement"), zero calls. **DELETE candidate that failed the checklist:** DELETE CHECKS 4–5 (reference sweep; external consumers) not run — per rules, held at INVESTIGATE | `hub/src/rpc_audit.rs:54-58` | INVESTIGATE (DELETE-leaning) | **E — direct** (strongest E-reading in run5) | run5 F-18/R5-18 | MEDIUM (procedural, not evidential) | Targeted reference sweep (strings/config/hooks/CI/prompts); if clean, converts to DELETE with high confidence; state git-revert plan in follow-up | Small | Yes (git revert) / low — retirement already declared | Sweep clean → method removed, LEGACY entry dropped | agent |
| **C-34** | `channel.create` ≈ `channel.post` (52,695 vs 53,538; ratio 0.98) against only 45 topics — 5.7% of all hub RPCs possibly redundant ensure-topic overhead | client ensure-topic path | INVESTIGATE | — (cost, not use) | run5 F-20/R5-20 | LOW | Measure hub-side cost of create-on-existing; if non-trivial, client-side "topic known to exist" cache | Small | N/A / low | create's share of RPC volume drops | agent |
| **C-35** | **48.7% of GO inceptions never propagate scope into build tasks** (77 of 158); recurs in ≥11 of last 30 audits; T-2548 (C-01) is one instance of the systemic pattern | `.context/audits/*.yaml` | INVESTIGATE (process defect) | — | run5 F-23 (self-judged, post-cutoff) | MEDIUM (measured; reduced one level as self-judged) | Treat inception→build handoff as one process defect, not 77 oversights; root-cause why a daily-reported 48.7% leak produces no action | Medium (process) | N/A / high to keep ignoring — silently drops half of approved work | Ratio falls over 30 days of audits | agent (investigation) |
| **C-36** | V3 "claim work": built, documented, skilled, canaried — zero production callers (108 calls/34.2 d = 0.012%, all demo/prover); AEF doesn't call it | claim lifecycle | INVESTIGATE (run2) — run5 verdicts **KEEP**, see §3 D-6 | run2: NEVER-WIRED + BROKEN · run5: none (low volume expected for work-stealing) | ×2 — run2 I1/SQ-2; run5 KEEP + N-6 | — (class conflict) | Determine whether AEF has a roadmapped adoption plan (unreachable from this repo, T-559). Per run2 SQ-2: keep the verb, fix the guarantee (C-09), **freeze surface growth** until something calls it | — | — | — | **human/sovereign** (charter verb status) |
| **C-37** | Bypass volume: 482 total; 313 = focus-drift (164) + `--skip-sovereignty` (149, 2nd-most-bypassed) — Authority-Model question | `.gate-bypass-log.yaml` | INVESTIGATE | — | ×2 — run2 I2; run4 I4 (+run4 Q4) | MEDIUM | Audit a sample of 20 bypass entries with reasons **before concluding anything; do not weaken the gate on volume** — likely reading: human-verification path saturated (→ C-21), not gate defect | Small | — | — | **human/sovereign** (Authority Model) |
| **C-38** | 66–67% of component-fabric cards have no edges (326/491 run2; 338/503 run5) — `fw fabric blast-radius`/`deps` return nothing for two-thirds of components | `.fabric/components/` | INVESTIGATE | — | ×2 — run2 I3; run5 E-54 | MEDIUM | Determine whether fabric verbs are actually consulted (needs invocation telemetry, C-45); then backfill edges or descope | — | — | — | agent |
| **C-39** | `stale-waker-code` and `stuck-claims` logs read "firing" but haven't appended in 34–36 days despite installed daily cron; 5 of 7 firing canary logs carry no per-entry ISO date | canary logs | INVESTIGATE (+ADD: per-entry dates) | — | ×2 — run2 I4; run4 S17.5 (data-gap) | MEDIUM | Per-canary heartbeat freshness check ("cron stopped" vs "condition stable"); add ISO dates to every canary log entry | Small | Yes / low | Blip vs chronic distinguishable for all canaries | agent |
| **C-40** | Chat-arc collapse/concentration: 0 posts + 0 unique speakers for 10 consecutive daily snapshots (progressive decline 270.4→12.1→31.9→zeros); historically 92% single-sender (871/950) | fleet-adoption snapshots; chat-arc | INVESTIGATE | D/E ambiguous | ×2 — run4 I2 (+GATHERER note); run2 I7 | MEDIUM | Check hub-dedup fix's commit date vs zero-streak start (step vs decline); identify sender `d1993c2c3ec44c94`'s role | Small | — | — | agent |
| **C-41** | Fleet-binary canary reports healthy while `.121`/`.122` run ≈355 commits behind — floors silently permissive for exactly the flagged hosts; true staleness needs per-hub build sha (patch numbers not comparable across tag epochs) | `fleet-version-floors.conf` | INVESTIGATE | D-adjacent | ×2 — run4 I1 + contradiction 10; run5 §8 fleet row (E-26/27) | MEDIUM | Read floors for `.121`/`.122`: deliberate exemption or floor too low? Get actual build sha per hub | Small | — | — | agent |
| **C-42** | Task completion fell ~8× (609/mo → ~75/mo) while commits held at 447/30d | task ledger | INVESTIGATE | — | run4 I3 | LOW | LOC-changed or session-count per completed task, by month (bigger tasks vs less work) | Small | — | — | agent |
| **C-43** | Receiver ack-lag: on `agent-chat-arc`, 4 of 5 identities have NEVER acked; lag 1,533 — write-only-sink shape (G-063) on the fleet's main broadcast topic | `check-receiver-ack-lag.sh` (guard FAIL) | INVESTIGATE | — | run5 post-cutoff (self-judged); run3 D-6 (evidence) | MEDIUM (self-judged) | Investigate read-side consumption; note rows keyed by identity fingerprint — shared keypairs measure a host, not an agent (T-2838) | Small | — | — | agent |
| **C-44** | `cron-drift-firing-fixtures.sh` — a guard's own regression fixture suite is red (baseline guard FAIL, uninvestigated) | `tests/` | INVESTIGATE | — | run5 post-cutoff | LOW (flagged, no root cause) | Root-cause: a failing fixture suite can silently degrade every other guard's reliability | Small | — | — | agent |
| **C-45** | **No per-tool MCP / per-verb CLI invocation telemetry anywhere** — the single biggest gap in all runs; caps every usage verdict on 260 tools + 40 verbs, and is the prerequisite for deciding C-01 on evidence | new instrument | ADD(instrument) | — | ×3 — run2 §9; run4 §9/S17.1; run5 gap #1 ("the highest-value ADD nobody has asked for") | **HIGH** (all runs converge) | Per-verb/per-tool counters, the way `fleet-adoption-snapshot` already does for the bus; covers session-scoped surfaces the hub audit misses (C-30/C-31) | Medium | Yes / low | Converts the surface question from inference to measurement; unblocks IW-1/T-2548 | agent |

**Totals (unique findings):** ADD **17** (C-09..C-13, C-15..C-20, C-22, C-23, C-45, plus instrument halves of C-30/C-31 counted with their rows) · REFACTOR **8** (C-04..C-08, C-24, C-25, C-26) · INVESTIGATE **16** (C-14, C-27..C-44 net of ADD-hybrids) · DELETE-class (gated) **3** (C-01, C-02, C-03; no run-5 DELETE verdicts — C-33 is the noted failed-DELETE-check candidate) · **45 consolidated rows** overall (some hybrid INVESTIGATE+ADD).

---

## 2. Sovereign questions (all runs, condensed — verbatim substance preserved)

None of these is an agent's to decide. Recommendations are the source runs' own.

1. **[run2 SQ-1 · run4 Q1 · run5 Q1] The 28 off-charter analytics tools: subtract, or amend the charter?** T-2548 records GO-to-subtract gated on IW-1 (run3 I-4 verbatim); run5 read the Decision section as empty; no removal task exists; measured cost ~3,341 tokens/agent/session. Rec (all runs): record/confirm the decision explicitly; clear IW-1 (cheapest route: build the invocation counter, C-45); then a scoped removal task. Leaving it acknowledged-but-undecided is a standing cost every review re-discovers.
2. **[run4 Q2] Execute the deletion of the 40/46 deprecated tools?** Deprecation was human-decided; `p4-surface-reduction.md` makes deletion a distinct still-open decision. Rec: given 48-day soak + zero callers, grant it — but the written gate makes it explicitly the human's.
3. **[run2 SQ-2] Is V3 "claim work" still a core verb?** 0 production callers; exclusivity unenforced (G-086/G-087). Rec: keep the verb, fix the guarantee, stop extending it until something calls it. (Run5 independently verdicts KEEP.)
4. **[run2 SQ-3] Budget-gate thresholds given ~95.5k tokens fixed overhead.** Rec: do not weaken the gate — reduce the overhead (C-04/C-05); the cost is the defect.
5. **[run2 SQ-4 · run4 Q4] 137 human-owned active tasks / 71 fully-done awaiting human ACs (median 125 d); `--skip-sovereignty` ×149.** Rec: audit a bypass sample before concluding; schedule verification sessions or narrow which ACs genuinely need human sign-off. Do not let agents close them.
6. **[run2 SQ-5] Four stalled arcs (arc-001/002/005/007), oldest since 2026-06-07.** Rec: close or re-commit each explicitly. (Run5 E-56: arc-005 is bookkeeping-stale, not rotting.)
7. **[run4 Q3] Revisit the T-2069 no-cross-crate-sharing convention?** Already cost one 45-day drift. Rec: worth revisiting on measured cost; cheaper alternative (paired-commit check) needs no convention change.
8. **[run4 Q5] CLAUDE.md restructuring intersects the T-2015 clobber boundary.** Rec: extract into script headers (established pattern) rather than invent a second protected region.
9. **[run4 Q6] Is the guard/governance layer in this repo TermLink, or AEF policy living here?** Non-goal #4 says the substrate stays mechanism. ~⅓ of run4's framing depends on the answer; yardstick UNCONFIRMED without it.
10. **[run5 Q2] Does `orchestrator.route`'s existence conflict with non-goal #4?** Hub-implemented, client-unreachable, name overlaps the non-goal; tripwire treats it as intentional. Rec: human ruling.
11. **[run5 Q3] Does G-064 (no per-user authorization, open 103 d) conflict with non-goal #5?** Rec: human ruling on whether G-064 targets something narrower (audit attribution) so it can be scoped precisely.
12. **[run5 Q4] Is the `/canaries` predicate change a gate change?** It strengthens reporting. Rec: treat as ordinary bug fix (clear-condition-only change); flagged rather than assumed.

---

## 3. Run disagreements

| # | Item | run2 | run4 | run5 | Evidence-strength note |
|---|---|---|---|---|---|
| D-1 | 28 analytics tools (C-01) | DELETE pending SQ-1, MEDIUM | DELETE, **HIGH** (reads T-2548 as decided GO, S4.6) | INVESTIGATE — reads T-2548 `## Decision` as **empty**, verdict capped LOW | **run1 O-36 and run3 I-4 both verify a recorded `Decision: GO` (gated on IW-1) in the task's Updates/Decision block.** Run5's "empty Decision section" likely read a different section of the file; the *substance* all runs agree on: GO recorded, IW-1 never discharged, no build task exists. The consolidated row treats reconciling the record as step 1. |
| D-2 | Deprecated tool count (C-02) | **40** tools, HIGH | **46** tools (8 wholly-deprecated categories), MEDIUM | — | Count must be re-measured at scoping; both runs agree on the class and the missing go-ahead. |
| D-3 | Binary-staleness diagnosis (C-12) | Comparator is structurally unsatisfiable (governance-inflated counter); binary only 1 product commit behind — fix the check | Binary genuinely stale — upgrade + restart | Binary genuinely stale (arc-live-probe SHIPPED-NOT-LIVE; 3 binaries at 2 versions) — restart | Run2's own contradiction #4 resolves it: **both readings are true** — they measure different things. Consolidated proposal carries both remediations. |
| D-4 | arc-005 mcp-slimming status (C-05) | Stalled through 18 consecutive audit warnings — "revive or formally close" | (not judged) | E-56: work **largely landed** (156→105 KB; worst description 11,751→1,546 chars) — bookkeeping-stale, not rotting | Run5's is the measured reading (it checked the payload); run2's is audit-warning-derived. Favor run5 for the diagnosis; run2's "formally close" remedy stands either way. |
| D-5 | `health:ring20-fedprobe` (C-14) | DELETE-adjacent within F3 but "confirm the arc's status before deleting"; also contradiction vs non-goal #2 | REFACTOR — set bounded retention now, MEDIUM | INVESTIGATE — extract subscriber cursors first, LOW | All agree the topic violates the non-goal's spirit and both guards are blind to it; they differ only on whether to bound now or measure readership first. Consolidated: measure-then-bound. |
| D-6 | Verb-3 claim work (C-36) | INVESTIGATE (I1) + SQ-2: NEVER-WIRED + BROKEN; keep verb, freeze surface | (implicit — I5 on the missing G-086/087 task) | **KEEP** with explicit reasoning: low volume is what a work-stealing primitive should look like; A–D ruled out | Not a true contradiction — both keep the verb — but the *posture* differs (freeze-growth vs healthy-as-is). Fix-the-guarantee (C-09) is common ground. |
| D-7 | Human-AC backlog (C-21) | INVESTIGATE (I6) + SQ-4 | **ADD(SURFACE)**, MEDIUM — build the staleness surface | INVESTIGATE — governance question, no code fix | Run4's ADD is the only actionable agent-side step; run5 frames the cadence half as sovereign. Consolidated row splits it accordingly. |
| D-8 | Guard-layer runtime (C-18) | I5: unmeasured (contended local runs) | ~20 min measured, HIGH | >600 s + 0-byte JSON, then a completing text-mode run (109/4/0) — JUDGE's MEDIUM raised to HIGH | Run5's second run resolved run4's single-data-point worry; no live disagreement remains. |
| D-9 | Canary population tally | "all eighteen" (CLAUDE.md) vs 21 sections/21 logs/27 crontabs (run2 contradiction 7) | 22 canary logs, 24 crontabs | 30 canaries (judge R5-01) | Doc-hygiene item, folded into C-24. |

---

## 4. Data gaps / instrumentation ADDs (deduplicated §9s of all runs)

Each gap that blocked a verdict is itself an ADD candidate (all runs applied this rule).

1. **Per-tool MCP / per-verb CLI invocation telemetry — ABSENT** (run2, run4 S17.1, run5 gap 1). The single highest-value instrument; prerequisite for C-01/IW-1, C-30, C-31, C-38. → consolidated as finding **C-45**.
2. **External/cross-project consumer visibility (IW-1) — BLOCKED by T-559** (run2, run4 S17.2, run5). Blocks DELETE CHECK 5 for everything on the install path; release/download telemetry also absent (run1 O-49).
3. **CI run history / pass rate — ABSENT** (run2, run4): whether the guard layer passes in CI and whether the 10-min cap is hit.
4. **Line coverage — ABSENT** (run2, run4 S17.7, run5): REFACTOR CHECK 2 precondition for C-06/C-07.
5. **Unused-dependency analysis — ABSENT** (`cargo-udeps`/`cargo-machete` not installed; run2, run5): a whole DELETE axis (30 workspace deps / 310 lockfile packages) invisible.
6. **BVP realization log — ABSENT/unverified** (run2, run4 S17.8): "did shipped arcs deliver?" unanswerable; also the only route to validating the runs' own pre-registered expected effects.
7. **Hub-side silent-drop/discard telemetry — ABSENT** (run4 S17.3): caps any reliability claim on verb 2; G-088 class.
8. **Per-entry ISO dates missing in 5 of 7 firing canary logs** (run4 S17.5; run2 I4): blip vs chronic indistinguishable. → C-39.
9. **Out-of-band delivery observer — on-demand only** (run5): verb 2's durability is bus-self-reported except when `session-message-selftest.sh` is run manually; schedule it or accept the gap.
10. **Subscriber cursor/offset extraction per topic — not done** (run5): caps C-14 and all topic-"usage" claims beyond raw counts.
11. **Audit-history trend analysis across 151 audit files — not done** (run5): would turn C-21/C-35 point measurements into trend lines.
12. **Guard-layer per-member timing — unavailable** (run4 S17.6, run5): needed for C-18.
13. **Healing events / per-task token telemetry — not sampled** (run2).
14. *(Already tracked, noted for completeness: `substrate-smoke` crontab MISSING from `/etc/cron.d` — run2 baseline, run3 J-6, tracked as T-2939.)*

---

## 5. Suggested task decomposition (one deliverable per task)

Per Task Sizing Rules. Order ≈ the runs' own value-per-cost ranking. "human" = owner:
human or sovereign gate before/within the task.

| # | Proposed task title | IDs | Type | Owner |
|---|---|---|---|---|
| S-1 | Fix `/canaries` FIRING→HEALTHY masking predicate + fixture | C-13 | build | agent |
| S-2 | Fix substrate-preflight binary comparator (tag/mtime-based) | C-12 (check half) | build | agent |
| S-3 | Operational: reinstall current binary, resolve 3-binary PATH ambiguity, restart hub via systemd, verify with arc-live-probe | C-12 (ops half) | build | agent |
| S-4 | Re-arm push-wake rail: relaunch unwakeable agents via tl-claude.sh | C-11 | build | agent |
| S-5 | Triage + ack the 11 inbound framework:pickup filings | C-22 (triage half) | build | agent |
| S-6 | Inception: structural consumer for framework:pickup (G-063) | C-22 (consumer half) | inception | human (go/no-go) |
| S-7 | Bind claim-verb identity to verified sender fingerprint + negative-auth tests (G-086/G-087) | C-09 | build | agent |
| S-8 | File upstream: dispatch-gate counter semantics + dead remediation text; register in .vendor-divergence.yaml | C-16, C-17 | build | agent |
| S-9 | File upstream: hook-telemetry flock fix (L-023) | C-20 | build | agent |
| S-10 | Guard-layer runner: streaming per-member JSON + member profiling + doc-budget fix | C-18 | build | agent |
| S-11 | Wire firing canaries to de-duplicated auto task-filing | C-19 | build | agent |
| S-12 | Fix stale CLAUDE.md factual claims (FRAMEWORK.md, "(seconds)", "per session", api-usage, canary tally) | C-24 | build | agent |
| S-13 | Sweep test/demo/probe residue topics + stale identity keys (fedprobe excluded) | C-03 | build | agent |
| S-14 | Investigate-then-bound `health:ring20-fedprobe` (cursors → retention) + rate-based forever-topic trigger | C-14, C-15 | build | agent |
| S-15 | Inception: CLAUDE.md split design (clobber-safe, target <20k tokens preload) | C-04 | inception | human (ratify approach; run4 Q5) |
| S-16 | Close out arc-005 bookkeeping + shorten remaining MCP description outliers | C-05 | refactor | agent |
| S-17 | Raise MCP parity coverage on highest-churn tools.rs regions (T-2748 ratchet) — split precondition | C-06 (gate) | test | agent |
| S-18 | Inception: coordinated tools.rs + channel.rs split design (post-S-17) | C-06, C-07, C-08 | inception | human (go/no-go on sequencing) |
| S-19 | Reconcile T-2548 record, discharge IW-1, then (on human GO) scoped removal of 28 analytics tools | C-01 | build (gated) | **human** |
| S-20 | Present deprecated-tool deletion go-ahead (soak evidence), then remove 40/46 | C-02 | build (gated) | **human** |
| S-21 | RPC-surface wiring-check sweep: orchestrator.route, dialog.presence, session.*, event.* family + event.broadcast reference sweep (may convert C-33 to DELETE) | C-27, C-28, C-31, C-32, C-33 | inception | agent (C-27 ruling: human) |
| S-22 | Per-tool/per-verb invocation telemetry instrument (incl. kv.* session-daemon blind spot) | C-45, C-30 | build | agent |
| S-23 | Human-AC staleness surface (partial-complete > threshold at session start) + refresh G-008 | C-21 (surface half) | build | agent |
| S-24 | Governance-artifact archival (>90 d handovers/audits; archive, never delete) | C-25 | refactor | agent |
| S-25 | Paired-commit drift check for `_mcp`/CLI helper twins (no T-2069 change) | C-26 (alt half) | build | agent |
| S-26 | Project-boundary read false-positive fix (runtime_dir in-bounds for reads; upstream if vendored) | C-23 | build | agent |
| S-27 | Presence-heartbeat truthfulness design (DELIVERED/BLOCKED/ENQUEUED vocabulary) | C-10 | inception | agent |
| S-28 | Canary log hygiene: per-entry ISO dates + heartbeat-freshness cross-check (34–36-day silent logs) | C-39 | build | agent |
| S-29 | Investigation bundle wrap-ups (each its own small task at creation time): GO-propagation leak (C-35), chat-arc collapse (C-40), fleet floors `.121`/`.122` (C-41), completion-rate drop (C-42), ack-lag read side (C-43), red fixture suite (C-44), fabric edges (C-38), artifact.* 90-day revisit_at (C-29), create/post ratio (C-34) | C-29, C-34, C-35, C-38, C-40..C-44 | inception/test | agent |
| S-30 | Sovereign-question review session (record decisions for §2 items 1–12; bypass-sample audit C-37) | C-36, C-37, §2 | inception | **human** |

## 6. Execution mapping (arc-009, T-2974)

All slices above are filed as tasks under **arc-009** ("Value-review execution: five-run
findings remediation", anchor T-2974). Coverage is 1:1 with §5; no finding is dropped.
Tasks marked **human** are `owner: human` and surface on the Watchtower review/inception
queues; agents do not complete them.

| Slice | Task | Consolidated IDs | Owner |
|---|---|---|---|
| S-1 | T-2975 | C-13 | agent |
| S-2 | T-2976 | C-12 (check half) | agent |
| S-3 | T-2977 | C-12 (ops half) | agent |
| S-4 | T-2978 | C-11 | agent |
| S-5 | T-2979 | C-22 (triage half) | agent |
| S-6 | T-2984 | C-22 (consumer half) | **human** (inception) |
| S-7 | T-2980 | C-09 | agent |
| S-8 | T-2981 | C-16, C-17 | agent |
| S-9 | T-2982 | C-20 | agent |
| S-10 | T-2983 | C-18 | agent |
| S-11 | T-2985 | C-19 | agent |
| S-12 | T-2986 | C-24 | agent |
| S-13 | T-2987 | C-03 | agent |
| S-14 | T-2988 | C-14, C-15 | agent |
| S-15 | T-2989 | C-04 | **human** (inception) |
| S-16 | T-2990 | C-05 | agent |
| S-17 | T-2991 | C-06 (gate) | agent |
| S-18 | T-2992 | C-06, C-07, C-08 | **human** (inception, DEFER until S-17) |
| S-19 | T-2993 | C-01 | **human** (gated build) |
| S-20 | T-2994 | C-02 | **human** (gated build) |
| S-21 | T-2995 | C-27, C-28, C-31, C-32, C-33 | agent (C-27 ruling: human) |
| S-22 | T-2996 | C-45, C-30 | agent |
| S-23 | T-2997 | C-21 (surface half) | agent |
| S-24 | T-2998 | C-25 | agent |
| S-25 | T-2999 | C-26 (alt half) | agent |
| S-26 | T-3000 | C-23 | agent |
| S-27 | T-3001 | C-10 | agent (inception) |
| S-28 | T-3002 | C-39 | agent |
| S-29a | T-3003 | C-35 | agent (inception) |
| S-29b | T-3004 | C-40 | agent (inception) |
| S-29c | T-3005 | C-41 | agent (inception) |
| S-29d | T-3006 | C-42 | agent (inception) |
| S-29e | T-3007 | C-43 | agent (inception) |
| S-29f | T-3008 | C-44 | agent (test) |
| S-29g | T-3009 | C-38 | agent |
| S-29h | T-3010 | C-29 | agent |
| S-29i | T-3011 | C-34 | agent |
| S-30 | T-3012 | C-36, C-37, §2 (12 sovereign questions) | **human** (inception) |
