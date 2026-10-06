# Design role chain: adopted from ring20 (T-3339)

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | Documentation layer adopted from ring20 T-2222 v0.2 (operator ruling C) | T-3339 |
| 0.2 | 2026-10-06 | ring20 practice v0.3: lessons 5.15–5.18 adopted (operator ruling A); 5.15–5.17 in `/decision-brief`, 5.18 applied (subscription-seat reviews without asking, paid routes only with the operator's go, no secrets in briefs) | T-3344 |

## 1 Provenance

1.1 Source: ring20-manager's answer to our request, `upstream-ring20/T-2222-inception-protocol.md` (v0.2,
sha256 a275af88…41d3aa, received on inbox offset 525), and its repository `proxmox-ring20-management` on OneDev
at commit `eaccc22` (contains `9c5aa55a2`, the commit the answer cites).
1.2 Copied verbatim, byte-identical to that commit: `cards/` (`_common.md` + 8 role cards), `adapters/aef.md`,
`schemas/role-handback-1.schema.json`. Kept verbatim as provenance only (never part of a role's prompt):
`upstream-ring20/` (ring20's profile, agent rules, its arc-008 chain file, its roles plan, the protocol
document). Hashes: `upstream-ring20/SHA256SUMS` (`sha256sum -c` from this directory).
1.3 Ours: `profiles/termlink.md`, this README, `../interactive-agent-communication-role-chain.yaml`, and
`tests/test-role-cards-portable.py` (ring20's test with only the cards path changed).
1.4 Ring20's cards descend from `1000-AI-Roles @ d7e6d8a35b90`; ring20 keeps those originals under its
`roles/upstream/`. We did not copy them.

## 2 What is adopted, and what is not yet

2.1 Adopted now (operator ruling C, 2026-10-04):
2.1.a Layer A, unchanged: AEF's inception (one question, research artifact first with a Dialogue Log, IW-n with
dispositions, GO/NO-GO by the operator only, build tasks after GO).
2.1.b Layer B, the documents: the 8 roles and their cards, the common rules, the hand-back record, the output
skeleton (section 4), the two step-task criteria (section 5), required drawings, and ring20's 18 lessons
(`upstream-ring20/T-2222-inception-protocol.md` section 5 for 5.1–5.14; v0.3's 5.15–5.18 in
`upstream-ring20/T-2222-v0.3-lessons-5.15-5.18.md`, extracted from ring20's page until the canonical v0.3 file arrives).
2.2 Not adopted yet, because vendored AEF 1.6.29 lacks `fw reviewer judge` and `fw reviewer verdict apply`:
the driver `role-chain.py`, `role-handback-check.py`, `design-render-check.py`, and the rung-5 review panel. They
come with the re-vendor to bleeding-edge AEF (a separate operator decision). Until then the orchestrator keeps
the step order and the approval ticks by hand (section 3), and does **not** build a stand-in review policy
(ring20 lesson 5.13).

## 3 Running a step by hand, until the driver arrives

3.1 One AEF task per step, in the chain's order. A step starts only when the step before it is approved.
3.2 The role works in its own session (`fw termlink dispatch`), never as an in-process sub-agent (ring20 2.2.6:
in-process sub-agents share the producer's context, so their review is not independent).
3.3 What the role session receives, in this order (ring20 1.8):
3.3.a project rules: the project-specific sections of `CLAUDE.md` (profile P1.1);
3.3.b `cards/_common.md`;
3.3.c its role card from `cards/`;
3.3.d `adapters/aef.md` (ring20's AEF adapter; where it names ring20 paths or commands this project lacks, the
profile's P2 and P4.5 govern);
3.3.e `profiles/termlink.md`;
3.3.f the step facts: inputs with their sha256, the output path, the chain file.
3.4 The role commits after every unit and hands back a `role-handback/1` record
(`schemas/role-handback-1.schema.json`) next to its output.
3.5 Independent review: until the panel exists, the orchestrator runs the external consultation (profile P4.4)
on the output and stores the answers unedited. This is evidence for the operator, not a verdict; no agent ticks
the review criterion.
3.6 Where the step `decides`, the operator reads the output and approves it; the agent records the approval in
the step task. Completion is never approval.

## 4 Output document skeleton (ring20 3.3)

Front matter (title, task, arc, role, status) → review link → inputs of record → `## 0 Version history`
(version, date, change, task) → numbered sections with ids (R-n, TH-n, SI-n, RR-n, CR-n, E-n, D-n) → residual
risks for the operator → change requests to earlier steps. Plain Markdown, everything numbered (1 / 1.1 /
1.1.a), every item traced to its source, RFC 2119 words for normative text, required drawings D-n each with a
numbered text equivalent.

## 5 The two acceptance criteria of every step task (ring20 3.5, lessons 5.3 and 5.4)

```
### Human
- [ ] [REVIEW] Independent review: step N output meets its role card's completion conditions and is fit as input for the next role
  (Steps: the orchestrator runs the review; a reviewer never runs review commands itself — lesson 5.2)
- [ ] [REVIEW] Step N output approved
  **Sovereignty:** operator decision only
```

## 6 Lessons we already share with ring20

6.1 Chat answers die at compaction: every answer to an operator question becomes a file (5.1). This project
learned the same over seven months of lost design rounds (arc-011).
6.2 Shared git index: commit with `-- <paths>` after checking the index (5.8; here T-3090/T-3231).
6.3 Cross-hub mail lands where nobody looks, and one notice per project, not per session (5.11, 5.12): both
feed the hub-routing review, `docs/reports/T-3335-routing-consult/`.
