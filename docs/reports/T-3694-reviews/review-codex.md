---
reviewer: Codex
task: T-3694
question: W1 pair prompt-origin route
recommended_option: C
agrees_with_agent: partly
confidence: medium
scores: {A: 16, B: 12, C: 28, D: -25}
top_risk: "C attributes PTY evidence or a checked reply to the wrong prompt, falsely marks an automated prompt human, and lets it renew or take main."
blocking_unknowns: ["Whether S1 is bound to each W1 session and reliably distinguishes human input from automated input.", "Whether either W1 session is headless or otherwise ineligible to claim, and what the status criterion requires for that session.", "Whether checked replies are correlated to the submitted prompt and session under retries and concurrent reports.", "Whether AEF can enable prompt reporting alone without enabling duplicate lifecycle reports, and which configured hooks currently claim main."]
---

## 1. Scoring

Scores reflect the routes as described, before the blocking unknowns are resolved.

| Option | D1 Antifragility ×9 | D2 Reliability ×7 | D3 Usability ×5 | D4 Portability ×3 | F-AUTONOMY ×4 | Total |
|---|---:|---:|---:|---:|---:|---:|
| A | +1 | 0 | 0 | +1 | +1 | **16** |
| B | 0 | 0 | +1 | +1 | +1 | **12** |
| C | +1 | +1 | +1 | +1 | +1 | **28** |
| D | −1 | −1 | −1 | 0 | −1 | **−25** |

I differ from the agent’s totals because A adds claim-order hazards, B’s explicitly prompt-only scope deserves more credit, and C still depends on unverified evidence plumbing; the bundle omits the agent’s individual driver scores.

## 2. Steelman and strawman

**A.** Its strongest case is ownership: TermLink can implement and test its own reporting path without waiting for AEF, using an existing checked-origin helper. Its weakest case is that the second claimant is harmless because absent-origin claims are ignored: that protection applies only after activation, while installation and rollback still encounter legacy claim behavior.

**B.** Its strongest case is reusing the existing adapter’s prompt classification and claim integration while retaining W1’s lifecycle hooks. Its weakest case is that narrowing one guard is sufficient: the adapter also reports BUSY, and the bundle does not establish which effects remain enabled or overlap with W1.

**C.** Its strongest case is placing classification immediately before the existing eligible claimant, preserving one claim path and the existing headless exclusion. Its weakest case is calling this a small, distinct alternative to B: if B enables exactly the same prompt-only exchange with the W1 socket, they are the same implementation.

**D.** Its strongest case is preserving known working delivery while declining an unproven migration. Its weakest case is presenting administrative closure as completion: changing the criterion leaves this project without the tightening it was intended to demonstrate.

## 3. Why (your recommended option)

**C best preserves the existing ownership of claiming while adding the missing evidence.** AEF already decides whether a session may claim; it should obtain the checked result and attach it to that same claim. TermLink should remain responsible for classification and its socket contract.

Specify C as a narrowly enabled adapter capability, reusing existing reporting code. Do not create a second independent implementation merely to distinguish it from B.

Agreement is conditional: the one-week fallback has no technical justification. AEF delay alone does not make A safe; A needs its own transition and eligibility proof.

## 4. Why not (each other option)

- **A — high:** Install the new hook before activation → AEF’s origin-less claim still counts → an automated prompt can affect the lease despite the new hook’s checked result. After activation that particular claim is ignored, but duplicated claim paths and ordering remain operational liabilities.
- **B — medium:** Narrow the shared guard without isolating its effects → prompt reporting also enables overlapping state updates → the meter receives duplicated or inconsistently ordered transitions. If B explicitly excludes those effects and uses C’s exchange, this rejection disappears: B is C.
- **D — high:** Keep legacy behavior → automated activity continues to count toward lease decisions → the project retains the failure mode the tightening addresses. Closing the task then obscures the missing production proof.

## 5. Alternatives the agent missed

**NEW-1 — Replace AEF’s prompt claimant with a complete W1 prompt path.** TermLink implements reporting, eligibility checks, claiming, and equivalent prompt-time mail handling; AEF supplies a supported way to disable its corresponding path. Cost is higher than A because replacement behavior and migration must be proven, with documented operator approval. It can meet the criterion for eligible interactive W1 sessions and removes A’s permanent dual-claim arrangement.

**NEW-2 — Carry checked prompt evidence directly from sidecar to hub.** TermLink implements a correlated evidence record or receipt that the hub consumes when evaluating the existing claim; AEF may need to carry a prompt identifier. This costs a protocol and storage change but reduces dependence on hooks faithfully forwarding classification. It can meet the criterion, provided claims and evidence have defined ordering, retry, and session-binding semantics.

## 6. Enhancements and suggestions

- **Define C precisely:** one prompt report, checked reply from the session’s own socket, one eligibility-gated claim. Keep lifecycle reporting under W1. Preserve AEF’s headless early return.
- **Pin classification and claiming:** test a real human prompt, an automated wake-up, stale-holder human takeover, and a headless prompt. As a negative control, unbind S1 and require `unknown`, no human timestamp advancement, and no renewal after activation.
- **Pin failures:** socket timeout, malformed reply, wrong session, duplicate report, and concurrent prompts must never manufacture `human`. Verify that the reply identifies the submitted prompt; the excerpt’s `last_prompt(session_id)` lookup alone does not prove this.
- **Roll out before switching:** implement and test the prompt-only path; deploy with reporting enforcement off; verify S1 binding and checked classifications on both W1 sessions; then obtain documented operator approval for activation. Package each operator action as an idempotent, self-verifying script.
- **Verify the actual criterion:** observe a human claim producing a human origin and non-null `last_human_at_ms`, then confirm automated activity leaves that timestamp unchanged. Exercise each eligible W1 session as holder, since project main status is not evidence about both sessions simultaneously.
- **Prove recovery:** test rollback before activation. Removing the reporter while leaving enforcement on would strand origin-less claims in the ignored path.

The bundle overstates “receiver-checked” if it means the **hub** independently verifies origin: the excerpts show the sidecar classifies it and the hook forwards it. They do not establish hub-side binding to that checked result.

An unbound S1 is a blocker for **every reporting route**, not an argument for A over C. A headless session reports before AEF’s early return but does not claim; requiring its human timestamp to become populated would conflict with the shown behavior. Finally, C and a correctly scoped B should share one implementation and one test suite.
