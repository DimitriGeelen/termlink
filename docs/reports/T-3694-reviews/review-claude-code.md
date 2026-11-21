---
reviewer: Claude Fable 5.2
task: T-3694
question: W1 pair prompt-origin route
recommended_option: C
agrees_with_agent: partly
confidence: medium
scores: {A: 16, B: 1, C: 20, D: -14}
top_risk: "After the switch, real human prompts come back `unknown` (S1 unbound, session-id mismatch, sidecar down or timeout), every claim is ignored, the lease cannot be renewed or taken, and the project's mail has no main."
blocking_unknowns: ["Whether the meter's PTY-counter source (S1) is bound, per session, for the W1 pair's sessions and for the operator's interactive session in that project", "Whether the session id AEF's hook would put in prompt_submit is the same id the W1 session-start hook registered in the meter (otherwise last_prompt() is None and no origin comes back)", "Whether the production W1 sidecar applies a prompt_submit that arrives without adapter/2's BUSY, and whether applying it changes turn or delivery state", "Whether TERMLINK_SIDECAR_SOCKET is present in the AEF hook's environment, and whether there is one sidecar per session or per agent identity (keystroke attribution across sessions)", "Whether status records origin and last_human_at before the switch, so the criterion can be met without switching", "What the lease does when every claim is ignored (expiry, mail routing) and whether origin reporting can be switched back off by script", "Whether AEF flags the W1 pair's sessions as headless (they would then report but never claim)", "Whether AEF will take the change, and when"]
---

## 1. Scoring

| Driver (weight) | A | B | C | D |
|---|---|---|---|---|
| D1 Antifragility (9) | 0 | 0 | +1 | -1 |
| D2 Reliability (7) | 0 | 0 | +1 | 0 |
| D3 Usability (5) | +1 | +1 | +1 | -1 |
| D4 Portability (3) | +1 | 0 | +1 | 0 |
| F-AUTONOMY (4) | +2 | -1 | -1 | 0 |
| **Total** | **16** | **1** | **20** | **-14** |

I differ from the agent in three places:
- **B vs C:** the agent's 39-point gap is not supportable, because C is B with the BUSY report dropped and the socket pinned; I put them 19 apart, and most of that is B's undefined scope.
- **A:** I score it lower (16 vs 32) because as framed it adds a second claimant that skips AEF's headless and eligibility gates.
- **All positive scores:** they are conditional on S1 being bound; if it is not, A, B and C all return `unknown` and are worth no more than D.

## 2. Steelman and strawman

**A**
- **Steelman:** TermLink owns the W1 stack, the sidecar and the lease, so it can close its own task without waiting on another team. `checked_origin()` already exists in `hook.rs`, and the W1 stack is the natural home of a prompt event it is visibly missing.
- **Strawman:** "It's TermLink code only, so it's safe." It is not only code: it needs a settings-file change, and it puts a second claim on every prompt. The new claimant has none of AEF's `headless` or `may_claim` checks, and same-event hooks have no ordering guarantee between the two claims.

**B**
- **Steelman:** adapter/2 is the component designed and tested to send this report and carry the origin. Narrowing its guard reuses one code path for all projects instead of creating a W1-only branch.
- **Strawman:** "Just relax the guard." The guard exists to stop double reporting, and `hook_reports("prompt")` sends BUSY as well as prompt_submit. Relaxing it puts two turn-state reporters on one meter, which is exactly what the guard forbids.

**C**
- **Steelman:** one claimant stays one claimant. The claim keeps AEF's existing headless and eligibility gates and gains only a receiver-checked origin, the meter gets the one event it lacks, and nothing working is switched off.
- **Strawman:** "It's small, so it's low risk." It is small in lines, but it lives in code vendored into every project and owned by another team. It adds a synchronous socket round trip to every prompt, and its correctness depends on unverified facts (S1, session id, socket in the environment).

**D**
- **Steelman:** today's behaviour works. If the W1 pair's sessions are effectively headless, or S1 cannot be bound for them, origin reporting can never say `human` there, and admitting that is cheaper than building a route that returns `unknown`.
- **Strawman:** "Nothing changes, so nothing breaks." The switch command stays available. Running it on this project locks everyone out of main, and the project meant to prove the design end to end never does.

## 3. Why (your recommended option)

- **One claim per prompt.** Status, `ignored_recent` and the lease generation stay readable, and there is no race between an origin-less and an origin-carrying claim for the same prompt.
- **AEF's gates stay in force.** In 4.2 the report runs before the `headless` return and the claim runs after `may_claim`. C inherits both; A would have to re-implement them or go without.
- **No double turn reporting.** The W1 hooks have no prompt event, so one prompt_submit from AEF fills a gap instead of duplicating BUSY, which is B's problem.
- **Safe before the switch.** A claim carrying an origin never switches a harness (test in 4.3), so C can run and be observed for days at built behaviour.

The cost is the cross-team dependency, which is why my lead over A is only 4 points. I agree with A as the fallback only in the amended form of NEW-2, not as framed.

## 4. Why not (each other option)

**A — high (as framed).**
1. Before the switch, a headless worker or wake-up prompt fires the new W1 hook.
2. The hook claims with `--origin automated`, but the harness is not switched, so the claim is decided as built.
3. A session AEF deliberately excludes takes main from the operator, and mail goes to a worker.

Further weaknesses (medium):
- After the switch, AEF's own claim returns Ignored on every prompt. `ignored_recent` fills with one entry per prompt and hides real ignores.
- Whether AEF's `sidecar-inbox` still surfaces mail when its own claim is ignored is unknown.
- If the W1 hook claims under a different session id than AEF's `my_sid`, one session appears as two holders.

**B — medium.**
1. AEF narrows the guard to "prompts only".
2. `hook_reports("prompt")` still sends BUSY alongside prompt_submit to the meter.
3. The W1 stop hook sends ready. The meter now has two writers of turn state, so a lost or reordered report leaves the session stuck BUSY, and delivery (which fails closed) stalls.

If B is specified to send prompt_submit only, it is C.

**D — medium.**
1. The criterion is rewritten and T-3694 closes.
2. Months later, someone runs `termlink main origin-reporting --harness claude-code` on this project.
3. Every claim is origin-less and ignored, the holder cannot renew, and no session can take main.

The design also never gets its end-to-end proof.

## 5. Alternatives the agent missed

**NEW-1 — the claim command fetches the origin itself.**
- **What:** `termlink main claim` with no `--origin`, and a W1 sidecar socket in the environment, asks the sidecar for the checked origin before claiming.
- **Cost:** small, and no AEF change. It keeps one claimant and AEF's gates.
- **Who:** TermLink.
- **Catches:** it only works if the claim command can build a valid prompt_submit without the hook input, and can know the claim is a prompt (AEF passes `--kind` only with an origin). Both are unverified.
- **Criterion:** met, if those two facts hold. Worth an hour's check, because it removes the cross-team wait.

**NEW-2 — A, gated.**
- **What:** the W1 prompt hook reports always, but claims only when the checked origin is `human`. It stands down if AEF's claim already carries an origin, so that C landing later does not create two claimants.
- **Cost:** medium, plus the settings-file change shipped as a script action.
- **Who:** TermLink.
- **Criterion:** met. It still leaves the per-prompt ignored AEF claim after the switch. This is the fallback I would accept.

**NEW-3 — probe first.**
- **What:** a read-only `termlink sidecar` diagnostic that reports, per session, whether S1 is bound and what the last prompts were classified as, with no claim.
- **Cost:** tiny.
- **Who:** TermLink.
- **Criterion:** not met by itself. It decides whether A, B or C can ever return `human` here, so it should run before any of them is built.

**NEW-4 — a lockout guard on the switch.**
- **What:** the hub or the switch script refuses to turn origin reporting on unless the harness has produced at least one checked-`human` claim recently, and reverts if no human claim renews within a set time after switching.
- **Cost:** small.
- **Who:** TermLink.
- **Criterion:** not met by itself. It removes the top risk for every option, including D.

## 6. Enhancements and suggestions

**Tests that pin C**
- **Positive:** W1 hooks present and a stub sidecar replying `human`. Expect exactly one `termlink main claim`, with `--origin human --basis … --kind prompt`.
- **Negative control 1:** socket dead or slow. The turn fails open within a fixed budget (about 250 ms), and the claim carries an explicit `--origin unknown` with a basis naming the failure, so `ignored_recent` explains itself.
- **Negative control 2:** a human keystroke in session X, then an automated prompt in session Y on the same agent identity. Y must come back `automated`.
- **Negative control 3:** a project without W1 hooks. The claim arguments are byte-identical to today's.
- **Sidecar:** a prompt_submit without BUSY is applied, turn and delivery state is unchanged, and a duplicate prompt_submit for the same prompt is idempotent.
- **Hub:** in a switched harness, `human` renews, and `automated`, `unknown` and absent do not.

**Rollout order**
1. Run NEW-3 and settle the S1 and session-id unknowns.
2. Send the AEF request, with the change behind a config flag that defaults off and is on only in this project.
3. Observe before the switch: every human prompt classified `human`, zero `unknown`, and status showing origin and `last_human_at`.
4. Bring the switch to the operator as its own script action with NEW-4's self-verify and auto-revert.
5. Close the criterion.

**To verify in the field**
- Prompt-hook latency with the added round trip.
- That headless sessions report `automated` and never claim.
- That the lease generation does not churn.
- What AEF's inbox does when its claim is ignored.

**Wrong or inconsistent in the bundle**
- **C vs B:** the scores treat them as far apart, yet the bundle's own note says C is "adapter/2's prompt report only, into the W1 socket". The only real differences are the dropped BUSY and which socket is used, and the scoring should rest on those.
- **Unbound S1:** A, B and C all yield `unknown`, so after the switch the outcome is the same lockout as today. The scores silently assume S1 is bound.
- **A is not "TermLink code only":** it needs a settings change and creates two claimants per prompt with no defined order.
- **Headless:** the bundle does not say whether the W1 pair's "production agents" are headless. If they are, the criterion can only ever be shown on the operator's interactive session.
- **The guard's reach:** `w1_hook_files()` also scans the user's settings file, so a W1 hook there turns adapter/2 off for every project on the machine. "Other projects are unaffected" holds only if that file is clean.
- **Fallback overlap:** "C, then A after a week" can end with both shipped; without NEW-2's stand-down rule that is two claimants and two prompt_submits per prompt.
