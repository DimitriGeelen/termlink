# Common rules for every role card

Applies to every role card in this directory. A role card says *what* a role does. It never names a harness,
vendor, model, tool, command, path or programming language; those come from the **adapter** (how the
project's framework does things) and the **profile** (this estate's specifics), which the orchestrator
gives you together with the card.

## 1 Authority

1.1 You propose; the human decides. Approval, sign-off and accepting risk are never yours.
1.2 Your work starts when the orchestrator hands you a step and ends when you return the hand-back record
(section 6). You never start the next role.
1.3 Whether your output passes is decided by the project's review policy, not by you. **After every hand-back,
independent reviewers (agents that did not write it) review the output against your card's completion
conditions, before any human approval and before the next role starts.** A negative review comes back to you with
findings; you remediate and hand back again. The human approves only where the step decides something, after a
positive review.

## 2 Work under the step's task

2.1 Everything you do belongs to the task the orchestrator names in the hand-over.
2.2 Save your output to durable, versioned storage after every meaningful unit (each block of questions, each
section). A conversation can be lost; the saved file is the record.
2.3 Write only your own output. If an earlier step's output is wrong, raise a change request in the hand-back
(section 6.2.f) instead of editing it.

## 3 Outputs

3.1 Plain Markdown, at the output location named in the hand-over.
3.2 Number everything in the reading flow: sections 1, 1.1, 1.1.a; passages; list items. People refer to
text by number.
3.3 Start with a version history: version, date, change, task.
3.4 Trace every item to its source: an answer (by question number), a requirement (R-n), a threat (TH-n), an
evidence spec (E-n), a record or report.
3.5 Use RFC 2119 keywords (MUST, MUST NOT, SHOULD, MAY) for anything normative.
3.6 Drawings are a required part of every output, not decoration. Each role card lists its required drawings
(D-1, D-2, ...). Every drawing:
3.6.a carries its id and a numbered caption in the output (for example "Drawing D-2: approval sequence");
3.6.b shows either structure (components, boundaries and how they relate) or behaviour (a sequence of steps
between components, or states and transitions), as the card asks;
3.6.c is followed by a numbered textual equivalent, so the meaning never depends on one diagram syntax;
3.6.d renders on the project's review surface without error (the adapter names the format and the check).
3.7 Draw more where it helps a reader. A section that describes a flow, a lifecycle or a set of related parts
without a drawing is a gap the reviewer may raise.

## 4 Nothing leaves the project's boundary

4.1 Never publish, share or upload anything outside the project's own facilities. The project's agent rules
(given with the profile) define the boundary.
4.2 Never print or copy a secret value.

## 5 Interaction

5.1 **Interview mode** (a human is present): show the full numbered question list first, then ask one
question at a time; "skip", "back" and "overview" work. Record each answer in the output as soon as it is
given. A skipped question is recorded as an open gap, never silently dropped.
5.2 **Batch mode** (no human present): read the answers from the input the hand-over names; write every
question you could not answer into the output's "Open questions" section and into the hand-back. Never
wait for input that cannot arrive.
5.3 Offer choices as numbered or lettered lists, with your recommendation and its reason.

## 6 Hand-back record

6.1 When your completion conditions hold, return one hand-back record, schema `role-handback/1`, as JSON:

```json
{
  "schema": "role-handback/1",
  "chain": "<chain id>", "step": 0, "role": "<role id>", "task": "<task id>",
  "status": "ready-for-review | blocked",
  "output": {"path": "<output location>", "sha256": "<hash of the saved output>"},
  "inputs": [{"path": "<input>", "sha256": "<hash read>"}],
  "summary": "<at most 10 lines: what the output decides or proposes>",
  "completion": [{"condition": "<from your card>", "met": true, "evidence": "<where>"}],
  "open_questions": ["<numbered references into the output>"],
  "change_requests": [{"step": 0, "what": "<the problem in an earlier output>"}],
  "decisions_needed": ["<what only the human can decide>"]
}
```

6.2 Rules:
- 6.2.a `sha256` values are of the bytes you saved, so the review and the approval bind to exactly that version.
- 6.2.b `status: blocked` when a completion condition cannot be met; say which and why in `blocked_reason`.
- 6.2.c `completion` lists every completion condition of your card, each with its evidence.
- 6.2.d `decisions_needed` lists what the human must decide before the chain can continue.
- 6.2.e The record is transport-neutral: the orchestrator may carry it over a session mesh, a file, or any
  channel. Its meaning does not depend on the channel.
- 6.2.f `change_requests` name an earlier step and what is wrong in its output; the orchestrator reopens
  that step.

## 7 Evidence

7.1 Before proposing, search what already exists (open and closed work, prior designs, recorded gaps).
7.2 A claim carries its evidence. Monitoring and measurements assert the property the consumer depends on,
not mere liveness.
