---
reviewer: GLM-5.3
task: T-3694
question: W1 pair prompt-origin route
recommended_option: C
agrees_with_agent: partly
confidence: medium
scores: {A: 11, B: -1, C: 34, D: -16}
top_risk: "Any silent failure in C's per-prompt fetch (missing TERMLINK_SIDECAR_SOCKET, unbound S1, meter timeout) makes the claim land origin-less/unknown, so the moment origin reporting is switched on every W1-pair claim is ignored, main starves, and mail has no main to reach — with no error surfaced."
blocking_unknowns: ["Whether the meter's PTY-counter source (S1) is bound for both W1 pair sessions (unbound forces checked origin to unknown, never human)", "Whether AEF will accept a W1-conditional branch in sidecar-inbox given its stated policy 'never both on one session', and on what timeline", "Whether AEF's sidecar-inbox and the W1 stack derive the same session id from hook input (claim/report keying; pre-switch dual claims under the A fallback)", "How a claim's checked origin is observable before the switch, since main status shows origin absent pre-switch and the standing rule demands a proven replacement before switching", "Whether the origin-reporting switch is reversible by one idempotent operator action (rollback from an all-claims-ignored state)", "Whether either W1 pair session is headless (this defines which session can ever show last_human_at)"]
---

## 1. Scoring

| option | D1 ×9 | D2 ×7 | D3 ×5 | D4 ×3 | F-AUT ×4 | total |
|---|---|---|---|---|---|---|
| A | +1 | 0 | −1 | +1 | +1 | **+11** |
| B | 0 | −1 | +1 | −1 | +1 | **−1** |
| C | +2 | +1 | +1 | 0 | +1 | **+34** |
| D | −1 | 0 | 0 | −1 | −1 | **−16** |

Where I differ: **A** (+11 vs agent's +32). The agent treats A's TermLink-only-ness as nearly free; I charge it for the permanent second claimant (chronic `ignored_recent` noise that destroys that diagnostic, silent fail-open starvation of a brand-new renewal path, and a settings-file edit that AEF re-vendoring can silently drop). Secondary difference: B's *end-state* converges with C's (one claimant, one prompt report, origin on the claim), so B's low score is about diff blast radius, not outcome — the agent's framing hides that.

## 2. Steelman and strawman

**A.** Steelman: zero cross-team dependency — TermLink owns the report format, the meter, and the hook, so it ships, tests, and fixes on its own schedule; it is purely additive (nothing working is switched off) and reusable for any future non-AEF stack. Strawman: "just add another hook" installs a permanent two-claimant regime in which every prompt files one ignored claim, renewal for the whole project hinges on one new fail-open path, and deployment requires editing a settings file another team may re-vendor.

**B.** Steelman: the guard is simply over-broad; adapter/2 was designed to be the origin reporter, so letting it do its job beside W1 hooks restores the intended architecture with one code path and no W1 special-casing. Strawman: "just narrow the guard" — `enabled()` gates BUSY reports, headless classification, and more across three scanned settings files, so "prompts only" must hold at every call site; one miss silently enables the exact double-reporting the guard was written to prevent.

**C.** Steelman: one claimant, no new hook, no settings edit, guard untouched; AEF already scans for W1 hooks (`w1_hook_files`) and already has the report code, and TermLink's sidecar already answers `origin_checked` (4.4) — C is three existing pieces wired together with the smallest diff that meets the criterion. Strawman: "one small AEF change" is a cross-stack dependency dressed as a patch — AEF hardcodes another stack's socket, every failure mode returns `(unknown, absent)` silently (which post-switch starves the lease), and TermLink's task now completes on another team's calendar and guard policy.

**D.** Steelman: honest and safe — nothing is switched off, no unproven path touches two production agents, and the limitation is recorded rather than papered over. Strawman: "close it as done" rewrites the acceptance criterion to dodge the work, leaves the flagship project permanently unable to use the tightening, and voids the only end-to-end proof of P-4.

## 3. Why (your recommended option)

C wins because it is the only option that keeps exactly one claimant per prompt *and* touches only already-existing surfaces: AEF's `sidecar-inbox` already runs on UserPromptSubmit in this exact project (4.5), AEF already detects W1 hooks (4.1), and the sidecar already returns a receiver-checked origin for a `prompt_submit` report (4.4, `checked_origin`). The guard invariant stays literally true (adapter/2 remains off — a report is added, not a second claim stack), there is no settings edit to lose, no chronic ignored-claim noise, and it actually satisfies the open criterion.

## 4. Why not (each other option)

**A — severity: high.** Operator switches reporting on → sidecar briefly down (or S1 unbound) → `checked_origin` returns `(unknown, absent)` and the W1 hook fails open, printing nothing → W1's claim ignored; AEF's origin-less claim is *also* ignored (FCW-53) → lease expires unrenewed → no main, mail undeliverable. Nobody notices: both hooks exit 0 on error and `ignored_recent` is chronically noisy by design under A. Independently, the hook line lives in a settings file AEF re-vendors and can silently drop.

**B — severity: high.** The "prompts only" narrowing must be enforced at every adapter/2 call site (BUSY, headless automated reporting, socket selection), because `enabled()` gates all of it across three scanned files; one miss turns on full adapter/2 beside W1's turn events → the meter sees two reporters for one session → contradictory prompt/turn state → wrong origins surfacing as unexplained lease decisions. The regression matrix is every adapter/2 behavior × W1 present.

**D — severity: medium.** Criterion rewritten and T-3694 closed → the tightening ships proven only on AEF-only projects → on this project automated wake-ups keep renewing main and fast takeover never engages → a headless worker holds the lease through a human prompt elsewhere and mail surfaces in a dead worker — precisely the failure P-4 exists to remove, now permanently accepted on the project meant to prove the design.

## 5. Alternatives the agent missed

**NEW-1 — consolidate the W1 pair onto the AEF stack.** Remove W1 hooks from the project's settings → the guard passes → adapter/2 reports origin natively; zero new code. Cost: loses W1-only behaviors (post-tool mail pull as additionalContext, detached finalizer, vendor-channel admin rows, session-end report), a behavior change on two production agents, operator approval, and AEF must confirm delivery parity (W1's delivery fails closed; AEF's path differs). Builder: operator + AEF config work, essentially no code. Meets the criterion: yes. Likely rejected because W1 hooks exist for capabilities AEF lacks — but it belongs on the table, especially as the alternative to a slipped AEF deadline.

**NEW-2 — claim-time enrichment in `termlink main claim` (TermLink-only).** The CLI, on an origin-less claim, asks the session's sidecar for `checked_origin_of(sid)` and stamps it on. Cost: the CLI gains a socket dependency and per-claim latency, and it risks overriding origins claimants deliberately omit. Builder: TermLink. Meets the criterion **alone: no** — the sidecar has no prompt record on the W1 pair today (nothing sends `prompt_submit`), so it only works combined with a reporter; its real value is as a hardening add-on to A or C that also covers future third-party claimants.

## 6. Enhancements and suggestions

**Tests to pin C** (the A fallback needs 1, 4, 5 too): (1) W1 present + socket set → exactly one `prompt_submit` sent, claim carries `origin_checked`/`basis`/`kind=prompt`; (2) **guard pin**: `adapter2.enabled()` stays `False` with W1 present — C must not leak any other adapter/2 behavior (this is B's failure mode); (3) **negative control**: socket absent/timeout/malformed reply → claim goes out origin-less, one hook-errors line, and under reporting it is `Ignored` (pin the starvation case explicitly); (4) meter: bound S1 + PTY input → human; headless/no-PTY → automated; **unbound S1 → unknown** (pin the dangerous default); (5) AEF and W1 derive the same session id from hook input; (6) the operator switch script is idempotent and its rollback restores built behaviour.

**Rollout order:** deploy C → verify `origin_checked=human` on the operator's interactive session from the sidecar reply/log (not `main status`, which shows absent pre-switch — see below) → negative-control one automated prompt → operator switches via the idempotent script (which should self-verify S1 binding and *refuse* to switch if the probe isn't human) → confirm `origin=human`, `last_human_at` advances, `ignored_recent` empty → alarm on any non-empty `ignored_recent` after the switch (it should be impossible on this project).

**Bundle issues.** (i) C vs B is thinner than the options imply: their end states converge (one claimant, one prompt report, origin on the claim); the honest distinction is diff scope and whether `enabled()`'s other gates stay closed — the test in (2) above is what makes the distinction real. (ii) Unbound S1 silently degrades *every* option to permanent `unknown`/ignored; the bundle flags it but gates nothing on it — make it a precondition inside the switch script. (iii) Under A there are two claims per prompt *pre-switch* too; if the stacks derive different sids, main flaps between two identifiers of one session — unaddressed. (iv) A headless W1 pair member never claims under C (4.2 returns before claim), so `last_human_at` for "the pair" is defined only by the interactive member; the criterion should name the session. (v) The criterion is observable only *after* the switch (status shows `absent` before), which collides with "nothing switched off without a proven replacement" — the bundle never defines the pre-switch proof; define it as the sidecar-level `origin_checked` evidence above. (vi) Reversibility of the OriginReporting record is asserted nowhere ("newest per harness wins" only implies it) — verify and document the rollback. Finally: since A scores +11, not +32, do not auto-slide to A after a week — escalate to AEF with the pinned-test list and put NEW-1 on the table as the real alternative.
