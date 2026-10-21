# External design review: who may type into an AI agent's terminal?

You are an independent security and systems reviewer. Be critical; disagreeing with the
recommendation is useful. Answer in English, at most ~700 words, markdown, with the sections listed
at the end. Answer from this text only; do not run tools.

## Context

**TermLink** is a Rust substrate: one hub per Linux host (TLS JSON-RPC over TCP, plus a local Unix
socket), through which AI coding agents (Claude Code and similar, each running in a terminal session
TermLink manages) discover each other, exchange durable messages on topics, claim work, and control
terminal sessions. A small fleet (a handful of hosts, a few dozen agent sessions) is run by one
human operator. The charter's trust model is "a cooperating fleet with TOFU-pinned hubs and a shared
hub secret, not a boundary between mutually distrusting tenants". The operator has just ratified a
new objective, O-7: "every message, injection and lifecycle write can be traced to the identity that
made it, and is accepted only from an identity allowed to make it: no peer can act as the operator".

**Inject** (`command.inject`) types keystrokes into a managed terminal session: whatever is injected,
the agent in that terminal receives as if typed at its keyboard. It is how a per-project "sidecar"
process rings an idle agent when mail arrives (the doorbell), and how one agent can drive another's
terminal.

## Facts (verified in the code)

1. A session registered without `--token-secret` grants full control (including inject) to any
   connection from the same Unix user on its local socket.
2. Over the network, sessions are reached through the hub; the only gate is the hub's shared secret
   (an HMAC secret), which most peers in the fleet hold.
3. A session registered with `--token-secret` starts every connection at a read-only scope; a caller
   must present a token (`auth.token`, HMAC-derived from the session's secret) to reach Control scope,
   which inject needs.
4. The `termlink inject` CLI cannot present a token today (its arguments are target, text, enter,
   key, json, timeout). The sidecar injects through that CLI. So a session locked with
   `--token-secret` also locks out its own sidecar: no doorbell, mail sits unnoticed.
5. Channel posts carry an Ed25519 signature from the sender's identity key and the hub verifies it.
   Plain RPCs such as inject carry no identity signature, so a session cannot tell who injected.
6. A peer project (which runs dashboards and other agents) asked for an inject ACL.

## Options the operator is choosing between

- **A. Defer.** Build nothing yet; keep designing. The peer may lock sessions at the cost of their doorbell.
- **B. First slice now.** Give `termlink inject` a token (`--token` / env var) and have the sidecar
  present it, so `--token-secret` sessions keep working end to end. Keep the inception open for C.
- **C. Full design now.** Signed inject requests (sender identity), a per-session allowlist of
  identity fingerprints, and an operator attach path. Three questions are unanswered: (a) how an RPC
  carries a signature the session can verify (only channel posts are signed today), (b) where a
  session's allowlist comes from (spawner, hub, config), (c) how a worker session spawned by another
  project's supervisor gets the right allowlist from its first second without locking that supervisor out.
- **D. No-go.** Accept the risk under the cooperating-fleet charter.

The agent's recommendation is **B**: it closes the practical gap (the existing lock becomes usable
without breaking delivery) with a small, testable, reversible change, and leaves C, which is what
O-7 actually requires, as a separate decision once (a)-(c) are answered with the peer projects. It
names B's limit: a token proves knowledge of a secret, not identity, and records no attribution.

## Answer with these sections

1. **Verdict.** Which option, and would you change it?
2. **Risks of the recommended option** (including anything B makes worse, e.g. token distribution,
   token leakage through process lists or environment, a false sense of safety).
3. **What we are missing.** Prior art (SSH agent forwarding, tmux/screen socket permissions, X11
   access control, capability tokens, signed RPC schemes), attack paths not considered, simpler
   alternatives (e.g. per-session Unix-socket permissions, a separate inject socket), and how you
   would answer (a)-(c).
4. **Order of work.** If you were sequencing B and C, what comes first and what must B not foreclose.
