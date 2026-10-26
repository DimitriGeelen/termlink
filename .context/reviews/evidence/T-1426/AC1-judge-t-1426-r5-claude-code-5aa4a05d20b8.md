# T-1426 Human AC#1 — deprecation warning informative, not noisy

Reviewer seat: rung-5-panel:claude-code. Revision e42276945e4009f580dcc935e75ff21470d5888c. Date 2026-10-26.

## Build
`cargo build --release -p termlink` → `Finished release profile ... in 10m 31s` (exit 0). Binary target/release/termlink rebuilt 2026-10-26 22:54. No uncommitted changes under crates/ or Cargo.*.

## Code inspected
- Helper: crates/termlink-cli/src/commands/mod.rs:72 `print_deprecation_warning` — returns early only if `TERMLINK_NO_DEPRECATION_WARN == "1"`, otherwise prints one `eprintln!` line.
- 7 call sites, one per legacy verb, each the first statement of its cmd_* fn:
  events.rs:281 (event broadcast), push.rs:23 (remote push), file.rs:185 (file send), file.rs:661 (file receive),
  infrastructure.rs:1018/1067/1114 (inbox status/clear/list).
- No cmd_* fn calls another warned fn (grep of callers: only main.rs dispatch), so there is no double-warn path. The hidden top-level `broadcast` alias goes through the same cmd_broadcast and warns once.

## Step 2 as written: two commands never reach the helper
```
--- remote push x --message x ---
error: the following required arguments were not provided:  <SESSION>
--- event broadcast topic-x payload ---
error: unexpected argument 'payload' found
```
Clap rejects both before cmd_* runs. Step 4 (`TERMLINK_NO_DEPRECATION_WARN=1 ... remote push x --message x | grep -c DEPRECATED` → `0`) is therefore vacuous: it prints 0 because of the clap error, not because of suppression.
(`remote push x x --message x` also fails before the helper: "Hub profile 'x' not found".)
Note: step 2's `inbox clear ring20-management-agent` actually calls channel.trim on the live hub ("No transfers to clear" — a no-op this time).

## Rerun with arguments that reach the helper (fail fast after it)
| Command | stderr line | count |
|---|---|---|
| remote push 127.0.0.1:1 x --message x --secret 00..00 --timeout 2 | `[DEPRECATED] termlink remote push — use 'termlink channel post' instead. See T-1166.` then "Failed to connect to hub" | 1 |
| event broadcast topic-x --payload notjson | `... event broadcast — use 'termlink channel post' ...` | 1 |
| event broadcast topic-x --payload notjson --targets nosuch | `... event broadcast — use 'termlink event emit_to' ...` | 1 |
| broadcast (hidden alias) topic-x --payload notjson | `... event broadcast — use 'termlink channel post' ...` | 1 |
| file send target /tmp/nonexistent | `... file send — use 'termlink channel post --file' ...` | 1 |
| file receive nosuch-target-zz --timeout 2 | `... file receive — use 'termlink channel subscribe' ...` | 1 |
| inbox status | `... inbox status — use 'termlink channel info' ...` | 1 |
| inbox list ring20-management-agent | `... inbox list — use 'termlink channel subscribe' ...` | 1 |
| inbox clear ring20-management-agent | `... inbox clear — use 'termlink channel subscribe --cursor' ...` | 1 |

Suppression with TERMLINK_NO_DEPRECATION_WARN=1, on paths that do reach the helper: remote push (127.0.0.1:1) → 0, inbox status → 0, event broadcast → 0, file receive → 0. With `=0` → 1 (only "1" suppresses, as documented).

All seven verbs warn exactly once, on one line, and suppression works. **Noise criterion: met.**

## Informativeness: two replacement strings do not exist
1. **file send → `termlink channel post --file`**: `termlink channel post --help` has no `--file`, and `termlink channel post x --file /tmp/nonexistent` → `error: unexpected argument '--file' found`. Following the advice fails.
   Also, docs/migrations/T-1166-retire-legacy-primitives.md:153-156 says to *keep using* `termlink file send` / `file receive` because they are "already wired to the channel protocol" (T-1164). So the warning contradicts the migration guide.
2. **event broadcast --targets → `termlink event emit_to`**: the CLI verb is `emit-to`. `termlink event emit_to x y` → `error: unrecognized subcommand 'emit_to'  tip: some similar subcommands exist: 'emit', 'emit-to'`.

The other replacements exist: channel post, channel subscribe, channel info, channel subscribe --cursor.

## Verdict: amber
Not noisy (one line, no double warning, suppression works). But "citing the right replacement verb" fails for 2 of 9 warning variants (file send; targeted event broadcast), and the step-2/step-4 commands in the AC cannot show the result for remote push or event broadcast.

## Needed
- file.rs:185: point the file-send replacement at something that exists. Either add `channel post --file`, or (per the migration guide) decide whether `file send/receive` should be deprecated at all.
- events.rs:280: change `"event emit_to"` to `"event emit-to"`.
- Fix the AC steps so they reach the helper: `remote push 127.0.0.1:1 x --message x --secret <64 hex>` and `event broadcast topic-x --payload notjson`. Use those in the suppression check too.
