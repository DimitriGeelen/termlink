# T-1693 — Per-agent ed25519 signing identity on shared hosts

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/completed/T-1693-per-agent-ed25519-signing-identity-on-sh.md`.
> The exploration and decision below happened on 2026-05-18; this file relocates
> that trail out of the archived task file into `docs/reports/` per C-001. No
> finding here is new.
>
> **Decision on record: GO** after a design phase (2026-05-18T21:02:26Z),
> operator-prioritised **after** T-1692.

## The gap

T-1427 enforcement — the envelope signature must match the host-derived identity —
revealed that **all co-resident agents on .107** (penelope, cohort-agent,
framework-agent, termlink-agent, and the Claude session) sign with a single
host-wide ed25519 key, `d1993c2c`.

The fingerprint `9219671e`, which had been treated as "Pen's" for six or more
weeks, **lives in payload only, never in the envelope**. Co-resident attribution at
the envelope layer was therefore *structurally unanswerable*.

That six-week misattribution is the part worth remembering: nothing was lying, and
nothing detected the difference between a payload label and a cryptographic
identity.

**What it affects:** audit trails on cross-agent contracts; multi-tenant trust
(one host = one trust unit); defence-in-depth against per-agent compromise; and
clean separation as more agents land on shared hosts.

**What was already built:** T-1159 (work-completed 2026-04-20) had already added
per-session ed25519 keyring infrastructure to `termlink-session`. The gap was
never the crypto — it was the *deployment and wiring model*: how agents provision
keys, how a key binds to a session at registration time, and how rotation works.

## Assumptions

- **A1** — T-1159's per-session keyring is reusable per-agent, not only as a
  per-host default. *(Verify by reading the session-init path.)*
- **A2** — `termlink register`, or the equivalent session-start verb, is the
  natural insertion point for an `--identity-key` flag or env var.
- **A3** — no protocol change is required: envelope signing already supports
  arbitrary ed25519 keys; the gap sits at the session-instantiation surface.
- **A4** — TOFU and `KnownHubStore` handle multiple identities cleanly when
  different sessions present different pubkeys.

## Design shapes considered

Steel-manned from cohort-agent's letter, scored against the four Constitutional
Directives.

### Shape 1 — per-agent key files, agent-managed · **recommended**

Each agent owns a keypair under its own project's secrets directory at mode 0600,
and passes the path at session registration via `--identity-key`. The hub does not
centrally manage keys.

Antifragility ✓ *(no central store to lose)* · Reliability ✓ · Usability ✓
*(mirrors the existing pattern, e.g. `instance/secrets/pen_outbound.key`)* ·
Portability ✓ *(no hub-side dependency)*

*Risk:* operators must remember to back up identity files. *Mitigation:* surface
in `fw doctor` plus a documented convention.

### Shape 2 — hub-managed keystore

The hub stores keys and serves them to registered sessions. Centralised, easier
rotation, a single backup point.

Antifragility ◌ *(central failure mode)* · Usability ◌ *(new infra)* ·
Portability ✗ *(couples agents to a hub keystore)*

### Shape 3 — operator-derived deterministic keys

Each agent's key derived from `(operator-master-key, agent-name)` via HKDF.

Antifragility ✗ *(master-key compromise compromises every agent)* · Usability ✓ ·
Portability ◌

## Decision — GO (after design phase)

A real structural gap with named implications: audit, multi-tenant trust,
defence-in-depth. T-1159 already shipped the foundation, so this is a wiring model
rather than new crypto. Shape 1 preserves agent sovereignty and matches the
existing project-local secrets convention.

The rotation story is deliberately minimal — *regenerate, restart, and TOFU
history covers old envelopes* — accepted as viable under the current threat model.

Sequenced **after** T-1692 per cohort-agent's letter: *"(A) first, smaller and
immediate consumer impact; (B) second, independent and larger."*

### Evidence

- T-1427 error `-32014` — empirical proof that `9219671e` was never an envelope
  identity
- 19 historical envelopes labelled "Pen", all signed by the host key `d1993c2c`
- T-1159 (completed) — keyring infrastructure exists; wiring is the gap
- Cohort-agent's letter — an explicit ask carrying a full steel-man analysis
- A concern registered as *"co-resident envelope identity undifferentiable"*

## Relationship to neighbouring tasks

| Task | Relationship |
|---|---|
| **T-1159** | Predecessor — the keyring exists; this is the wiring model |
| **T-1448** | Sibling (`owner: human`, in flight). T-1448 explores co-resident *disambiguation* at the payload layer; T-1693 provisions per-agent *identity* at the envelope layer. T-1693 enables T-1448's strongest variant but is independent of it |
| **T-1427** | The enforcement that surfaced the gap — no change needed |
| **T-1457** | Register identity on .141 under the existing host-shared model; a compatibility flag for the per-agent-key path |
