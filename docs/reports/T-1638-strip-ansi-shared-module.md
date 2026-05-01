# T-1638 — extract `strip_ansi_codes` to shared `ansi` module

**Local task ID:** T-1437
**Workflow type:** refactor
**Date:** 2026-05-01
**Crate touched:** `termlink-session` (only)
**Behavior change:** none (pure extraction)

## Problem

Two byte-identical copies of `fn strip_ansi_codes(s: &str) -> String` lived in
`crates/termlink-session/src/`:

| File | Pre-refactor lines |
|------|--------------------|
| `handler.rs` | 374–415 |
| `governance_subscriber.rs` | 121–162 |

Display-side ANSI stripping (`handler.rs`, gated on the `command.execute`
`strip_ansi` param) and governance pattern matching (`governance_subscriber.rs`,
ANSI-stripped text fed into regex rules) shared an algorithm by copy-paste.
Drift between the two would silently desynchronise what the user sees from
what governance rules score against.

## Byte-identity confirmation

```
$ diff <(sed -n '374,415p' crates/termlink-session/src/handler.rs) \
       <(sed -n '121,162p' crates/termlink-session/src/governance_subscriber.rs)
$ echo $?
0
```

Body diff is empty. The only textual difference between the two definitions
was the leading doc-comment: `handler.rs` had a one-liner; `governance_subscriber.rs`
had a three-line variant naming the CSI / OSC / bare-ESC cases. The richer
doc-comment was preserved in the new shared module.

## Refactor

- Added `crates/termlink-session/src/ansi.rs` with one function:
  `pub(crate) fn strip_ansi_codes(s: &str) -> String`. `pub(crate)` because both
  callers live in the same crate; nothing outside `termlink-session` needs it.
- Added `pub(crate) mod ansi;` to `crates/termlink-session/src/lib.rs`.
- `handler.rs`: removed the private definition (former lines 373–415) and
  retargeted the single call site to `crate::ansi::strip_ansi_codes(...)`.
- `governance_subscriber.rs`: removed the private definition (former lines
  117–162) and retargeted the single call site to
  `crate::ansi::strip_ansi_codes(...)`.
- All `strip_ansi_*` unit tests from both files were moved into
  `ansi::tests` verbatim. The two test sets did not collide (handler had 7
  tests, governance had 4 with distinct names — total 11 preserved).

No other crate was touched; `Cargo.toml` was not modified; no new dependency
was introduced.

## Verification

| Command | Exit | Output (last line) |
|---------|------|--------------------|
| `cargo check -p termlink-session` | `0` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s)` |
| `cargo test -p termlink-session --lib` | `0` | `test result: ok. 316 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |

### Test count delta

| Phase | Lib tests passing |
|-------|-------------------|
| Pre-refactor baseline | 316 |
| Post-refactor | 316 |
| **Delta** | **0** |

(The dispatch instruction quoted a 250-test baseline; the actual current
baseline on `main` is 316. The refactor preserved that count exactly.)

## Diff summary

```
crates/termlink-session/src/ansi.rs                | 130 ++++++++++++++ (new)
crates/termlink-session/src/lib.rs                 |   1 +
crates/termlink-session/src/handler.rs             | -113 (fn + 7 tests removed; 1 call site updated)
crates/termlink-session/src/governance_subscriber.rs| -50  (fn + 4 tests removed; 1 call site updated)
```

## Risk assessment

Pure extraction. Function body is byte-identical to both removed copies
(empty `diff`). Tests preserved 1:1. No public surface change. No call
sites outside the two updated lines. No new transitive code paths.
