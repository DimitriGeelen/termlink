# T-3076 — Binary blob on the notify rail via artifact.put (inception research)

## Problem statement

arc-011 slice S2 ("payload may carry a binary blob") needs a shell script — the
notify rail (`scripts/notify-sidecar.sh`) — to move artifact BYTES, not just a
`--artifact-ref` pointer, over `termlink channel post`.

Measured 2026-09-22 (T-3076 IW-1, confidence 3, re-confirmed here 2026-09-24):

- `termlink channel post --artifact-ref <ref>` exists — a topic message CAN
  carry a pointer.
- `artifact.put` / `artifact.get` are hub-routed protocol methods
  (`control.rs:316,323`, `router.rs:143,146`) wrapped by
  `send_artifact_via_client` / `download_artifact_via_client`
  (`crates/termlink-session/src/artifact.rs:137,525`).
- Re-confirmed today: those two functions have exactly three callers —
  `crates/termlink-cli/src/commands/file.rs` (`cmd_file_send`/`cmd_file_receive`,
  i.e. `file send`/`file receive`), `crates/termlink-cli/src/commands/remote.rs`,
  and `crates/termlink-mcp/src/tools.rs`. **No `termlink artifact` CLI subcommand
  exists** (`grep '"artifact"'` across `cli.rs`/`commands/*.rs` finds nothing).

So the gap is exactly as IW-1 described: a shell script can attach a reference
but cannot move the bytes. `artifact.rs`'s own module doc confirms the intended
shape — `send_artifact_via_client` is explicitly "a single async entry point
that callers ... use", i.e. designed to be reused, not file-send-specific.

## Sovereign question — already resolved

IW-2 ("Should TermLink grow `artifact put`/`artifact get` CLI verbs?") is
recorded RESOLVED in `.context/arcs/arc-011.yaml` (SQ-3, 2026-09-23): **YES**.
Rationale on file: the two verbs are thin wrappers over functions that already
exist and are already exercised by `file send`; this is a primitive
(charter non-goal 4 disclaims orchestration, not primitives), consistent with
`target_blast_radius: 3` already declared on this task.

IW-3 (eager/lazy fetch, `--expected-sha256` mandatory?) was deferred behind
IW-2 and answered conditionally at filing: **mandatory**, because
`artifact_ref` IS the sha256 so verification is free (T-2472 precedent: an
unverified fetch must not claim "verified").

## Recommendation

**Recommendation:** GO
**Rationale:** Root cause is identified (no CLI surface over existing,
already-tested functions) with a bounded fix path (two thin wrapper
subcommands, `blast_radius: 3` — file.rs, cli.rs enum entry, one new small
module or extension of artifact.rs, plus fixtures). The sovereign question
that gated this (new CLI surface on a project that prunes aggressively) is
already answered YES by the operator (SQ-3). Nothing discovered today
contradicts that answer or widens the scope.
**Evidence:**
- `send_artifact_via_client`/`download_artifact_via_client` are stable,
  already unit-tested (`artifact.rs` lines ~809-1064), and used from 3
  call sites today — a 4th (CLI `artifact put`/`get`) is additive, not novel.
- No competing CLI name collision (`grep '"artifact"'` returns nothing in
  `cli.rs`).
- `--expected-sha256` verification is free (the ref already IS the hash);
  making it mandatory on `artifact get` costs nothing and closes a T-2472-class
  gap before it can open.

## What this inception does NOT authorise

Per Inception Discipline, no build artifacts were written. If GO is recorded
via `fw inception decide T-3076 go` (Tier-0, human-only — this agent hit that
gate directly attempting even `--help`), the follow-up is a **separate build
task**, scoped to: `termlink artifact put <path> --to <peer>` /
`termlink artifact get <sha256> --expected-sha256 <sha256> -o <path>`, both
thin wrappers, with `--expected-sha256` required (not optional) on `get`.
