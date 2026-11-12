# External consultation brief: dispatch for a non-root agent session (T-3759)

You are an independent reviewer. Answer from your own reasoning; read the repository at /opt/termlink where the
brief points you, but do not edit any file except your own answer file. Do not run commands that change state.

## The system in two paragraphs

TermLink is a message, session and hub layer through which AI agent sessions on a few Linux hosts send mail and start
worker sessions. A hub process runs as root on each host. It keeps its state in a runtime directory (here mode 0700,
root-owned), which holds the hub's HMAC secret, its TLS key, and one small file per registered session (the hub
discovers sessions by reading that directory). Remote or other clients can reach the hub over TCP with a capability
token of a limited scope (observe, notify, interact; control and execute stay with holders of the secret).

"Dispatch" (`fw termlink dispatch`, a framework wrapper) starts a worker session (an AI agent run for a task, e.g. a
review). The worker registers as a session in the hub's runtime directory so that the hub and other agents can find
and message it. Since this week, `termlink register` refuses a runtime directory the running hub does not serve,
because a session registered elsewhere was invisible for 10 days on another host and messages to it were lost.

## The problem

One project's design session runs as a NON-ROOT user on a host whose hub runs as root. Tonight that user was given
its own interact token over TCP (send, read, list): messaging works. But it cannot dispatch workers: it cannot write
into the root-only runtime directory, and registering anywhere else is refused. The owner wants that session to be able
to start reviews (dispatch) without weakening the host's security, and without a human relaying requests.

## Options on the table

- A. The user runs its own hub (own runtime directory under its home, own port). Its workers register there; it talks
  to the main hub over TCP with its token. Its workers are not visible on the main hub.
- B. Run the design session as root (the user may use sudo). No new code.
- C. A small root-owned dispatch service: the non-root session sends a dispatch request (as a message or RPC); the
  service checks it against an allow list (which task types, which worker kinds, limits) and starts the worker on the
  main hub as root, logging who asked. Must be built.
- D. No dispatch for that user: it asks a root-run agent session of the same project to run reviews on its behalf.

The orchestrator's current lean: C long-term, D until C exists.

## What we ask you

1. Rank A-D and say why, with the security and reliability argument for each. Name the risks you see in the option
   you rank first, including privilege-escalation paths (a non-root caller causing root to execute something).
2. Is there an option E we missed (for example: a group-writable sessions subdirectory, per-user hub federation,
   socket activation, polkit/sudo rules for one command, the hub accepting session registration over TCP with a
   scoped token)? Evaluate it the same way.
3. If C (or your E) is chosen: what must the allow list and the request contain, how is the requester authenticated,
   what is logged, and what must fail closed?
4. Anything in the framing above you think is wrong.

Relevant files if you want to check claims: `crates/termlink-cli/src/commands/` (register, hub token),
`crates/termlink-session/src/` (registration, runtime dir), `docs/operations/fleet-join.md`, CLAUDE.md (project rules).

## Answer format

Write your answer to the file path given in your prompt: numbered sections matching 1-4, under 800 words, plain
language, cite file:line for any code claim. End with one line: `RANKING: <e.g. C > D > A > B>`.
