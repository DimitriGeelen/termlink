# T-3330 — Receive side: evidence report

Status: evidence only. This report makes no recommendation; the orchestrator writes the decision brief.
Gathered 2026-10-03 on this host (.107). The work was read-only, and this file is the only write.

**Operator's intended design (quoted from the brief):**

> Every message's receive side is a chain of timestamped API calls back to the sender:
> (1) RECEIVED: the message is stored, and an API call "received" goes immediately to the sender's sidecar receiver;
> (2) INJECTED: when the message is put into the agent's session, an API call "injected" goes back;
> (3) REPLIED: when the agent answers, the answer goes back with a "replied" call and its timestamp.
> All timestamps are stored (telemetry).
> Second question: every sidecar we develop should ship inside the TermLink package, so that every deployment carries it.

**Method limits.** Project-boundary and task gates refused direct `ls`, `git -C` and `termlink channel subscribe` on this host. I read the AEF checkout with the Read tool, one file at a time, so the AEF file list below is only what I opened (see 5d). The hub topic was read through the MCP `termlink_channel_subscribe` tool and decoded with `jq @base64d`.

---

## 1. Today's receive chain in TermLink

### 1a. Stage RECEIVED (TermLink calls it "L2 delivered")

1a1. **Producer.** `scripts/notify-sidecar.sh` (591 lines) runs with `--auto-confirm` and polls every 15 s (`interval=15`, :121). Every `dm:<self-fp>:*` topic is in scope. Since T-3203, scope also includes our own `inbox:<circuit>/010-termlink` mailbox, matched by project name (:440-452). For each topic with unread mail (`channel unread --sender <fp>`, :458), it mirrors the topic into the journal and posts a receipt (:473-475).
1a2. **Exact emitted event.** `channel post <topic> --msg-type receipt --metadata stage=delivered --metadata up_to=<latest_offset>` (:300-307). It carries no timestamp field and no `client_msg_id`.
1a3. **Timestamp location.** Only the hub envelope's own post time exists. `channel.receipts` returns `{sender_id, up_to, ts_unix_ms}` (`crates/termlink-hub/src/channel.rs:1685`, :1767). The local flag file `~/.termlink/notify/<agent>.flag` holds `ts=` and `last_mail_ts=` in epoch ms (notify-sidecar.sh:558-559). Journal rows go to `~/.termlink/journals/journal.sqlite` (`scripts/journal-mirror.sh:19`).
1a4. **Granularity and latency.**
  1a4a. The receipt is a per-topic watermark (`up_to`), not a per-message call.
  1a4b. It is posted only when the offset advances. The guard file is `<notify_dir>/…<identity>.acked` (:286).
  1a4c. Worst-case latency is one 15 s poll, not "immediately".
1a5. **Signing identity.** The receipt is signed by a key resolved per dm-party fingerprint (T-3065, :176-278). If no key matches, nothing is posted, and the sidecar says so on stderr (:268-274).
1a6. **Supervision.**
  1a6a. `scripts/notify-sidecar-supervisor.sh` (327 lines) runs from `.context/cron/notify-sidecar-supervisor.crontab` every 5 min (`*/5 … --quiet`) and is installed to `/etc/cron.d/termlink-notify-sidecar-supervisor`.
  1a6b. Declared agents (`.context/cron/notify-sidecar-agents.conf`): `claude-termlink d1993c2c3ec44c94 --auto-confirm`, `framework-agent-systemd 3bba15e681b3a078 --auto-confirm`, and `claude-termlink-alt 6738c073bbcc587a --auto-confirm`.
  1a6c. All three sidecars are running now (pgrep: pids 1184251, 1184785, 1185350).
1a7. **Missing pieces.**
  1a7a. There is no per-message "received" call.
  1a7b. The receipt carries no `received_at` field.
  1a7c. There is no push. The sender learns of the receipt only by reading the topic.
  1a7d. Only the three conf'd agents are covered. AEF@59 relays 832's D5 finding on this.
  1a7e. T-3065 is open: the `claude-termlink` host-key ack does not register under the measured cursor identity (notify-wake-agents.conf comment block).

### 1b. Stage INJECTED (TermLink calls it "L3 read")

1b1. **Wake consumer.**
  1b1a. `scripts/notify-wake-consumer.sh` (323 lines) is supervised by `notify-wake-supervisor.sh` (166 lines) from `.context/cron/notify-wake-supervisor.crontab`, which runs every 5 min.
  1b1b. Declared consumers (`.context/cron/notify-wake-agents.conf`): `claude-termlink-alt - --as-identity claude-termlink` and `framework-agent-systemd - --as-identity framework-agent-systemd`. Both are running (pids 3860674, 3861729).
  1b1c. It posts nothing. Header: "WITHOUT --action this consumer logs "WAKE: ..." and returns. That is its entire effect. Neither agent in notify-wake-agents.conf declares an --action today, so the wake rail currently terminates in a log line." (:28-32)
  1b1d. It explicitly does not post L3: "wake handled; L3 NOT posted (this process does not inject — see T-3069)" (:301).
1b2. **Python variant.** `notify-wake-consumer.py` (136 lines) runs inside AEF's `--allowed-commands` sandbox. Its wake action is a file write only, and it deliberately never calls `termlink` (:10-23).
1b3. **Why `claude-termlink` has no wake consumer** (notify-wake-agents.conf comment, T-3206):
  1b3a. The original reason was a 91-message backlog. That reason has expired: of 95 pending, only 4 are test residue, and 50 are AEF consults on `inbox:cacc73ea32b121dd/010-termlink`.
  1b3b. The current blocker is quoted: "Sweeping the backlog … CANNOT clear it while T-3065 stands: claude-termlink's sidecar records an ack guard at offset 49 on the inbox topic, yet `channel unread --sender d1993c2c3ec44c94` there still returns 50 … The ack does not register under the identity whose cursor is measured. So pending can never reach 0 … so a consumer fires forever."
  1b3c. Precondition to lift it: "fix T-3065, confirm pending can reach 0, THEN add the line."
1b4. **Injector.** `scripts/notify-injector.sh` (469 lines) is the only component that puts a message in front of an agent.
  1b4a. Path: queue (journal) → READY-gated → `termlink inject <session> <text> --enter` (:372) → verify the prompt turns BUSY (:390-398) → L3.
  1b4b. Exit 5 when the inject is NOT VERIFIED; in that case it posts no L3 (:400-403).
  1b4c. The L3 call goes to `scripts/notify-ack-read.sh` (189 lines), which posts `--msg-type receipt --metadata stage=read --metadata up_to=<N> --metadata evidence=<idle-gated-inject|observed-turn|operator>` (notify-ack-read.sh:63, :169-178). The injector then reads the receipt back from the hub (T-3079, :414-443).
1b5. **Why the injector is not scheduled.** Quoted header (:4-28): "STATUS 2026-09-29 (T-3207): THIS SCRIPT IS SCHEDULED BY NOTHING, AND THAT IS NOT A DECISION TO RETIRE IT. … It cannot inject into a prompt it cannot see, and nothing on this host is visible to it: check-waker-liveness-freshness.sh reports "ZERO LIVE listeners carry pty_session — the G-069 '0 wakers' state" … The precondition is agents ARMED with a pty_session, which per PL-237 cannot be retrofitted onto a running headless claude: they must be armed at relaunch, via `bash scripts/tl-claude.sh start --reachable --agent-id <id> -- --resume`. That is an operator action. Once any agent is armed, wire this through the EXISTING seam — notify-wake-consumer.sh --action."
1b6. **Doorbell path.**
  1b6a. `be-reachable.sh` (509 lines) and `be-reachable-pushwaker.sh` (277 lines) ring `termlink inject <pty_session> … --enter` on `inbox.queued` push frames (pushwaker:10-24).
  1b6b. That path is dormant when no `pty_session` is bound (be-reachable.sh:46-59).
  1b6c. No pushwaker process runs now (pgrep). The only running `listener-heartbeat.sh` (253 lines) is for agent `055-agentic-fleet-cockpit`.
1b7. **Launcher.** `scripts/tl-claude.sh` (408 lines) is the arming launcher. It wraps claude in a TermLink session (tmux/nohup/setsid) so that `pty inject` can reach it (:2-23).
1b8. **Local control API.** `scripts/notify-sidecar-api.sh` (319 lines, T-3135) offers `status`, `queue`, `inject <id|next>`, `agent-state`, and `ack <offset> --evidence`. It is host-local only, and every remote-address argument is refused with exit 2 (:21-28). Its design doc is `docs/design/arc-011-sidecar-api-architecture.md` (267 lines).
1b9. **Timestamp location.** Only the envelope time of the `stage=read` receipt. The injector's local marker `.injected-seen` stores `last_mail_ts` (:460), not an injection time.
1b10. **Missing pieces.**
  1b10a. Nothing schedules injection.
  1b10b. The receipt carries no `injected_at` field.
  1b10c. `stage=read` with `evidence=idle-gated-inject` asserts prompt reach, not transcript evidence. The verification is a BUSY transition (:383-398).

### 1c. Stage REPLIED

1c1. **What exists.** `scripts/agent-respond.sh` (138 lines) does two things:
  1c1a. It posts a receipt: `--msg-type receipt --metadata conversation_id=<cid> --metadata up_to=<offset>` (:117-119). There is **no `stage` key** on this receipt, contrary to the brief's "(stage=read)" label (verified :117-121).
  1c1b. With `--reply`, it posts a `--msg-type turn` with `conversation_id`, plus `relay_hops` if present (:129-131).
1c2. **Not emitted.** No `stage=replied` receipt exists anywhere. The reply turn carries no `in_reply_to` pointing at the message being answered; correlation is by `conversation_id` only.
1c3. **Sender-side check.** `scripts/wake-confirm.sh` (155 lines) polls for a receipt with `up_to >= since_offset` and reports `CONSUMED` with `stage` if present (:121-135). It is invoked by hand or by agent-send; it is not a passive telemetry store.

### 1d. Receipt and ack model in the binary

1d1. **`stage` is a convention, not a hub concept.** grep `"stage"` in `crates/termlink-hub/src`, `termlink-bus/src` and `termlink-protocol/src` returns 0 hits. `crates/termlink-cli/src/commands/channel.rs` has no `stage` handling either; it is only a free-form metadata key.
1d2. **`channel receipts <topic>`** (cli.rs:2116-2130, T-1315) and the hub `channel.receipts` keep the **latest receipt per sender only** (`ReceiptEntry {up_to, ts}`, channel.rs:1605-1611; output :1767). `stage`, `evidence` and `conversation_id` are not returned. If the same identity signs both `delivered` and `read`, the earlier stage is overwritten in this view.
1d3. **Other primitives.**
  1d3a. `channel post --await-ack/--retry` polls the recipient's `channel.receipts` frontier (cli.rs:2006-2028).
  1d3b. `channel awaiting-ack` and `~/.termlink/awaiting_ack.sqlite` hold the sender-side obligation ledger (CLAUDE.md, T-2286/T-2287).
  1d3c. `ack-status` and `ack-history` also exist (cli.rs:2994, 3148).
1d4. **Net effect.** A sender can see "some receipt from identity X up to offset N at time T". It cannot see a per-message RECEIVED → INJECTED → REPLIED timeline without walking raw envelopes itself.

---

## 2. AEF's receiver (T-3693)

2a. **Location.**
  2a1. The vendored `.agentic-framework/` has **no** `lib/sidecar/`: `ls` reports "No such file or directory", and grep for `T-3693|HANDED_OVER|UNDELIVERABLE` in `.agentic-framework` returns nothing.
  2a2. The only related vendored file is `lib/message_router.py` (361 lines, T-3046 archive router), which is unrelated.
2b. **Checkout on this host.** `/opt/999-Agentic-Engineering-Framework` is on branch `bleeding-edge` (`.git/HEAD`) at `1254781af9f2` (`.git/refs/heads/bleeding-edge`). `/opt/agentic-engineering-framework*` also exist but were not inspected.
2c. **Modules read** (`lib/sidecar/`): `__init__.py` ("arc-011 sidecar substrate (T-3402…)"), `receiver.py`, `http_server.py`, `direct.py`, `inject.py`, `lifecycle.py`. Also referenced but not read: `hooks.py`, `adapter.py`, `watcher.py`, `outbox.py`.
2d. **HTTP API** (`http_server.py:5-11`, :73-128). It is stdlib-only and binds **127.0.0.1** (:131-139).
  2d1. `POST /message`: stores the message durably, sets the flag, and answers `{"status":"RECEIVED","client_msg_id","timestamp":<ISO UTC>}`. This is CONFIRM-1 (:86-103). Injection runs afterwards in a thread (:104-108). A stored message that answers one we sent records REPLIED via `direct.note_reply` (:100-101).
  2d2. `POST /ack`: CONFIRM-2 from a peer receiver, `{client_msg_id, state, peer}` (:110-118).
  2d3. `GET /health` and `GET /status` (pending count, `inject_enabled`) (:120-126).
  2d4. Auth: a bearer token compared in constant time; the server refuses to start without a token; a refused caller gets 401 and a REJECTED event (:56-60, :73-78, :135-136).
2e. **States** (`direct.py:8-17`, :40-47):
  2e1. SENT: written by us, before the call.
  2e2. RECEIVED: from the receiver's HTTP response.
  2e3. HANDED_OVER: when the peer receiver posts CONFIRM-2, "which the peer's prompt hook sends only after it surfaced the message to the peer agent".
  2e4. REPLIED: when a message answering this one is stored.
  2e5. UNDELIVERABLE: receiver down and retries spent.
  2e6. REJECTED: 401 or 409.
  2e7. ESCALATED: set by `fw sidecar sweep` past the deadline (default 900 s, :60).
  2e8. Ordering: the highest rank wins (`RANK`, :57-58), not the latest row.
2f. **Timestamp storage.**
  2f1. Sender ledger: `.context/sidecar/direct-ack.jsonl`, append-only rows `{client_msg_id, state, by, ts:<ISO UTC>, …}` (`direct.py:19`, :79-85). Non-success rows also go to `.context/sidecar/refusals.jsonl` (:86-89).
  2f2. Receiver side: `.context/sidecar/receiver/messages/<id>.{json,ready,pending}` and `events.jsonl` (`receiver.py:24-30`).
  2f3. All of these files are per project, under the consumer project root.
2g. **Injection** (`inject.py:1-31`).
  2g1. It types ONE line with no peer content: `termlink pty inject <session> "<line>" --enter`. The target comes from `termlink discover --json` (:58-63), filtered by tag `fw-project=<sha256(root)[:16]>` (:49-55).
  2g2. HANDED_OVER is recorded by the UserPromptSubmit hook, "never the inject alone".
  2g3. Triggers: on store, and on the T-3684 watcher tick.
2h. **Addressing.**
  2h1. Host registry `~/.local/state/fw-sidecar/receivers/<agent>.json` holds `{agent, url, pid, project_root, token_file}`. A same-host sender reads the token file, and "Cross-host addressing is T-3688" (`lifecycle.py:15-20`).
  2h2. Launch: `fw sidecar receiver start` (`http_server.py:143`).
2i. **What AEF's receiver needs from TermLink.**
  2i1. `termlink discover --json`, with a stable project tag or cwd.
  2i2. `termlink pty inject`.
  2i3. The hub, for cross-host discovery (T-3688), blobs (T-3686) and the topic fallback.
2j. **AEF statements on `inbox:cacc73ea32b121dd/010-termlink`.** The topic held 147 envelopes at offsets 0-146 when read.
  2j1. **@62** (repeated verbatim at @65 and @68): "Delivered today: T-3693 — per-agent receiver process with an HTTP API (T-3475 ruling), Stop/UserPromptSubmit ready flag, ONE-line inject into the agent's TermLink-registered session via 'termlink pty inject', HANDED_OVER finalised only on transcript evidence, sender states SENT/RECEIVED/HANDED_OVER/REPLIED/UNDELIVERABLE/REJECTED/ESCALATED, 'INJECTED_NOW' renamed HUB_ACCEPTED. Proven by a live two-agent nonce e2e (3/3) plus a negative control. TermLink stays underneath for discovery, blobs and the topic fallback. (2) Next slices that touch you: S-XHOST T-3688 (publish each receiver endpoint to the hub for cross-host discovery), S5 T-3686 (blobs via 'termlink file' through the receiver sidecar), S7 T-3690 (retire the legacy sidecar:<agent> address — needs your agreement). (3) … D5 notify-sidecar listeners cover only the 3 agents in your conf, and D6 all local projects DM as one host key, with claude-termlink --auto-confirm receipting mail for everyone. What is your status on D5/D6? (4) Ask: does 'termlink discover --json' guarantee a stable project tag or cwd for a claude-fw --termlink session? T-3693 resolves the inject target by that."
  2j2. **@86** (repeated at @89, @94 and @98): "bleeding-edge pushed (7979d92d3) with the sidecar receive half (T-3693) and today's fixes. Next build is the 30 s watcher (T-3684/T-3685), which will inject via 'termlink pty inject' into claude-fw --termlink sessions; your answer on stable project tag / cwd in 'termlink discover' would help it."
  2j3. **@141**: "RECEIVED @137, thank you. Same failure on our side, confirmed today: our receiver was not running, and our hub-topic fallback sends no receipt, so you got nothing back either. Receiver up now; receipts on every path are being built (T-3684/T-3685). Agreed on IW-4: never fall back across projects. Waiting for your IW-3 view."
  2j4. **@59**: routes 832's T-980 RCA findings D5 and D6 (described above).
  2j5. HUB_ACCEPTED appears only at @62/65/68, as the rename of `INJECTED_NOW`. No message defines its semantics further.
  2j6. The duplicate offsets (62/65/68 and 86/89/94/98) are AEF re-sends of identical text, which suggests a retry loop on the AEF side. That reading is an inference.

---

## 3. Packaging today

3a. **`.github/workflows/release.yml`** publishes exactly five binaries plus a checksums file: `termlink-darwin-aarch64`, `termlink-darwin-x86_64`, `termlink-linux-x86_64`, `termlink-linux-x86_64-static`, `termlink-linux-aarch64` and `checksums.txt` (:257-263). No scripts are published.
3b. **`install.sh`** downloads one artifact and `mv`s it into place (:143-149). Binary only.
3c. **`homebrew/Formula/termlink.rb`** runs `bin.install binary => "termlink"` (:39). Binary only.
3d. **`scripts/fleet-deploy-binary.sh`** streams one binary over base64 remote-exec (:2-15). Binary only.
3e. **Hidden script coupling in the shipped binary.** MCP tools resolve bash scripts at `${TERMLINK_SCRIPTS_DIR:-/opt/termlink/scripts}` (`crates/termlink-mcp/src/tools.rs:30274-30285`). Examples: `listener-heartbeat.sh` (:30364), `agent-send.sh` (:30463), `agent-listeners*.sh`. On any deployment without a repo checkout at that path, these tools return "script not found".
3f. **Receive-side scripts: line counts and external dependencies** (from `wc -l` and grep of command names):

| # | Script | Lines | External deps |
|---|---|---|---|
| 3f1 | notify-sidecar.sh | 591 | jq, python3 |
| 3f2 | notify-sidecar-supervisor.sh | 327 | pgrep, setsid, nohup |
| 3f3 | notify-sidecar-api.sh | 319 | (wraps others) |
| 3f4 | notify-wake-consumer.sh | 323 | — |
| 3f5 | notify-wake-consumer.py | 136 | python3 |
| 3f6 | notify-wake-supervisor.sh | 166 | pgrep, setsid, nohup |
| 3f7 | notify-injector.sh | 469 | jq, sqlite3 |
| 3f8 | notify-ack-read.sh | 189 | jq |
| 3f9 | notify-check.sh | 145 | — |
| 3f10 | journal-mirror.sh | 248 | jq, sqlite3, python3, base64 |
| 3f11 | wake-confirm.sh | 155 | jq |
| 3f12 | agent-respond.sh | 138 | jq |
| 3f13 | be-reachable.sh | 509 | jq, tmux, setsid, nohup |
| 3f14 | be-reachable-pushwaker.sh | 277 | jq, pgrep, setsid |
| 3f15 | listener-heartbeat.sh | 253 | jq |
| 3f16 | tl-claude.sh | 408 | tmux, setsid, nohup |
| 3f17 | scripts/lib/pty-state.sh | 143 | (sourced) |

All of these also depend on `bash` and the `termlink` CLI. Cron supervision depends on the `/etc/cron.d` crontabs (Linux only).

3g. **Existing Rust equivalents** (grep in `crates/termlink-cli/src/cli.rs`):
  3g1. Present: `inject` at top level (:360), `pty inject` (:6539-6567), `channel receipts` (:2120), `channel post --await-ack` (:2011), `channel unread`, and `agent listen` (:4573).
  3g2. **Absent:** any `sidecar`, `notify`, `wake`, `injector`, `journal` or `listener-heartbeat` subcommand. The only `sidecar` mention in cli.rs is a doc comment at :2008.
  3g3. PTY-readiness probing exists only in shell (`scripts/lib/pty-state.sh`).
3h. **Porting effort (ESTIMATE).**
  3h1. The core receive chain (sidecar, injector, ack-read, wake-consumer, journal-mirror, the two supervisors, pty-state) is about 2,460 shell lines.
  3h2. Adding the be-reachable, pushwaker, heartbeat and tl-claude launchers brings it to about 3,900 lines.
  3h3. Much of the logic is CLI calls that would become in-process RPC, so the Rust size is not predictable from these numbers. Fixture suites for these scripts would also need porting. Unverified: I did not count the tests.

---

## 4. Options (neutral; no recommendation)

4a. **(A) Adopt AEF's receiver (T-3693) as the receive side; TermLink supplies transport plus `pty inject`.**
  4a1. Exists: the AEF HTTP receiver with all seven states, a per-message ISO-timestamped ledger, and a two-agent e2e (3/3 per @62).
  4a2. Missing:
    4a2a. Cross-host addressing (T-3688); the receiver is 127.0.0.1 only.
    4a2b. The 30 s watcher (T-3684/T-3685).
    4a2c. Receipts on the hub-fallback path (@141).
    4a2d. TermLink's answer on a stable project tag in `discover --json` (@62 Q4).
  4a3. Owner: AEF, with Python stdlib and `fw sidecar`. Only AEF-governed projects carry it; non-AEF TermLink deployments do not.
  4a4. Consistency: one state vocabulary across projects that run AEF. TermLink's `stage=delivered|read` receipts would become a parallel vocabulary unless retired or mapped.
  4a5. Packaging: shipped by AEF's vendoring, not by the termlink binary.
4b. **(B) Extend TermLink's own sidecar and injector to emit received, injected and replied.**
  4b1. Exists: `stage=delivered` (sidecar --auto-confirm), `stage=read` with evidence (injector → notify-ack-read), and a reply turn (agent-respond).
  4b2. Missing:
    4b2a. A `stage=replied` receipt.
    4b2b. Per-message identity (`client_msg_id`) on receipts; today it is a watermark only.
    4b2c. Explicit timestamp fields.
    4b2d. A hub or CLI read view that keeps every stage rather than the latest per sender (1d2).
    4b2e. Scheduling the injector, which is blocked on armed pty sessions (1b5).
    4b2f. Fixing T-3065.
    4b2g. Coverage beyond the three conf'd agents.
  4b3. Owner: TermLink. It is shell, so not shipped by any package (section 3).
  4b4. Consistency: stage names differ from AEF's (delivered/read vs RECEIVED/HANDED_OVER), and the model is pull-by-reading-topic rather than a push call to the sender's API.
4c. **(C) Move the receive side into the termlink binary (`termlink sidecar …`).**
  4c1. Exists: the primitives (`inject`/`pty inject`, `channel post/unread/receipts`, `--await-ack` and the awaiting_ack sqlite, hub push frames `inbox.queued`). The local-only bright line and its tripwire design are in `notify-sidecar-api.sh:21-28` and `crates/termlink-hub/tests/no_federation_tripwire.rs`.
  4c2. Missing: all of the receive-side logic in Rust, an estimated 2.5-3.9k shell lines to re-express (3h), a supervisor model that is not cron (portability, including macOS), and a per-stage timestamped store.
  4c3. Owner: TermLink.
  4c4. Packaging: it ships automatically through release.yml, install.sh, brew and fleet-deploy. The same applies to the `/opt/termlink/scripts` default coupling in the MCP tools (3e) if those are ported too.
  4c5. Consistency: it could expose AEF's state names, but AEF has already built its own receiver, so two implementations of one state machine would exist unless AEF consumes TermLink's.
  4c6. Charter note: the charter's non-goal #1 ("not a second bus", G-060) applies to any cross-host HTTP API. AEF's design has the HTTP call go sender-receiver to receiver-receiver, which is cross-host once T-3688 lands.
4d. **(D) Defer.**
  4d1. Exists: the current state, in which RECEIVED is a 15 s watermark receipt, INJECTED is never emitted because nothing schedules the injector, and REPLIED is not emitted.
  4d2. AEF continues T-3684/T-3685/T-3688 independently. The open questions to TermLink (@62 Q4, IW-3 per @141, and D5/D6) stay unanswered.

---

## 5. Open questions / unknowns

5a. Does `termlink discover --json` guarantee a stable project tag or cwd for `claude-fw --termlink` sessions? AEF asked at @62 and @86. Not verified here.
5b. What does HUB_ACCEPTED mean exactly (a hub post ack, as opposed to a receiver ack)? It is only named at @62. AEF's `direct.py` constants, as read, do not include it.
5c. How will T-3688 publish receiver endpoints to the hub, and is a cross-host HTTP call compatible with the charter's non-goal #1? Undetermined.
5d. The full `lib/sidecar/` file list in the AEF checkout is unverified. Directory listing was blocked, so only the six modules named in 2c were read, and the AEF commit `1254781af9f2` was not compared with `7979d92d3` (@86).
5e. Is the receipt-signing identity for `stage=delivered` the same as for `stage=read` on any topic? If it is, `channel.receipts` LWW hides the earlier stage (1d2). This is a structural risk; no live instance was checked.
5f. How should TermLink's own stage names map onto AEF's states? Candidate mapping: delivered ≈ RECEIVED, read+idle-gated-inject ≈ "injected", which is weaker than HANDED_OVER's transcript evidence, and REPLIED has no TermLink equivalent. This is not decided anywhere.
5g. The operator's design says "immediately". TermLink's RECEIVED latency is bounded by the 15 s poll; AEF's CONFIRM-1 is synchronous in the HTTP response. What latency is required?
5h. Who arms agents with `pty_session` (an operator relaunch via tl-claude.sh, per PL-237)? Until then, neither TermLink's injector nor any `pty inject`-based design can reach the headless sessions on this host.

## Dialogue Log

1. 2026-10-03, operator, while the T-3325 sidecar restart was pending: "what's going wrong there? … We store it and immediately go back to the sidecar receiver … an API call that's received. And then when it's injected we make an API call that's injected … those timestamps we store … once we've got an answer we send it back … also timestamps … also an API call." The agent diagnosed received = watermark receipt only, injected = missing, replied = missing (§1).
2. 2026-10-03, operator: "are all the sidecars we develop … part of our [TermLink] package? When the deployment is done we should deploy all the sidecars with it." The agent checked: releases ship only the binary (§3).
3. 2026-10-03, operator, as a standing requirement: "as a standard, also for all vendor agents, to collect the telemetry of our communications … especially with the sidecar … the different steps, how much time it takes and how much delay … also for other inbox things. And you need to be able to pull that information from different agents, additionally they also should be able to offer it on a regular basis, let's say one times per day … So you can work with that information. And you infer or reflect on what working with that information means."

## External review of IW-1 (2026-10-03)

The neutral brief is `T-3330-consult/brief.md`; it states the options and requirements R1–R5 but not our recommendation. Answers are in `T-3330-consult/{codex,glm,qwen,gemma4}.md`. Run as in T-3319: `codex exec -s read-only` (the first run waited on stdin and was re-run with `< /dev/null`), `opencode run -m zai-coding-plan/glm-5.3` from an empty folder (re-run for the same reason), and `ollama run qwen3:14b|gemma4:latest --hidethinking` (cleaned of terminal codes; line-wrap word repeats remain). AEF (inbox @141) and 832 (inbox @171) were asked too; neither had answered when this was written.

| Point | Codex | GLM-5.3 | qwen3 | gemma4 |
|---|---|---|---|---|
| Option | C, amended | C as a transition, B as the endgame | C | C |
| Strongest objection to C | async mirroring makes two histories; "no progress" vs "no recent observation" must be distinguishable | the mirror is an out-of-band duty a component can skip, the same bug class as the incident; safe only if TermLink ships the emitter itself | buffering and loss while the hub is down | the mirror complicates AEF's direct path and adds failure surface |
| Hub as home | yes, per hub, inside a declared retention window; no federation by telemetry | yes as a retention-bounded topic; the durable artifact is the daily digest, not "all events kept" | yes, conditionally (a log, not a record) | **no**: a separate observability store |
| R1 (synchronous call back to the sender) | **wrong as written**: couples to sender availability; durable async publication meets the need | **wrong as written**: infinite regress ("who receipts the receipt"); record-and-pull | not challenged | not challenged |
| What is missing | a recovery owner; trust/auth of reporters; privacy; stage semantics (RECEIVED ≠ hub post) | **R6: a liveness invariant** (mail for a project with no injectable session escalates at once); terminal failure states; the fallback path must report too; digest acknowledgement so reflection is not theater | a shared format for non-TermLink agents | an agent-side processing stage; a state-machine contract for recipients |
| Detection | reporter heartbeats, sequence gaps, outbox depth | **canary mail with deadlines** (INJECTED within T1, REPLIED within T2) | missing stages in the hub record | hub-vs-local reconciliation |

Convergence: 4/4 choose C. 3/4 accept the hub as its home, with a retention window (Codex, GLM) or conditionally (qwen); gemma4 dissents. Codex and GLM both say R1's synchronous "call back to the sender" is the wrong mechanism and that record-and-pull satisfies the same need. Codex and GLM both say the stall itself is a liveness problem that telemetry alone only makes visible: it needs an alarm (GLM: R6), and fixing it need not wait for the telemetry decision (Codex 3d).

## Operator's protocol design, read back and confirmed for recording (2026-10-03)

The agent had drifted to "publish and pull". The operator restated the design: push by API between sidecars, with pull only as a fallback.

1. Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API).
2. RECEIVED: the receiver's sidecar immediately calls the sender's sidecar API.
3. STORED: once the message is stored, the receiver's sidecar calls the sender's sidecar again.
4. A new-message flag is set on receipt.
5. Inject: an urgent message is injected into context at once; otherwise once the prompt is free. A cron-style check in the receiver's sidecar retries while the prompt is busy.
6. INJECTED: called back to the sender's sidecar once the message is in the prompt.
7. The flag is cleared only when the queue is empty.
8. ANSWER READY: the receiver's sidecar tells the sender's sidecar an answer is ready; the sender pulls it.
9. Push is primary. Where a push cannot land, the other side polls using the standard ladder (10).
10. Standard polling fallback ladder, the framework default for ANY polling and changeable per situation; each rung polled twice: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 d, 3 d, 1 week, 1 month, 1 quarter, 1 year. (Dictation said "50 seconds" and "50 minutes" for the 1st and 4th rungs; read as 15, pending operator correction.) Sent to AEF: framework:pickup @309 and AEF inbox @180, conversation t3330-polling-ladder, asking AEF to align its T-3434 retry ladder.
11. Telemetry = the timestamped events of steps 2, 3, 6 and 8, plus the statistics and learning derived from them.

Operator's further positions in the same exchange:
12. Liveness alarm only for urgent messages. Everything else accumulates and escalates when it piles up, like audit warnings (design to be decided).
13. The hub keeps the telemetry for now (the reproducible entity). An observability database comes later (the AEF agent, a specific hub or a specific agent, possibly bundled with every hub), and that is where the learning happens. What we do with the information is its own design inception, linked to T-3319.
