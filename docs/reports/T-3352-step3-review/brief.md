You are an independent reviewer of ONE design document. Read only; modify nothing. Answer in English, under ~1300 words, numbered headings, hierarchical labels (1, 1a), never plain bullets. Cite section numbers / ids for every finding.

Document: docs/design/interactive-agent-communication-03-floor-and-phasing.md (arc-011 step 3, "Security floor and phasing", DRAFT v0.1).
Its role card: docs/design/roles/cards/mvp-specialist.md (completion conditions 6.1-6.4) and docs/design/roles/cards/_common.md.
Inputs of record (operator-approved): docs/design/interactive-agent-communication-01-requirements.md (v0.5.2) and docs/design/interactive-agent-communication-02-threat-model.md (v0.3.7; rulings register 22.18). The operator's rulings are binding; their full records are in the "## Decisions" sections of .tasks/completed/T-3351-arc-011-role-chain-step-2-threat-modeler.md and .tasks/completed/T-3388-arc-011-step-1-reopen-requirements-v05-f.md.

Check, and report only real problems:
1. Role card fidelity: is every SI-n and every requirement that prevents an unauthorised irreversible action in the floor, or in an explicit risk-acceptance request? Does every other requirement have a priority with a reason? Does every phase have scope, exit criteria and rollback? Is there at least one hypothesis that could refute the whole approach? Are drawings D-1 and D-2 present with id, caption and textual equivalent?
2. Ruling fidelity: does the document reopen, contradict, weaken or quietly extend any operator ruling (e.g. OQ-2 accounts target, OQ-3 numbers, OQ-8 D″ Tier 0 events, OQ-12 boundary model, OQ-14 D‴ no approval on silence, OQ-16 activity veto, RR-8 duplicate mains and R-67 precedence, RR-13/CR-21 TermLink-only delegation)? Anything presented as decided that is only a proposal?
3. Internal consistency: contradictions between sections, phases that use a capability before the phase that delivers it, floor items that cannot hold in an early phase without saying so, numbers that disagree with the requirements.
4. The open questions (section 13): are they genuinely the operator's, each with options and a recommendation, and is anything that needs an operator decision missing from the list?
5. Anything that would mislead step 4 reading this as its input.

End with: a verdict (fit / fit after listed fixes / not fit) and a numbered fix list, smallest first, marking any item that needs an operator decision rather than an editorial fix.
