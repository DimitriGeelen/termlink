# T-3302 — Which identity is canonical for claude-termlink? (decision brief)

Status: awaiting the operator's decision. Recommendation: **B, per-agent key canonical.**

## Facts (measured 2026-10-01, read-only)

- **Two keys exist for this agent.**
  - The shared **host key** `d1993c2c3ec44c94` lives at `~/.termlink/identity.key`. Anything on the host without its own key signs with it.
  - The **per-agent key** `6738c073bbcc587a` lives at `~/.termlink/identities/claude-termlink.key` and is what `TERMLINK_AGENT_ID=claude-termlink ... --resolve` returns.
- **Every live session signs with the host key; none with the per-agent key.**
  - Registration fingerprints: `/var/lib/termlink/sessions` 24 x d1993; `/tmp/termlink-0/sessions` 102 x d1993; 0 x 6738.
  - This interactive Claude session has no `TERMLINK_AGENT_ID`, so it resolves `source: host_default` -> d1993.
- **Mailboxes:** 11 `dm:*` topics involve d1993. One involves 6738, and that is T-3061's own test topic. The sidecar watches both (`.context/cron/notify-sidecar-agents.conf`, `claude-termlink` / `claude-termlink-alt`).
- **There is precedent for per-agent keys.** The systemd agents `framework-agent`, `termlink-agent` and `cashweb-integration` each run with `--identity-key` (their own key). The reason is written in their unit files: with a shared host key, attribution falls back to `from_project` / `_from` metadata, which the envelope signature does not cover, **so it can be forged**.
- **Cost seen this session:** pen-agent shares d1993, so pen's acks count as ours. SQ-19's receiver-ack-lag check had to demote a class because a topic fingerprint could not tell the two apart.

## Options

- **A — host key canonical.**
  - Do: retire the `-alt` mailbox. Zero migration.
  - Cost: every session on the host, plus pen, stays indistinguishable on the wire; attribution stays forgeable; ack-lag ambiguity stays.
- **B — per-agent key canonical (recommended).**
  - Do: set `TERMLINK_AGENT_ID=claude-termlink` for this project's Claude sessions, so they sign with 6738. This matches how the systemd agents already work.
  - Keep both sidecar mailboxes watched during the transition, since peers that copied old d1993 topic names still land there. Retire d1993 from this agent's config once those topics go quiet. Peers that resolve by agent id reach 6738 already (T-3061).
  - Cost: one config change (where the session's env is set) and a period with two live mailboxes, which already exist. pen-agent getting its own key is a separate follow-up for its owner.
- **C — keep both indefinitely.**
  - Do: the status quo. Mail to either is delivered.
  - Cost: the attribution problem in A persists, and so does the confusing dual mailbox.

## Scoring against the joint value drivers (operator request, 2026-10-01)

Each option scores −2..+2 per driver, multiplied by the driver's weight. Drivers come from `policy/value-drivers.yaml` (framework and TermLink). F1–F3 (prompt / context / component fabric) are not affected by this choice and are left out.

| Driver (weight) | A host key | B per-agent | C both | Why |
|---|---|---|---|---|
| D1 Antifragility (9) | −1 | +1 | 0 | One shared key = one blast radius: rotate or lose it and every agent on the host changes identity at once. Per-agent keys contain that. |
| D2 Reliability (7) | −2 | +2 | −1 | Auditable attribution. Under A, acks and receipts from pen and 126 sessions are indistinguishable (the SQ-19 class). Under C the ambiguity remains and the mail splits across two boxes. |
| D3 Usability (5) | +1 | 0 | −1 | A is simplest. B has a transition with two mailboxes, then is simple. C stays confusing indefinitely. |
| D4 Portability (3) | 0 | 0 | 0 | All use the existing env/key mechanism. |
| F-RECALL (6) | −1 | +1 | 0 | Per-agent sender_id makes history (recent-dm, ack-history, claims) answerable per agent. |
| F-AUTONOMY (4) | −1 | +1 | 0 | Unattended wake/inject routes by fingerprint. Under A, pen's injector and ours watch the same identity. |
| F-ORCH (5) | −1 | +1 | 0 | Orchestration (find-idle, claim, handoff) can trust who signed. |
| **Weighted total** | **−33** | **+38** | **−12** | Indicative, not precise. The ordering is robust; the magnitudes are judgement. |

### Steelman / strawman

- **A, steelman.** On one host, every agent runs as root and can read every key in `~/.termlink/identities/`. So per-agent keys give **no protection against a malicious co-resident agent**: "forgeable" is true either way within the host. A has zero migration, keeps the 11 live threads where they are, and adds no new moving parts right after T-3291 showed what moving parts cost.
- **A, strawman.** "It works, so don't touch it." This ignores that it is already *not* working as intended: pen's acks count as ours, and an ack-lag check class had to be demoted because of it.
- **B, steelman.** It removes a whole class of misattribution (acks, receipts, claims, presence) by construction rather than by heuristics. It matches what the systemd agents already do and why. The delivery rail already watches the per-agent mailbox, so most of the plumbing exists. It scales to many agents per host and allows per-agent rotation and revocation.
- **B, strawman.** "Per-agent keys make us secure." False, per A's steelman: B buys **accidental-misattribution prevention and observability, not security against a hostile local process**. If that is the goal, B is not enough; it would need per-agent OS users or a key agent.
- **C, steelman.** Nothing breaks, nothing to build, and the sidecar delivers to both. Decide later, when a concrete incident shows which way to go.
- **C, strawman.** Indecision as policy. T-3061 set up the dual watch explicitly as *temporary* pending this decision, and keeping it permanently is the confusing state A and B each try to end.

**Net:** B, on the honest grounds of reliability and attribution clarity, not security. If security against co-resident agents ever becomes a goal, that is a separate, larger decision (per-agent OS users).

## What GO on B would authorise (build tasks, not done here)

1. **Find where this project's Claude sessions get their environment** (claude-fw launcher / `.claude/settings.json` `env`) and set `TERMLINK_AGENT_ID=claude-termlink` there. Verify with `termlink agent identity --resolve --no-create` that it prints `6738...` and `source: per_agent`.
2. **Mailbox migration:** keep `claude-termlink-alt` (6738) and `claude-termlink` (d1993) in the sidecar until d1993's topics are quiet for N days, then rename: 6738 becomes `claude-termlink` and the d1993 line is removed.
3. **Tell pen's owner** that pen shares the host key (a separate decision on their side).
