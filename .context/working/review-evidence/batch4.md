# Batch 4 Review Evidence — T-1529..T-1535

Date: 2026-05-22
Binary: `termlink` (PATH-resolved), version 0.11.1
Hub: local (agent-chat-arc, 105 thread roots of real data)

Mutation-safety rule applied: none of the 4 mutating verbs (forward/edit/redact/describe)
expose a `--dry-run` flag, so live mutation on real chat-arc data was SKIPPED. Their
`--help` output is captured instead, evidencing the verb EXISTS with the correct interface
matching each task's ACs. The 3 read-only verbs (threads/redactions/pin-history) were run live.

---

## T-1529 — agent forward (MUTATING, no --dry-run)

Command: `termlink agent forward --help`
Verdict: HELP-ONLY (live mutation skipped to protect real chat-arc data)

```
Re-publish a chat-arc post to another topic (T-1529): thin wrapper over `channel forward`.
Walks `agent-chat-arc` to fetch the envelope at `<offset>` and re-posts it to `--to <TOPIC>`
with metadata.forwarded_from=agent-chat-arc:<offset> and metadata.forwarded_sender=<original-sender>.

Usage: termlink agent forward [OPTIONS] --to <TO> <OFFSET>

Arguments:
  <OFFSET>  Offset on agent-chat-arc to forward

Options:
      --to <TO>    Destination topic
      --hub <HUB>  Override hub address (default: local hub)
      --json       Output result as JSON envelope
  -h, --help       Print help
```

Interface matches AC: positional `<OFFSET>`, `--to <TOPIC>`, `--hub`, `--json`. No `--dry-run`.

---

## T-1530 — agent edit (MUTATING, no --dry-run)

Command: `termlink agent edit --help`
Verdict: HELP-ONLY (live mutation skipped to protect real chat-arc data)

```
Edit a prior chat-arc post (T-1530): thin wrapper over `channel edit agent-chat-arc`.
Posts a msg_type=edit envelope with metadata.replaces=<offset> and the new text payload.

Usage: termlink agent edit [OPTIONS] <OFFSET> <TEXT>

Arguments:
  <OFFSET>  Offset of the prior post being edited
  <TEXT>    New text payload

Options:
      --hub <HUB>  Override hub address (default: local hub)
      --json       Output result as JSON envelope
  -h, --help       Print help
```

Interface matches AC: positional `<OFFSET> <TEXT>`, `--hub`, `--json`. No `--dry-run`.

---

## T-1531 — agent redact (MUTATING, no --dry-run)

Command: `termlink agent redact --help`
Verdict: HELP-ONLY (live mutation skipped to protect real chat-arc data)

```
Retract a prior chat-arc post (T-1531): thin wrapper over `channel redact agent-chat-arc`.
Posts a msg_type=redaction envelope with metadata.redacts=<offset> and optional metadata.reason.
The arc remains immutable — readers see the redaction marker, not a deletion.

Usage: termlink agent redact [OPTIONS] <OFFSET>

Arguments:
  <OFFSET>  Offset of the post to redact

Options:
      --reason <REASON>  Optional reason logged on the redaction envelope
      --hub <HUB>        Override hub address (default: local hub)
      --json             Output result as JSON envelope
  -h, --help             Print help
```

Interface matches AC: positional `<OFFSET>`, `--reason`, `--hub`, `--json`. No `--dry-run`.

---

## T-1532 — agent describe (MUTATING, no --dry-run)

Command: `termlink agent describe --help`
Verdict: HELP-ONLY (live mutation skipped to protect real chat-arc data)

```
Set chat-arc topic_metadata description (T-1532): thin wrapper over
`channel describe agent-chat-arc`. Posts a msg_type=topic_metadata envelope with
metadata.description=<text>. WRITE companion to T-1524 `agent info`.

Usage: termlink agent describe [OPTIONS] <TEXT>

Arguments:
  <TEXT>  New topic description text

Options:
      --hub <HUB>  Override hub address (default: local hub)
      --json       Output result as JSON envelope
  -h, --help       Print help
```

Interface matches AC: positional `<TEXT>`, `--hub`, `--json`. No `--dry-run`.

---

## T-1533 — agent threads (READ-only, run live)

Command: `termlink agent threads`
Verdict: EVIDENCE-CLEAN

```
Threads on 'agent-chat-arc' (105 roots):
  [1785] replies=2 participants=1 last_ts=1779383756679: @ring20-management — two items combined, in_reply_to chat-…
  [224] replies=15 participants=1 last_ts=1779302415485: {
  "subject": "Contract design — cohort member alias+forw…
  [1737] replies=3 participants=1 last_ts=1779302415485: {
  "subject": "Provision request — Jack Put (warm precurs…
  [1740] replies=2 participants=1 last_ts=1779302415485: {
  "subject": "Provision ack — Jack Put onboarded",
  "fr…
  [712] replies=2 participants=1 last_ts=1779302104264: {
  "subject": "Probe — does 050-email-archive see inbound…
  [1741] replies=1 participants=1 last_ts=1779302104264: {"subject":"Re: Probe — yes, Pen sees inbound; classifier …
  [1660] replies=1 participants=1 last_ts=1779217072056: {
  "subject": "Add Mustafa Burak İlter to forwarding-verif…
  [1654] replies=1 participants=1 last_ts=1779217071782: {
```

105 thread roots with child counts, participants, last_ts — matches AC expectation.

---

## T-1534 — agent redactions (READ-only, run live)

Command: `termlink agent redactions`
Verdict: EVIDENCE-CLEAN

```
Redactions on 'agent-chat-arc':
  [210] redacts → [209] by d1993c2c3ec44c94 reason="empty payload — jq quoting failure, reposting":
  [232] redacts → [229] by d1993c2c3ec44c94 reason="scope correction — over-asked Mail.Read ...":
  [235] redacts → [233] by d1993c2c3ec44c94 reason="underspecified — operator wants explicit alias list ...":
  [237] redacts → [236] by d1993c2c3ec44c94 reason="follow-up pointing to redacted :233 ...":
  [361] redacts → [360] by d1993c2c3ec44c94 reason="smoke test": T-1531 redact smoke baseline
  [379] redacts → [378] by d1993c2c3ec44c94 reason="empty payload — file path failed to resolve":
  [1556] redacts → [1554] by d1993c2c3ec44c94 reason="Source workflow JSON had nested-= expression bug ...":
```

List of redactions with target offset + reason — matches AC expectation. Note [361] is the
T-1531 redact smoke-test baseline, confirming the write-side verb functioned during build.

---

## T-1535 — agent pin-history (READ-only, run live)

Command: `termlink agent pin-history`
Verdict: EVIDENCE-CLEAN

```
Pin history for 'agent-chat-arc':
  [7] PIN → [6] by d1993c2c3ec44c94: {"event":"inception-rfc","task":"T-1425","title":"agent-to-a…
  [355] PIN → [346] by d1993c2c3ec44c94: Session wrap: 14 features shipped (T-1511..T-1524). Handover…
```

Chronological pin events with target offsets — matches AC expectation.

---

## Summary

| Task  | Verb         | Verdict        | Note |
|-------|--------------|----------------|------|
| T-1529 | forward      | HELP-ONLY      | No --dry-run; interface correct, live mutation skipped |
| T-1530 | edit         | HELP-ONLY      | No --dry-run; interface correct, live mutation skipped |
| T-1531 | redact       | HELP-ONLY      | No --dry-run; interface correct, live mutation skipped |
| T-1532 | describe     | HELP-ONLY      | No --dry-run; interface correct, live mutation skipped |
| T-1533 | threads      | EVIDENCE-CLEAN | 105 thread roots rendered |
| T-1534 | redactions   | EVIDENCE-CLEAN | Real redaction log with offsets + reasons |
| T-1535 | pin-history  | EVIDENCE-CLEAN | 2 pin events, chronological |
