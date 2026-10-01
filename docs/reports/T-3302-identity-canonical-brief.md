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

## What GO on B would authorise (build tasks, not done here)

1. **Find where this project's Claude sessions get their environment** (claude-fw launcher / `.claude/settings.json` `env`) and set `TERMLINK_AGENT_ID=claude-termlink` there. Verify with `termlink agent identity --resolve --no-create` that it prints `6738...` and `source: per_agent`.
2. **Mailbox migration:** keep `claude-termlink-alt` (6738) and `claude-termlink` (d1993) in the sidecar until d1993's topics are quiet for N days, then rename: 6738 becomes `claude-termlink` and the d1993 line is removed.
3. **Tell pen's owner** that pen shares the host key (a separate decision on their side).
