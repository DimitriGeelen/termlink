# OBS-108 successor-path audit (G-089 scoping)

Read-only audit. Question: does the content-addressed **successor** transfer path
(`channel.post`+`artifact.put` send / `channel.subscribe`+`artifact.get` receive)
inherit the OBS-108 false-green that T-2472 just fixed in the legacy `file send`/`file receive`
verb? OBS-108 = receiver reports success on the wrong/older artifact because it only ever
compares bytes against the sha the SENDER advertised (self-consistent), never a
RECEIVER-supplied EXPECTED digest.

## Successor path map

There is **no standalone `channel post --file` / `channel subscribe --to-file` verb**.
`channel post` (cli.rs:1903) accepts only `--payload`/stdin/`--artifact_ref` (opaque string);
`channel subscribe` (cli.rs:2509) only renders envelopes — it never writes artifact bytes to disk.
The channel content-addressed transport is reachable **only through the `file` verb**, which was
rewired onto it in T-1249/T-1250:

- Send: `send_artifact_via_client` → `channel.post` + `artifact.put`
  (`crates/termlink-session/src/artifact.rs`; hub store `crates/termlink-bus/src/artifact_store.rs:94`).
- Receive: `cmd_file_receive` → `try_recv_via_artifact` → `recv_artifacts_via_client`
  (`artifact.rs:367`, `channel.subscribe`) → `process_artifact_batch`
  (`crates/termlink-cli/src/commands/file.rs:493`) → `download_artifact_via_client`
  (`artifact.rs:466`, `artifact.get`) → `std::fs::write` (`file.rs:538`).

The only receiver-to-disk materialization in the codebase is `cmd_file_receive` (three write
sites: channel path file.rs:538, legacy-event fallbacks file.rs:859 and file.rs:976) plus the
MCP twin `termlink_file_receive` (tools.rs:13706-13966, legacy events only).

## Transit integrity

Present on the successor path. `download_artifact_via_client` (`artifact.rs:534-539`) hashes the
reassembled bytes and rejects a content-address mismatch:

```rust
let got = hex_sha256(&out);
if got != sha256 {
    return Err(io::Error::other(format!(
        "artifact.get sha256 mismatch: requested {sha256}, computed {got}")));
}
```

This proves "bytes == the sha I asked for" — content-address integrity, NOT intent verification.

## Receiver-expected-digest seam

**CLI successor path: PRESENT (fixed).** `cmd_file_receive` routes the channel-path result
through `reconcile_expected_sha256(&s.sha256, expected_sha256, s.via)` at `file.rs:649` BEFORE
reporting success; a caller-supplied `--expected-sha256` mismatch is a loud non-zero exit
(same T-2472 seam as the legacy fallbacks at file.rs:849 and 966). The successor path therefore
CAN detect "I got a different artifact than the one I wanted." Not the OBS-108 false-green.

**MCP `termlink_file_receive`: ABSENT (still false-green).** `FileReceiveParams` (tools.rs)
carries only `target` + `output_dir` — no receiver-supplied expected digest. Its check
(tools.rs:13914-13948) sources `expected_sha256` from the SENDER's `file.complete` event
(`complete.sha256`) and compares it to `actual_sha256` recomputed from the same received bytes —
the exact self-consistent tautology OBS-108 names. It reports `ok:true` on whatever artifact it
reassembles. This is the legacy events path (not the channel successor), so it is the *legacy MCP
twin that never received the T-2472 fix*, not the successor inheriting it.

## Wrong-selection analogue (bug #1)

Addressed on the successor path. `select_newest_artifact` (`file.rs:483`) picks
`artifacts.iter().max_by_key(|a| a.channel_offset)` — newest offset, not `first()`. A regression
test (file.rs:1099) documents the OLD behavior was `artifacts.first()` (oldest) and guards against
its return. So the "stuck on oldest / first()" recurrence is closed for the channel path.
(The MCP twin selects by `transfer_id`, a different model, so bug #1 does not apply there.)

## Verdict + fix sizing

- **(a) Does the successor path inherit the OBS-108 false-green?** **NO.** The channel
  content-addressed receive path (CLI `file receive` T-1250/T-2473) carries both the transit
  check (artifact.rs:534) and the receiver-expected-digest seam (reconcile at file.rs:649), and
  its wrong-selection analogue is fixed (select_newest_artifact).
- **(b) Where a fix would go (for the real remaining gap):** the MCP `termlink_file_receive` tool
  — add an optional `expected_sha256: Option<String>` to `FileReceiveParams` and a
  receiver-vs-actual compare at `crates/termlink-mcp/src/tools.rs:13944` (replace the
  sender-sourced `expected_sha256` tautology with a caller-supplied compare, mirroring
  `reconcile_expected_sha256`). Small: ~1 struct field + ~10 lines. This is a legacy-surface
  parity fix, not a successor defect.
- **(c) Bug-#1 analogue present on successor path?** No — already fixed (max_by_key offset).
- **(d) G-089:** **Can be closed** on its stated condition — the successor path does NOT inherit
  the seam. Recommend closing G-089 with a note, and filing a SEPARATE follow-up task for the MCP
  `termlink_file_receive` OBS-108 tautology (independent receiver surface, one-bug-one-task).
