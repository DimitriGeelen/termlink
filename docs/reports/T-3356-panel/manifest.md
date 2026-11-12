# T-3356 review panel manifest (card review-panel 3.4)

**Task:** T-3356, arc-011 step 7. **Panel:** operator ruling A, 2026-11-11 (four vendors).
**Brief:** `docs/reports/T-3356-panel/brief.md`, version 1, sha256 `1f829a93e8956d003df5e61ddc4f93b994bedca9cfc448669603f99c3ea780be`
(the version every seat received, after the orchestrator's redaction commit `6440da616`).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 1 | 2026-11-11 | Skeleton (phase 1): material revision, seat table to fill | T-3356 |
| 2 | 2026-11-12 | Completed (phase 2): material hashes corrected to the redacted ARC v0.3.2 and PCb v0.2.1 that the seats actually read; seat identity, model, date and answer hashes filled; Codex recorded as unavailable | T-3356 |

## 1 Material revision sent to every seat

1.1 The brief names commit `c9aea3befd7ce44751ea7f9bf954da02a65c6845`. Before dispatch the orchestrator redacted two
internal addresses (commit `6440da616`: ARC drawing D-1 subnet; PCb V-T0-1 target replaced by an RFC 5737 documentation
address, digest and signature recomputed) and updated the brief's hashes. Every seat checked the hashes below and
reported "all match", so the material reviewed is the redacted revision.
1.2 Input hashes (sha256), as sent:

| Id | Path | sha256 |
|---|---|---|
| REQ v0.5.3 | `docs/design/interactive-agent-communication-01-requirements.md` | `f6ce42a2984bd18e049f1f4dda05e6eaab2cc3508396fe373060c6fc527193c9` |
| THR v0.3.7 | `docs/design/interactive-agent-communication-02-threat-model.md` | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |
| FP v0.3 | `docs/design/interactive-agent-communication-03-floor-and-phasing.md` | `e4de2e3e102ee59f6434db584e099e5bb180c6b33dfd823ae217e6ebb4e5a8c4` |
| ARC v0.3.2 (redaction only) | `docs/design/interactive-agent-communication-04-architecture.md` | `a5cc08eba8f72ddaabc09e31939db9072155066313e7bced4b437b4eda24927a` |
| TRC v0.3 | `docs/design/interactive-agent-communication-04a-traceability.md` | `c516a290997e0c75957c0be95af4d611e58155cd818d2fbae961c0f4ad6d4667` |
| EV v0.1 | `docs/design/interactive-agent-communication-05-evidence.md` | `29be80ae3ad8d32b71ef53caa00f9fdc6d03426c901215eaed85c019f889526e` |
| COV v0.1 | `docs/design/interactive-agent-communication-05a-coverage.md` | `16a4aa77b23cad191cf1b9153e60016a266ffc763c54674cb2d6e1f6341824fc` |
| PC v0.2 | `docs/design/interactive-agent-communication-06-pseudo-code.md` | `7d469813239e991304ddad04b35fe2dc713c1d0e2ed8142ee7c05b5ff4dbffcc` |
| PCa v0.2 | `docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md` | `1db1532a8d1d4c1002326a4b4e948be959b7a52c5bd33b8fba0ee2f405b69ebf` |
| PCb v0.2.1 (redaction only) | `docs/design/interactive-agent-communication-06b-pseudo-code-later-phases.md` | `8878cf38acb328b5aee89ef43be71685f4cbd044bb63006b6d3189e268c9f8c2` |

1.3 Version 1 of this manifest listed the pre-redaction hashes of ARC (`29c6d1f7…`) and PCb (`6b4e5296…`); those were
never sent. Corrected here.

## 2 Seats

| Seat | Vendor | Model and version (as the seat reported it) | Date | Material revision | Answer file | Answer sha256 |
|---|---|---|---|---|---|---|
| claude-code | Anthropic | Claude Opus 5.5 (`claude-opus-5-5`), in Claude Code | 2026-11-11 | brief v1, section 1.2 hashes, all match | `claude-code.md` | `e7dfbbe605b553ac13f42327d94acecc56ec43c9ddd1803f6fdb2aed61fdd2b9` |
| opencode | Z.ai | GLM `glm-5.3` (`zai-coding-plan/glm-5.3`), in opencode | 2026-11-11 | brief v1, section 1.2 hashes, all match | `opencode.md` | `20c66671208a561720f73045a50bc763e9b1119e34a65e71e78b8c3458281146` |
| antigravity, run 1 | Google | not stated in the surviving summary | 2026-11-11/12 | brief v1, all 10 hashes match (per its summary) | `antigravity-run1-summary.md` (summary only) | `f3b3e1476995cf1224bfcb009dc89fff1fd1d0b7a19f4329bea301031acef6ac` |
| antigravity, run 2 | Google | "Antigravity / Gemini 4 Argon (Medium)" | 2026-11-12 | brief v1, all 10 hashes match (read-only export without `.git`; commit not checkable, files bit-identical) | `antigravity.md` | `2700ccd1684d49c82a5407c41cfd757d6f0a6b712cd8e8aa64bdf7b0b8258afb` |
| codex | OpenAI | — | — | — | **no answer** (section 3.2) | — |

## 3 Notes

3.1 Answers are stored unedited in this directory; each seat saw the brief only, not the others' answers (card 3.3).
3.2 **Codex did not answer.** Run 1 wrote into its read-only export instead of answering in reply (voided, nothing
recovered); run 2 hit the subscription's usage limit; run 3 failed with "model at capacity" (2026-11-12 04:26Z). The
panel closes with three of the four vendors. The step-7 output records this as a gap in vendor diversity (07-review
section 3.3), not as a pass.
3.3 **Antigravity run 1** also wrote its full answer into its read-only export; the file was lost with the export. Only
its closing summary survives (`antigravity-run1-summary.md`). Run 2 was dispatched with "answer in the reply" and is
complete. Both runs are the same seat; the synthesis counts run-1 findings that run 2 does not repeat.
3.4 **opencode** ran in plan mode: its stored answer is the reply text, which carries a short preamble and the answer
inside one fenced block, followed by a request to write the file. The answer was stored as the reply, unedited.
3.5 **claude-code** answer was in the working tree, not committed, when phase 2 started; it is committed unchanged with
the phase-2 output (sha256 above, same bytes).
3.6 The brief text carries no secrets, credentials, internal addresses or hostnames. The two internal-network
references flagged in version 1 were redacted by the orchestrator before dispatch (1.1).
