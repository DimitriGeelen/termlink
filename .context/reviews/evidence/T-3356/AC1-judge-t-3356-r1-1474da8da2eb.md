# T-3356 Human AC#1: independent review evidence

**Reviewer:** reviewer-judge-t-3356-r1-1474da8da2eb (rung-1-same-vendor-independent). **Revision:** d6b13e275c90aaca84187de04298c3c12026f7df
**Under review:** `docs/design/interactive-agent-communication-07-review.md` (v0.1), `docs/reports/T-3356-panel/*`,
`docs/design/interactive-agent-communication/role-handback-step-07.json`. Card: `docs/design/roles/cards/review-panel.md`.

## Integrity
- I recomputed the sha256 of every file in the hand-back (`output` plus 23 `inputs`) from `git show d6b13e275:<path>`. All 24 match.
- The hand-back is valid JSON and validates against `docs/design/roles/schemas/role-handback-1.schema.json` (python jsonschema).
- The redaction commit 6440da616 changes only the ARC D-1 subnet label and the PCb V-T0-1 target, with the digest and signature recomputed. Both are marked as redaction-only version rows. A grep of the brief and the ten sent design files for RFC 1918 addresses, private keys, `ghp_`, `sk-` and `.local` hosts finds nothing (card 3.1).

## Card 6.1: manifest complete, answers stored unedited
- manifest.md v2 records, for each seat, the vendor, the model as the seat reported it, the date, the material revision (hashes) and the answer hash. Codex is recorded as unavailable three times, with a reason for each.
- `git log` per answer file: claude-code.md, opencode.md, antigravity.md and antigravity-run1-summary.md were each committed once and never modified. That is consistent with "stored unedited".
- Caveat, disclosed in manifest 3.3: Antigravity run 1's full answer was lost in the seat's export, and only its summary survives. What exists is stored unedited. The loss is recorded, not hidden.
- Operator go for outside vendors: T-3356 Decisions 2026-11-11, ruling A (card 3.2).

## Card 6.2: every finding dispositioned, every accepted finding has a CR
- Raw ids in the answers: CLAUDE-CODE-1..25, OPENCODE-1..12, ANTIGRAVITY-1..18 (run 2) and ANTIGRAVITY-1..16 (run 1 summary), 71 in total.
- I parsed the section 4 table with a script. There are 51 rows F-1..F-51, and each of the 71 raw ids appears in the "Raised by" column; none is missing.
- Dispositions: 42 A, 6 R, 3 D. Severities: 1 critical, 9 high, 22 medium, 13 low, 6 rejected. Both sets of numbers match 4.2 and the hand-back summary.
- Every A row cites at least one CR. Each CR-x.y cited resolves to section 6.x.y, which names the same F-n; the only exception is CR-2.1, which is the single item in 6.2 and names F-50.
- I spot-checked the five SEC rejections against the text:
  - F-38: PC line 455 has the `<= 120 s` window.
  - F-37: PCa line 234 has `require verify(pin, h)`.
  - F-39: PC line 915 has "`last_human` unknown | Held".
  - F-7: PC line 973 has the lapse-like vacancy row.
  - F-36: PCb AL-42 step 1 refuses anything but the root-only admin path (FCW-157).
- F-41: PCa line 749 has the matcher `SendMessage|Agent|Task`. AG-9 is class "ruling", so treating it as non-SEC is consistent with the 4.1 legend.
- 3.4.a: none of the 11 identifiers, nor `VENDOR_TOOLS = {`, occurs in 01..06b; all of them occur in antigravity.md.
- Built-code claims: `meter.rs:565-568` ("S2 (a): every valid report") and `meter.rs:916` (`b == "pty-input"`) are as described.
- Security rejections and deferrals are routed to the operator (4.3, 8.3, 8.4). They are also listed in the hand-back `decisions_needed` and in Human AC#2 (card 5).

## Card 6.3: no step final while it has an open CR
- 5.1.g and the 9 table state that steps 1..6 are reopened and that step 8 must not start until the CRs are applied or the operator rules otherwise. B-27's CRs go to T-3687 (6.7).

## Card 6.4: drawing D-1
- D-1 is present with its id, a caption and a textual equivalent (5.1.a-g). It shows the disposition flow from each seat, through accepted, rejected or deferred, to the CRs for each step, as card 4.D.1 asks.
- I re-rendered it independently: I extracted the mermaid block and ran `mmdc` with chromium --no-sandbox. Exit code 0, a 41 KB SVG, 20 nodes (all expected nodes), no syntax or parse error. The producer's r1.out says "blocks 1 / OK / exit 0".

## Fit as input for step 8
- The planner's inputs (chain yaml) are ARC and 07-review. 07-review gives step 8 a complete, numbered CR set per step and per running build item, the open decisions (section 8, OQ-7.1) and the explicit gate in 9/6.3. It is fit as the input that tells step 8 what is open and blocks premature planning.

## Non-blocking notes
- Editorial: the version-history row says "change requests CR-1..CR-7 per step". The document actually has CR-1..CR-6 plus CR-B27. Fix it in the next revision.
- RQ-1: the panel has three of the four ruled vendors. This is disclosed, and the operator decides it under AC#2.

## Verdict: green
