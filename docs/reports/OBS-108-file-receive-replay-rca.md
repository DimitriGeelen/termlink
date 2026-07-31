# OBS-108 — `termlink file receive --replay` RCA

**Date:** 2026-07-31
**Type:** Read-only root-cause analysis + remediation design (no source edits)
**Primitive:** legacy `file send` / `file receive` (T-1164 artifact-migration path)
**Key files:**
- `crates/termlink-cli/src/commands/file.rs` (CLI verbs `cmd_file_send`, `cmd_file_receive`, `try_recv_via_artifact`, `process_artifact_batch`)
- `crates/termlink-session/src/artifact.rs` (`recv_artifacts_via_client`, `download_artifact_via_client`)

---

## Symptom

A peer sent two repaired fixture files via `termlink file send`. `termlink file receive --replay` was run to re-fetch them. Two distinct failures:

1. **Wrong-transfer served (routing/cursor bug):** Every `--replay` invocation re-serves the *same earliest historical* transfer (`escalation-patterns.yaml`, months old). Four separate calls each returned the identical old file. Newer transfers are unreachable through `--replay`.
2. **Tautological digest verify (false-green, MORE SEVERE):** Every call prints `SHA-256 verified` and exits 0. The digest is "correct" for the (wrong, old) file it served. The check structurally cannot detect "served the wrong transfer."

## Reproduction

`termlink file receive <self> --replay` against a hub whose `inbox:<self>` topic already holds ≥1 prior artifact envelope. On the new (T-1250) artifact path:
- `try_recv_via_artifact` subscribes from **cursor 0** and calls `process_artifact_batch`, which returns `artifacts.first()`.
- The oldest envelope in the topic is written to disk; `SHA-256 verified: <hash>` is printed; exit 0.
- Re-running produces the identical result (no cursor is persisted; replay always restarts at 0).

---

## Root Cause — Bug #1 (wrong-transfer served)

**Location:** `crates/termlink-cli/src/commands/file.rs:435-445` (replay branch of `try_recv_via_artifact`) and `file.rs:484` (`process_artifact_batch`).

The replay path:

```rust
// file.rs:432 — initial subscribe starts at offset 0 (whole history)
let initial = recv_artifacts_via_client(&mut client, &host_port, target, 0, &cache, &mut ctx).await?;
let mut cursor = match initial {
    RecvOutcome::Received { artifacts, next_cursor } => {
        if replay
            && let Some(s) = process_artifact_batch(&mut client, &artifacts, out_path).await?
        {
            return Ok(Some(s));   // returns first artifact of the ENTIRE history
        }
        next_cursor
    }
    ...
};
```

`process_artifact_batch` then blindly selects the oldest:

```rust
// file.rs:484
if let Some(a) = artifacts.first() {   // <-- always the earliest envelope
```

Precise cause: **`--replay` subscribes from `cursor = 0` and `process_artifact_batch` unconditionally takes `artifacts.first()`.** `recv_artifacts_via_client` returns the topic's envelopes in ascending-offset order (oldest first, `limit: 1000`, `artifact.rs:397-435`), so `.first()` is deterministically the oldest transfer ever posted to `inbox:<self>`. There is:
- no persisted per-target receive cursor across invocations (each call restarts at 0),
- no transfer-id / offset / `--since` selector,
- no "serve newest" ordering.

So replay is hardwired to the single oldest envelope; every later transfer is permanently unreachable through this verb. `escalation-patterns.yaml` was simply the first artifact posted to that inbox topic.

This is a **selection/addressing defect** — the artifact-based file-receive has no addressable-transfer model.

---

## Root Cause — Bug #2 (tautological verify / false-green)

**Location:** `crates/termlink-cli/src/commands/file.rs:485-521` (`process_artifact_batch`) + the "verified" print at `file.rs:597`; underlying self-referential compare at `crates/termlink-session/src/artifact.rs:534-539` (`download_artifact_via_client`).

Two sub-paths, both self-referential:

**Chunked path** (`file.rs:485-513`): the digest checked is the artifact's *own* `artifact_ref`, which the sender stored alongside the same envelope:

```rust
let (bytes, sha256_hex, ...) = if let Some(sha) = &a.artifact_ref {   // sha == stored ref
    ...
    let bytes = download_artifact_via_client(client, sha).await?;      // verifies bytes hash == sha
    (bytes, sha.clone(), ...)
```

and inside `download_artifact_via_client` (`artifact.rs:534-539`):

```rust
let got = hex_sha256(&out);
if got != sha256 {                        // sha256 == the ref we asked for
    return Err(...sha256 mismatch...);
}
```

This proves only "the bytes I downloaded match the ref stored *with* them" — a content-addressed store integrity check — **not** "these are the bytes the caller wanted."

**Inline path** (`file.rs:514-521`): even weaker — it just hashes the received bytes and reports that hash. No comparison at all:

```rust
} else {
    let mut h = Sha256::new();
    h.update(&a.payload);
    let computed = format!("{:x}", h.finalize());   // hash of whatever arrived
    ...
    (a.payload.clone(), computed, filename, "channel.inline")
};
```

Then `cmd_file_receive` prints the label unconditionally on success:

```rust
// file.rs:597
eprintln!("SHA-256 verified: {}", s.sha256);
```

`s.sha256` is either (a) the artifact_ref the bytes trivially match, or (b) the freshly-computed hash of the received bytes. In neither case is it compared to an **independent expected digest** supplied by the caller or the sender's out-of-band manifest. The "SHA-256 verified:" line is a fixed success label — a tautology. Right digest, wrong file, exit 0.

**Contrast — the legacy event path does a real (weaker-but-genuine) check.** `file.rs:751-758` and `847-860` compare the reassembled bytes against `complete.sha256` carried in a *separate* `file.complete` envelope. That is the sender's self-declared digest transmitted independently of the chunk bytes, so it can at least catch in-transit corruption. The new artifact path removed even that, replacing it with a self-referential compare.

This is an **integrity-contract defect** — no end-to-end digest check against caller expectation.

---

## Shared-or-distinct

**Two independent bugs → two tasks** (per "one bug = one task").

- Bug #1 is *selection* logic (`.first()` + cursor-0, no addressing).
- Bug #2 is *verification* logic (self-referential digest).

Fixing one does not fix the other: correct selection still leaves a tautological verify; a correct verify still always serves the oldest.

They share a **common structural theme** — the T-1164 artifact-receive path has neither an addressable-transfer model nor an end-to-end integrity contract — and they **interact**: bug #2 is what makes bug #1 *silent*. If verify compared against a caller-supplied expected digest, serving the wrong transfer would fail loudly (sha mismatch). So bug #2 masks bug #1; fixing bug #2 alone would at least convert the wrong-transfer failure from false-green to a loud error. Bug #2 is therefore the higher-priority fix.

---

## Remediation Proposal (design only)

**Bug #1 — addressable replay.** Give `file receive --replay` a selection model. Minimal:
- Add a selector flag: `--transfer-id <id>` and/or `--offset <n>` / `--since <cursor>`, threaded through `try_recv_via_artifact` → `process_artifact_batch` so the batch is *filtered/selected* rather than blind `.first()`.
- Change default replay ordering to serve the **newest** matching artifact (`.last()` / max-offset) — the intuitive meaning of "re-fetch what was just sent."
- Optionally persist a per-target receive cursor so successive `--replay` calls advance instead of restarting at 0.
- **Signature change:** new CLI flag(s) on `file receive`; new selector param on the two helper fns.

**Bug #2 — real integrity check (load-bearing, do regardless of retirement).**
- Add `--expected-sha256 <hex>` to `file receive` (sourced from the sender's manifest / the known digest of the repaired fixture). Thread it into `process_artifact_batch` and **compare against the served bytes; FAIL loudly (non-zero exit) on mismatch.**
- When no expected digest is supplied, **downgrade the wording** from `SHA-256 verified:` to `SHA-256 computed:` — the tool must not claim verification it did not perform.
- **Signature change:** new CLI flag; new expected-digest param on the helper.

---

## Structural / Inception Assessment (incl. T-1166 interaction)

**Is a point-fix enough?** For the *severity* of bug #2, yes — the false-green must be neutralized immediately (at minimum, relabel `verified`→`computed` and add loud-fail on a supplied expected digest). But the pair reveals a genuine structural gap: the file-transfer primitive has **no addressable-transfer identity and no end-to-end integrity contract** against caller intent. That is inception-shaped *if* the primitive were staying.

**T-1166 interaction — this changes the remediation target.**
- `T-1166` ("Retire legacy event.broadcast + inbox + file.send/receive primitives") is **`started-work`**, `workflow_type: decommission`, `owner: agent`, `horizon: now`; a cut-flip was projected for 2026-05-10 (`docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md`). `T-1415` ("post-cut cleanup: delete retired primitive handlers + fallback paths") is also `started-work`.
- The code already emits deprecation warnings: `cmd_file_send` → `print_deprecation_warning("file send", "channel post --file")` (`file.rs:161`); `cmd_file_receive` → `("file receive", "channel subscribe")` (`file.rs:544`). The blessed successor is **`channel post --file` / `channel subscribe`**, which is addressable (offset/cursor) and content-addressed by design.

**Verdict:** Do **not** open an inception to build an addressable-transfer model *into the retiring primitive.* The right structural move is **accelerate retirement / redirect users to the channel path**, aligned with T-1166/T-1415. But retirement is gated and in-progress, so the primitive is still live and shipping false-green integrity results today. Therefore:
1. **Immediately (point-fix, not inception):** kill the false-green in the deprecated verb — relabel `verified`→`computed` and add loud-fail-on-mismatch when `--expected-sha256` is provided. A false-green integrity check is a data-integrity hazard regardless of deprecation status; leaving a tautological "verified" on a live command is unacceptable even for a retiring primitive.
2. **Bug #1:** the minimal `.first()`→newest / add-selector fix is worth it only if T-1166 cut is not imminent; otherwise the primary remediation is to point OBS-108's use case at `channel subscribe --cursor …` and let retirement remove the buggy path. Confirm current T-1166 cut status before investing in the selector.

**Inception needed: NO.** Both are bounded point-fixes on a deprecated primitive; the "structural" answer is the already-planned T-1166 retirement, not a new exploration.

---

## Existing-registration check

Searched `.context/project/{gaps,concerns,learnings}.yaml`, `.context/observations/`, `.context/audits/discoveries/`.

- **OBS-108 is NOT registered anywhere.** No gap/concern/learning describes the replay-serves-earliest or tautological-verify defects in the artifact path.
- Related-but-distinct prior art (do not conflate):
  - `learnings.yaml:150,166` — legacy *event* path picking up stale/30-min-old `FileInit` events (the T-1018 fresh-vs-replay design), not the artifact path.
  - `learnings.yaml:2546` — MCP `file_send` transfer_id per-PID collision (chunk-stream blend), a distinct send-side bug.
  - `concerns.yaml:590` — self-audit guard for calls to retired verbs (`file.send|receive`), the T-1166 decommission guard.
  - `learnings PL-246` (from T-2363) — divergent send-file code paths use different hub RPC verbs (legacy 3-phase vs artifact); adjacent to bug #1's dual-path fragility.
- **Recommendation:** register OBS-108 as a new gap (data-integrity: false-green digest on a live deprecated verb) and/or a learning ("self-referential digest compare is not verification — verify against an independent caller/manifest digest, and never print `verified` for a `computed` hash"). Two separate build tasks (bug #1 selection, bug #2 integrity) per one-bug-one-task.
