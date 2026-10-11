# T-3352 step 3 review round 1: comparison and disposition (orchestrator, 2026-10-11)

Reviews stored unedited: `codex.md` (Codex, read-only, rc 0) and `glm.md` (GLM-5.3 via opencode, rc 0). Both verdicts:
**fit after listed fixes.** Orchestrator spot-checked the consequential findings in the text (lines cited) before
disposing them.

## 1 Where they agree
1. PH-3's exit test needs the C-10 policy route, whose subject (lease successor, hand-over) only exists in PH-4
   (Codex 3a, GLM 4a). Verified at 6.2.4.c (2).

## 2 Disposition (E = editorial fix by the role session; O = becomes an operator question)

| # | Finding | Source | Verified | Disposition |
|---|---|---|---|---|
| 1 | `may_remove` value `anyone_in_its_conversations` marked ruled; OQ-12 ruled "anyone in conversations it started" | Codex 1a | line 742 | E: restore the ruled value |
| 2 | C-17 human release of custody mail outside C-10 conditions contradicts B′ (R-80.a (5)) | Codex 1b | line 721 | E: remove C-17 as a route; break-glass stays R-80.a (7). Note in OP-3 that wider release would need a ruling change |
| 3 | C-9c renewal silently widened to every policy record; budget reset unaddressed | Codex 1c | 10.3.2.a | E: constrain to grants as defined; state budget behaviour on renewal as [P] |
| 4 | C-15 batch digest relaxes "every account creation is a Tier 0 act" (R-82.a (3)) | GLM 3b | line 719 | E: one event per creation per the ruling; mention in OP-1 that a batch form would need a ruling change |
| 5 | Digest includes itself; profile example has empty approvals | Codex 2a | 10.3.2.a, 10.3.9 | E |
| 6 | Vocabulary lacks verbs for C-2a, C-6a, C-9c; no event-rule representation; epochs per scope and same-level conflict resolution undefined | Codex 2b, 2c | 10.3.2–10.3.7 | E: complete the format (055 waits for it) |
| 7 | PH-3 exit test uses C-10 | both | 6.2.4.c (2) | E: test C-2a/C-6a in PH-3; C-10 by fixture until PH-4 |
| 8 | Account pilot (PH-2) precedes its Tier 0 machinery (PH-3) | Codex 3b | 6.2.3.a (7) | E: move the pilot to PH-3 (no new risk acceptance) |
| 9 | Existing subjects deferred without risk acceptance: SI-7 framing of other rails until PH-4; SI-15 reconciliation after PH-1 storage | Codex 3c | 3.3 | E where an early minimal control is cheap; otherwise an explicit RA plus an OP question (O) |
| 10 | BV-5 (delivery at next tool call) vs CX-2 (normal mail waits for turn end) | Codex 3d | 5.5.5, 12.3 | O: new OP question on CX-2 delivery semantics; BV aligned to the recommendation, marked pending |
| 11 | Receive-side ownership fallback (6.6) "proposed, not ruled" but no OP | Codex 5a | 6.6 | O: new OP question |
| 12 | PH-0 rollback turns off the mandated re-pin record (R-72.a (6), OQ-8) | Codex 4a | line 428 | E: rollback keeps the recorder |
| 13 | PH-6 rollback "per item"; D-1 omits it | Codex 4b | 6.2.7.b | E: per-item rollback procedures; update D-1 and its text equivalent; re-render |
| 14 | RA-4 ends on provider OR key; distinct protections | Codex 4c | 11.3.4.d | E: separate completion conditions |
| 15 | Floor binding rules 3.1.b–3.1.f not put to the operator | GLM 3c | OP-10 | O-light: extend OP-10 A to name them |
| 16 | FX-5 probe needs a PH-0 baseline of plaintext content on the hub | GLM 4b | 6.2.1.a (2) | E |
| 17 | "R-7 independence arrives last of the P1 items" overstates | GLM 4c | 4.4.c, 11.2 | E |

## 3 Round plan
One fix pass by the role session, verified by the orchestrator (diff, renders, grep of the fixed ids), then one short
Codex confirmation. No third full round (lesson 5.16).
