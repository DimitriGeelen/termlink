# Envelope-versus-content principle: review comparison (T-3351, 2026-10-08)

Brief: `brief.md` (the operator's idea, formalised as P1-P4). Answers stored unedited: `codex.md` (Codex,
`codex exec -s read-only`, rc 0) and `glm.md` (GLM-5.3 via opencode, rc 0). Evidence for the operator, not a verdict.

## 1 Verdict

Both: **adopt with changes.** Both call the direction sound and the standard pattern.

## 2 Where they agree

1. P1 and P3 contradict each other: P1 lets a project read all its agents' mail, P3 encrypts agent mail only to the
   agent. One must give.
2. The "mailman" understates the hub: it is also the key directory (cards), the recovery store and the monitor.
   First-contact key substitution by a hostile hub is the real attack; card/key fingerprints must be pinned out of band
   (GLM: a prerequisite, a fleet pin list; Codex: manually approved pinned bindings).
3. The envelope leaks the who-talks-to-whom graph and timing; state it as an accepted residual.
4. Mechanism: a random content key per message, wrapped to each authorised recipient's key (fan-out); separate
   encryption and signing keys; no shared project private key needed at home-lab scale.
5. An encrypted hub copy recovers a lost store only with bounded retention and surviving recipient keys; ciphertext
   retention must be bounded (else a forever-store with a forever metadata map).
6. P4 is not real until GP-11: an off-host operator key with message keys wrapped to it in advance, scoped unlock,
   off-host log, time-boxed; Tier 0 (a harness hook) cannot be the control.
7. Costs: hub content search, body digests and content telemetry move to endpoints; routing, stuck detection and
   transport canaries keep working on the envelope; debugging needs receiver-side "show my own mail" tooling.
8. It closes the hub axis, not the local-root axis (RR-2; the OQ-2 per-agent accounts are what close that).

## 3 Where they differ: how to resolve P1

- Codex: explicit reader sets per message; the project is a default collaboration group; distinguish agent-private
  from project-shared mail; if universal project read is intended, say "addressing selects the worker, membership
  grants reading".
- GLM: the project reads by right only project-addressed mail and mail that fell back or dead-lettered (otherwise
  fallback is theater); reading a live agent's mail is a logged, deliberate audit act, never ambient; blanket project
  read would undo the per-agent OS accounts the operator ruled in OQ-2.

## 4 Suggested order (GLM 3c, compatible with Codex 3f)

1. Pin hub card-signing keys out of band. 2. Encrypt agent-to-agent DMs. 3. Project-addressed mail by fan-out.
4. Break-glass after GP-11.
