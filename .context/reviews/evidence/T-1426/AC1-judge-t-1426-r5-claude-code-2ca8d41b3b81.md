# T-1426 Human AC#1 — deprecation warning informative, not noisy

Seat: rung-5-panel:claude-code. Revision 7dedae3804b7c8cbf538d2714d61f6ee3961de7c. Date 2026-11-15.

## Build
`cargo build --release -p termlink` at the reviewed revision, with `CARGO_TARGET_DIR=/tmp/t1426-target` so the repo's
own `target/` (binary dated Nov 3, older than the last CLI source change of Nov 14) was not touched.
`Finished release profile [optimized] in 13m 16s`, exit 0. `termlink --version` → `0.12.13+69.g7dedae380`.
No uncommitted changes under crates/, Cargo.toml, Cargo.lock.

## Code read
- Helper `print_deprecation_warning` (crates/termlink-cli/src/commands/mod.rs:76): returns early only when
  `TERMLINK_NO_DEPRECATION_WARN` is exactly `1`; otherwise one `eprintln!` line.
- Seven call sites, each the first statement of its command function: push.rs:62 (remote push), events.rs:281
  (event broadcast), file.rs:187 (file send), file.rs:670 (file receive), infrastructure.rs:1078/1127/1174
  (inbox status / clear / list).
- Each of those functions is called only from the dispatch in main.rs (lines 194, 227, 726, 729, 782, 800-802).
  None calls another warned function, and `cmd_push` warns outside its inner retried body, so there is no
  double-warning path. The hidden top-level `broadcast` alias (main.rs:227) goes through `cmd_broadcast` once.

## Step 2 as written
| Command | Result |
|---|---|
| `remote push x --message x` | `error: the following required arguments were not provided: <SESSION>` — 0 DEPRECATED lines |
| `event broadcast topic-x payload` | `error: unexpected argument 'payload' found` — 0 DEPRECATED lines |
| `file send target /tmp/nonexistent` | `[DEPRECATED] termlink file send — use 'termlink artifact put' instead. See T-1166.` — 1 |
| `inbox status` | `[DEPRECATED] termlink inbox status — use 'termlink channel info' instead. See T-1166.` — 1 |
| `inbox list ring20-management-agent` | `[DEPRECATED] termlink inbox list — use 'termlink channel subscribe' instead. See T-1166.` — 1 |
| `inbox clear ring20-management-agent` | `[DEPRECATED] termlink inbox clear — use 'termlink channel subscribe --cursor' instead. See T-1166.` — 1 |

Two of the six commands are rejected by argument parsing before the command function runs, so they never show a
warning. `inbox clear` was run with `TERMLINK_RUNTIME_DIR=/tmp/t1426-rt` (empty dir): as written it would clear
transfers on the live hub, which a read-only reviewer must not do. It fails fast after the warning.

## Step 4 as written
`TERMLINK_NO_DEPRECATION_WARN=1 termlink remote push x --message x 2>&1 | grep -c DEPRECATED` → `0`.
This is vacuous: it prints 0 without the flag too, because of the parse error above.

## Same checks with arguments that reach the helper
| Command | DEPRECATED line | count |
|---|---|---|
| `remote push 127.0.0.1:1 x --message x --secret <64 zeros> --timeout 2` | `remote push — use 'termlink channel post'` | 1 |
| `event broadcast topic-x --payload notjson` | `event broadcast — use 'termlink channel post'` | 1 |
| `event broadcast topic-x --payload notjson --targets nosuch` | `event broadcast — use 'termlink event emit_to'` | 1 |
| `broadcast topic-x --payload notjson` (hidden alias) | `event broadcast — use 'termlink channel post'` | 1 |
| `file send target /tmp/nonexistent` | `file send — use 'termlink artifact put'` | 1 |
| `file receive nosuch-target-zz --timeout 2` | `file receive — use 'termlink artifact get'` | 1 |
| `inbox status` | `inbox status — use 'termlink channel info'` | 1 |
| `inbox list ring20-management-agent` | `inbox list — use 'termlink channel subscribe'` | 1 |
| `inbox clear ring20-management-agent` | `inbox clear — use 'termlink channel subscribe --cursor'` | 1 |

Seven distinct legacy verbs, each exactly one line, the warning first on stderr.

Suppression (`TERMLINK_NO_DEPRECATION_WARN=1`) on the helper-reaching form of all seven verbs: 0 lines each.

## Do the cited replacements exist?
`<verb> --help` on the fresh binary: `channel post`, `artifact put`, `artifact get`, `channel info`,
`channel subscribe` (has `--cursor`) all exist.
`termlink event emit_to --help` → `error: unrecognized subcommand 'emit_to'`. The verb is `event emit-to` (exists).
So the targeted `event broadcast --targets ...` warning tells the user to run a command that does not exist
(events.rs:280).

## Verdict: amber
Met: one line per verb, seven verbs, no double warning, suppression works, six of seven verbs cite an existing
replacement (file send / file receive now point at artifact put / get, which exist).
Not met:
1. `event broadcast` with `--targets` cites `termlink event emit_to`; the real verb is `event emit-to`.
2. The criterion's step 2 and step 4 literals are out of date: `remote push x --message x` and
   `event broadcast topic-x payload` no longer parse, so they show no warning and the suppression check proves nothing.
   Step 2 also lists six commands while Expected says seven verbs (`file receive` is missing).

## Needed
- events.rs:280: `"event emit_to"` → `"event emit-to"`.
- Operator: accept or reject the restatement in the verdict guidance.
