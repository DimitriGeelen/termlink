# T-3219 — triage of one-sided MCP/CLI twin changes

## Method

Input: `/tmp/t3219-hits.txt` (29 rows from `check-mcp-cli-twin-drift.sh`). For each row I ran
`git show <commit> -- crates/termlink-cli/src crates/termlink-mcp/src` and checked which function
the changed lines actually belong to. The script attributes each changed line to the nearest
preceding `fn`. That means doc comments, structs, enums and brand-new functions added just
*after* a twin get charged to that twin, which is the main source of MISATTRIBUTED rows. For rows
where the change really is inside the named function, I found both twins at HEAD
(`fn <name>(` in `crates/termlink-cli/src`, `fn <name>_mcp(` in `crates/termlink-mcp/src`) and
compared their logic. **DRIFT** is used only when the lagging twin lacks the behaviour at HEAD, the
difference is observable, and I verified it by reading the code. `to_json` / `to_json_mcp` is a
**false twin pair**: these are per-struct methods on unrelated row types (26 `to_json_mcp`
definitions in `tools.rs`), not mirrors of one another. Some MISATTRIBUTED or LEGIT rows sit next
to a **real** gap in a neighbouring function. Each one is marked **ADJACENT GAP**, is not counted
as DRIFT for its row, and is listed separately at the end. Rows 23–25 were already resolved by
T-3218 and were not re-investigated. No source was modified.

## Findings

| commit | twin | changed | class | evidence |
|---|---|---|---|---|
| 6bbbf8cc5 | aggregate_find_idle_entries | mcp | MISATTRIBUTED | Every added line comes after `aggregate_find_idle_entries_mcp` closes: new `QueueHistoryAggMcp` struct and `parse_queue_log_mcp` (T-2087 queue-history MCP parity). The twin body is untouched. |
| c262e43f9 | aggregate_queue_entries | mcp | MISATTRIBUTED | Added lines are a new `parse_substrate_log_mcp` fn and its doc comment, placed after `aggregate_queue_entries_mcp` (T-2117). The twin body is untouched. |
| c8fd00c39 | parse_find_idle_log | cli | DRIFT | CLI `parse_find_idle_log` gained a `kind_filter` param (`crates/termlink-cli/src/commands/agent_find_idle.rs:504`), and `agent find-idle-history --kind` echoes `kind_filter` in the JSON summary. MCP `parse_find_idle_log_mcp` (`crates/termlink-mcp/src/tools.rs:665`) takes only `agent_id_filter`, and `AgentFindIdleHistoryParams` (`tools.rs:11263`) has no `kind` field. So `termlink_agent_find_idle_history` cannot filter by `new`/`removed` and its summary has no `kind_filter`. The sibling `ChannelQueueHistoryParams` (`tools.rs:11284`) does have `kind`. |
| 857d4d27a | aggregate_find_idle_entries | mcp | MISATTRIBUTED | Added lines are a new `build_queue_status_exists_value` fn (T-2253 dead-letter parity), placed after the `QueueHistoryAggMcp` struct. The twin body is untouched. |
| 93dc836e5 | connect_remote_hub | mcp | DRIFT | MCP `connect_remote_hub_mcp` passes connect errors through `render_connect_error` (`tools.rs:7941`), so a TOFU/cert-drift rejection is shown verbatim, not as "is the hub running?". CLI `connect_remote_hub` still wraps every connect failure in `.context("Cannot connect to {hub} — is the hub running?")` (`crates/termlink-cli/src/commands/remote.rs:780`). The TOFU text survives only in anyhow's "Caused by:" chain, so the headline still points the operator at connectivity. Low severity: the information is present, but misframed. The auth-failure half of this commit is matched on the CLI by T-2625 (row 26). |
| 4546f4124 | connect_remote_hub | mcp | MISATTRIBUTED | Added lines are the new `ContactHub` enum and its impl (T-2274), placed after `connect_remote_hub_mcp` closes (`tools.rs:7776` onward). The twin body is untouched. |
| 0cf1cf91f | resolve_contact_via_fleet | cli | MISATTRIBUTED | Added lines are the new `FleetAgentRecord` struct and `resolve_agent_registry_via_fleet` (the T-2293 `agent resolve` verb), placed after `resolve_contact_via_fleet`. The twin body is untouched. |
| fcf18a445 | resolve_contact_via_fleet | cli | MISATTRIBUTED | Changes are to the `FleetAgentRecord` struct fields, `resolve_agent_registry_via_fleet` and `cmd_agent_resolve` (T-2297 observed_addr). None are inside `resolve_contact_via_fleet`. |
| 700b5c797 | resolve_contact_via_fleet | cli | MISATTRIBUTED | Added lines are the new `prefer_presence_fp` fn (now `agent.rs:1001`) plus its call site in `cmd_agent_contact` (`agent.rs:1485`). None are inside `resolve_contact_via_fleet`. **ADJACENT GAP:** MCP `termlink_agent_contact` still addresses a locally registered peer by its registration fp only (`tools.rs:18958`, `reg.metadata.identity_fingerprint`). It lacks the presence-fp preference, so on a shared host it can DM the host-fp rail the peer's waker does not subscribe to (T-2384 silent no-wake). |
| c96e0b7c4 | to_json | cli | LEGIT | New `ReachabilityReport::to_json` (`agent.rs:1031`) is CLI JSON shaping for a CLI-only struct. There is no MCP twin of that struct, so to_json/to_json_mcp is a false pair. **ADJACENT GAP:** MCP `termlink_agent_contact` has no reachability preflight at all: no `classify_reachability` equivalent (CLI `agent.rs:1051`, used at `agent.rs:1531`) and no `waker_running`/`recipient_live` in its output (grep finds none in `tools.rs`). |
| e73505c0f | resolve_contact_via_fleet | cli | DRIFT | CLI `resolve_contact_via_fleet` (`agent.rs:857`) routes to the peer-declared home hub: `hub_address = resolve_home_hub(m).unwrap_or(read_hub)` (T-2386). MCP `resolve_contact_via_fleet_mcp` (`tools.rs:7891`) always returns the hub the heartbeat was *read from* (`tools.rs:7930`, `best = Some((fp, address.clone(), …))`) and ignores `metadata.addr`. So an MCP cross-hub contact can post to a hub the peer does not read: the hub-split silent no-delivery case T-2386 fixed on the CLI. |
| c2a713a19 | fetch_topic_msgs | cli | MISATTRIBUTED | Added lines are the new `fetch_topic_current_values` and `current_value_msgs` fns (T-2391), placed after `fetch_topic_msgs`. The twin body is untouched. |
| c2a713a19 | resolve_contact_via_fleet | cli | LEGIT | The real change swaps `fetch_topic_msgs("agent-presence",…,500)` for `fetch_presence_msgs` (cv_index snapshot). The MCP twin was fixed equivalently in fab4803c0 (T-2392): `resolve_contact_via_fleet_mcp` calls `conn.fetch_presence_recent(500)` (`tools.rs:7912`), which subscribes with `include_current_value:true` and falls back to count-seek only on an empty snapshot. |
| fab4803c0 | fetch_topic_msgs | mcp | MISATTRIBUTED | Added lines are the new `current_value_msgs_mcp` fn, placed after `fetch_topic_msgs_mcp`. The twin body is untouched. |
| fab4803c0 | resolve_contact_via_fleet | mcp | LEGIT | This is the MCP catch-up to the CLI's c2a713a19 (`fetch_recent` → `fetch_presence_recent`). Both twins now read agent-presence via the cv_index snapshot. |
| 0fb1524ed | dm_list_filter | mcp | MISATTRIBUTED | The changed lines are the doc comment and body of `count_unread_mcp` (Option<u64> never-acked sentinel, T-2494), which follows `dm_list_filter_mcp`. The CLI `count_unread` was changed in the same commit. |
| 9b377c1ad | current_value_msgs | mcp | MISATTRIBUTED | Added lines are the new `receipt_frontier_replaces` fn (placed after a `const`, so charged to the preceding `current_value_msgs_mcp`) plus two receipt-reducer call sites. The CLI side changed in the same commit (T-2506). |
| 487309708 | detect_ack_in_msgs | cli | MISATTRIBUTED | The changed lines are the doc comment and body of `wait_for_peer_ack` (count-anchored slice → `walk_topic_from` cursor walk, T-2507). The doc comment sits just after `detect_ack_in_msgs`. The two `detect_ack_in_msgs` twins are unchanged and equivalent. **ADJACENT GAP:** the MCP `termlink_agent_contact` `ack_required` poll still uses count-anchored `conn.fetch_recent(&topic, 200)` (`tools.rs:19165`; `fetch_recent` uses `count.saturating_sub(slice)`, `tools.rs:7837`). On a swept DM topic it therefore reads the oldest live page, can miss the tail ack and report `ack.received=false`, which is the T-2507 false "unconfirmed". The CLI uses `walk_topic_from` (`channel.rs:1974`). |
| 487309708 | walk_topic_full | cli | LEGIT | `walk_topic_full` became a thin delegate, `walk_topic_from(sock, topic, 0)`, returning the same envelopes. This is a behaviour-preserving refactor, so the MCP `walk_topic_full_mcp` (`tools.rs:2916`) needs nothing. |
| f787c508a | latest_description | mcp | MISATTRIBUTED | Added lines are the new `output_delta` fn (T-2519 UTF-8 boundary-safe interact delta), placed after `latest_description_mcp`. The CLI interact path already has a boundary-safe delta (`crates/termlink-cli/src/commands/pty.rs:1584`, comment about the raw-slice panic). |
| ac859d321 | to_json | cli | MISATTRIBUTED | The changed lines are the doc comment and body of `compute_unread_rows` (T-2533 `latest_offset`), which follows `UnreadRow::to_json`. The MCP side changed in the same commit, and `latest_offset_from_list_entry_mcp` exists (`tools.rs:3096`). |
| 53d6dbd98 | latest_offset_from_list_entry | mcp | MISATTRIBUTED | Added lines are the new `mcp_run_result_json` fn (T-2537 `truncated` forwarding), placed after `latest_offset_from_list_entry_mcp`. The CLI `execution.rs` changed in the same commit. |
| 8154379b1 | parse_find_idle_log | cli | DRIFT | Already fixed by T-3218 (history-log parsers count an unparseable ts as malformed). Not re-investigated. |
| 8154379b1 | parse_substrate_log | cli | DRIFT | Already fixed by T-3218. Not re-investigated. |
| 89614cb57 | parse_queue_log | cli | DRIFT | Already fixed by T-3218. Not re-investigated. |
| 0a9369369 | connect_remote_hub | cli | DRIFT | CLI `connect_remote_hub` appends `auth_failure_hint(hub)` (names `fleet doctor` + `fleet reauth`) to **both** hub.auth failure arms: the RPC-error arm and the transport-error arm (`remote.rs:785-791`). MCP `connect_remote_hub_mcp` has an equivalent recovery message only on the RPC-error arm (`tools.rs:7766`). Its transport-error arm returns a bare `"Authentication error: {e}"` with no recovery path (`tools.rs:7773`). |
| a375a02a6 | resolve_contact_via_fleet | cli | DRIFT | CLI bounds each per-hub presence fetch with `tokio::time::timeout(FLEET_PRESENCE_HUB_TIMEOUT = 8s, …)` (`agent.rs:23`, used in `resolve_contact_via_fleet`, T-2659). MCP `resolve_contact_via_fleet_mcp` has no per-hub bound. `connect_remote_hub_mcp` bounds only the TCP/TLS connect (10s); the following `hub.auth` call and `conn.fetch_presence_recent(500)` (`tools.rs:7912`) are unbounded (`client.call` has no timeout). A half-open hub that accepts the connection can therefore wedge the MCP `termlink_agent_contact` fleet-fallback walk. |
| ac38b06b4 | to_json | cli | LEGIT | `UnreadRow::to_json` gained `receipt_up_to`/`frontier`/`indeterminate` (T-2757). The MCP side was mirrored in the same commit: `unread_verdict_mcp` / `reconcile_consumption_frontier_mcp` (`tools.rs:3031`, `tools.rs:3064`), and the MCP inbox emits `frontier` + `indeterminate` (`tools.rs:19593-19598`). |
| 6f42f2b0d | to_json | cli | MISATTRIBUTED | Added lines are the new `ack_status_rows` fn, placed after `AckStatusRow::to_json`, plus a change inside `cmd_channel_ack_status` (T-2838: the frontier counts content only, via `latest_content_offset`). **ADJACENT GAP:** MCP `termlink_channel_ack_status` still computes `latest_offset` as a naive `max(offset)` over every envelope, receipts included (`tools.rs:29898`). Each ack therefore raises the frontier it is chasing: MCP reports a permanent phantom `lag ≥ 1` for a caught-up consumer, while the CLI reports 0. |

**Totals:** DRIFT 8 · LEGIT 5 · MISATTRIBUTED 16

(3 of the 8 DRIFT rows are already fixed by T-3218. That leaves **5 open DRIFT** rows: c8fd00c39, 93dc836e5, e73505c0f, 0a9369369, a375a02a6.)

## Adjacent gaps (real, verified at HEAD, not counted above)

These came to light while checking MISATTRIBUTED or LEGIT rows. Each is a genuine CLI-ahead-of-MCP
behaviour gap in a function the script did not name.

1. **T-2384 presence-fp preference**: MCP `termlink_agent_contact` uses only the registration fp for local peers (`tools.rs:18958`). CLI: `prefer_presence_fp` (`agent.rs:1001`, `agent.rs:1485`).
2. **T-2385 reachability preflight**: missing entirely from MCP `termlink_agent_contact`. CLI: `classify_reachability` (`agent.rs:1051`, `agent.rs:1531`).
3. **T-2507 ack poll**: MCP `ack_required` poll is count-anchored (`tools.rs:19165`, via `fetch_recent`, `tools.rs:7837`). CLI: `walk_topic_from` (`channel.rs:1974`).
4. **T-2838 ack-status frontier**: MCP `termlink_channel_ack_status` counts receipt envelopes in `latest_offset` (`tools.rs:29898`). CLI: `latest_content_offset`.
