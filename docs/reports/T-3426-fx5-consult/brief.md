# Independent consult: does a hub's file store count toward a "no growth of readable content" floor? (arc-011 FX-5)

You are an independent reviewer. Work from the text below only; do not read other files and do not modify
anything. The operator asked for an outside opinion before ruling. Disagree where you see reason to; a
confirmation without reasons is of no use. Answer in at most 600 words, with numbered points.

## Context

1. TermLink is a message hub for AI coding agents on a host. Per-agent "sidecars" send signed messages through the
   hub. A floor item (must hold, not traded off), FX-5: "Disclosure does not grow. No feature writes content to a hub
   in a form a token holder or the hub operator can read beyond what today's rails already expose." Its check in this
   phase: "count plaintext content fields on the hub per phase: never above the PH-0 baseline". In later phases,
   copies of conversation turns on the hub are encrypted per message to their readers.
2. A baseline was just measured from the code: 8 plaintext content fields across 7 message rails (channel posts, DM
   turns, chat broadcasts, an inbox mirror, receipts, the sidecar message body, topic descriptions). A script fails if
   the count rises above 8.
3. The hub also has an artifact store: the file-transfer blob store behind `termlink file send`. Files (up to 2 GiB)
   are kept unencrypted on the hub's disk, keyed by their SHA-256. The hub operator can read them, and so can a client
   holding a token of the highest permission level (the artifact RPCs fall to the deny-by-default "Execute" scope).
   The baseline left it out; the measurer could not tell whether FX-5's author meant it to count.
4. An upcoming build row adds attachments ("blobs") to sidecar messages ("the sender gives the message, optionally
   with a blob, to its own sidecar"). The sidecar's send path already carries an unused `artifact_ref` field, so the
   obvious implementation routes attachments through this artifact store.
5. The metric counts FIELDS, not volume: routing attachments through an already-counted store raises no number.

## Options

- A: count the artifact store (baseline 9); attachments may use it.
- B: do not count it (baseline stays 8); the attachment build must keep blobs off the hub or encrypted, enforced by
  review only.
- C: count it (baseline 9, honest about today) AND add a check rule that fails if sidecar code writes unencrypted
  content to the artifact store; attachments must be encrypted to the recipient or travel sidecar to sidecar.
- D: defer to the attachment build; baseline stays 8 for now.

The proposer recommends C although A scored higher (A +22, C +17, D −5, B −18 on weighted drivers: antifragility,
reliability, usability, count-truthfulness, operator-simplicity), arguing that no value driver measures
confidentiality and FX-5 is a floor that must hold regardless of score.

## Questions

1. Which option would you pick, and why? Is there a better option?
2. Is the proposer right that routing agent attachments through the existing file store would be "growth" in FX-5's
   sense, even though the field count does not rise? Or is it "what today's rails already expose"?
3. Is "count fields" the right metric for FX-5 at all? What would measure disclosure growth better (e.g. classes of
   content, readers, volume)?
4. If C: is a static check "sidecar code must not write unencrypted content to the artifact store" enforceable, and
   what would it miss? Is encrypting to the recipient the right requirement, given that the hub operator is the same
   person as the host operator in this deployment?
5. Anything in the proposer's framing you think is biased or wrong (including the decision to recommend against the
   top score).
