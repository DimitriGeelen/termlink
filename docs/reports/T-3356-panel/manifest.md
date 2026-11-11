# T-3356 review panel manifest (card review-panel 3.4)

**Task:** T-3356, arc-011 step 7. **Panel:** operator ruling A, 2026-11-11 (four vendors).
**Brief:** `docs/reports/T-3356-panel/brief.md` (sha256 in the hand-back and in the commit that sends it).

## 1 Material revision sent to every seat

1.1 Git HEAD when the brief was written: `c9aea3befd7ce44751ea7f9bf954da02a65c6845`.
1.2 Input hashes (sha256):

| Id | Path | sha256 |
|---|---|---|
| REQ v0.5.3 | `docs/design/interactive-agent-communication-01-requirements.md` | `f6ce42a2984bd18e049f1f4dda05e6eaab2cc3508396fe373060c6fc527193c9` |
| THR v0.3.7 | `docs/design/interactive-agent-communication-02-threat-model.md` | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |
| FP v0.3 | `docs/design/interactive-agent-communication-03-floor-and-phasing.md` | `e4de2e3e102ee59f6434db584e099e5bb180c6b33dfd823ae217e6ebb4e5a8c4` |
| ARC v0.3.1 | `docs/design/interactive-agent-communication-04-architecture.md` | `29c6d1f7de9f18898cab105651f17977ebf97da30f32d16501819bad217dd69c` |
| TRC v0.3 | `docs/design/interactive-agent-communication-04a-traceability.md` | `c516a290997e0c75957c0be95af4d611e58155cd818d2fbae961c0f4ad6d4667` |
| EV v0.1 | `docs/design/interactive-agent-communication-05-evidence.md` | `29be80ae3ad8d32b71ef53caa00f9fdc6d03426c901215eaed85c019f889526e` |
| COV v0.1 | `docs/design/interactive-agent-communication-05a-coverage.md` | `16a4aa77b23cad191cf1b9153e60016a266ffc763c54674cb2d6e1f6341824fc` |
| PC v0.2 | `docs/design/interactive-agent-communication-06-pseudo-code.md` | `7d469813239e991304ddad04b35fe2dc713c1d0e2ed8142ee7c05b5ff4dbffcc` |
| PCa v0.2 | `docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md` | `1db1532a8d1d4c1002326a4b4e948be959b7a52c5bd33b8fba0ee2f405b69ebf` |
| PCb v0.2 | `docs/design/interactive-agent-communication-06b-pseudo-code-later-phases.md` | `6b4e5296fe7673360ff054919c15a851fdb5bdc8af366f6222de79967e092ddf` |

## 2 Seats

| Seat | Vendor | Model (to fill) | Date (to fill) | Material revision | Answer |
|---|---|---|---|---|---|
| claude-code | Anthropic | | | HEAD `c9aea3bef` + section 1.2 hashes | `claude-code.md` |
| codex | OpenAI | | | HEAD `c9aea3bef` + section 1.2 hashes | `codex.md` |
| opencode | Z.ai (GLM) | | | HEAD `c9aea3bef` + section 1.2 hashes | `opencode.md` |
| antigravity | Google | | | HEAD `c9aea3bef` + section 1.2 hashes | `antigravity.md` |

## 3 Notes

3.1 Answers are stored unedited in this directory; each seat sees the brief only, not the others' answers (card 3.3).
3.2 The brief text carries no secrets, credentials, internal addresses or hostnames. The repository documents the
seats read contain two internal-network references: ARC line 308 (an operator LAN subnet in drawing D-1) and PCb
line 45 (a hub address in a worked example). Flagged to the orchestrator; not edited here (card: write only your
own output).
