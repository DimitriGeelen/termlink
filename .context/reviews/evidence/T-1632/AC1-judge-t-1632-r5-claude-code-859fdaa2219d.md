# T-1632 Human AC#1 — evidence report (seat claude-code, rung-5-panel)

Reviewed revision: 6f6fac33a0fa6822b5301aecd5edec1b93518d1e
Criterion: on the .122 deploy, `hub.capabilities` returns `control_plane_version: 3` alongside `protocol_version: 1`.

## What I checked

1. **The Measured block at the reviewed revision** (`git show 6f6fac33a:.tasks/active/T-1632-…md`, lines 99-104).
   It names the call (`hub.capabilities` scope execute, `hub.version` scope observe, on 192.168.10.122:9100 via the termlink MCP
   remote call, 2026-11-17) and records `"protocol_version": 1, "control_plane_version": 3,
   "hub_version": "0.12.13+135.gd167da0e2"`. The output is unambiguous and shows both expected fields.

2. **Independent re-measurement (not taken on trust).** I ran the same call myself through `termlink mcp serve`
   (tool `termlink_remote_call`, hub 192.168.10.122:9100):

   ```
   == hub.capabilities (scope execute) 2026-11-17T13:59:25Z
   {"control_plane_version": 3, "features": {"legacy_primitives": false}, "hub_version": "0.12.13+135.gd167da0e2", "protocol_version": 1}  (methods: 44 entries, omitted)
   == hub.version (scope observe) 2026-11-17T13:59:33Z
   {"control_plane_version": 3, "hub_id": "22c19fedafd73da2", "hub_version": "0.12.13+135.gd167da0e2", "protocol_version": 1, ...}
   ```

   Matches the recorded block field for field (values, hub_version, hub_id).

3. **Hub identity / deploy.** `termlink fleet doctor --json`: profile `ring20-management` = 192.168.10.122:9100,
   status ok, hub_version 0.12.13+135.gd167da0e2, hub_id 22c19fedafd73da2. `termlink remote doctor ring20-management --json`: ok, 3/3 pass.
   So the new build is the one serving on .122.

4. **Source at the deployed commit.** `git grep d167da0e2`: `crates/termlink-hub/src/router.rs:1174` emits
   `"control_plane_version": termlink_protocol::CONTROL_PLANE_VERSION` in the capabilities handler, next to
   `"protocol_version": termlink_protocol::DATA_PLANE_VERSION`; `crates/termlink-protocol/src/lib.rs` has
   `CONTROL_PLANE_VERSION: u8 = 3`, `DATA_PLANE_VERSION: u8 = 1`. Wire output agrees with source.

5. **"After T-1166 bake clears".** The live response carries `features.legacy_primitives: false`, i.e. the legacy cut is in the
   deployed build; the deploy is post-bake.

6. **Widening.** `.context/reviews/standing-delegations.jsonl`: SD-ab145e8e, kind widening, class act-in-the-world, ruling T-3800,
   tasks include T-1632, pattern includes `control_plane_version: 3`, expires 2027-02-15 — active and covers this criterion.

## Observation (does not change the verdict)

The Steps line gives the CLI form `termlink remote call --hub … --method hub.capabilities`. The installed CLI
(0.12.13+135.gd167da0e2) has no `remote call` subcommand (`error: unrecognized subcommand 'call'`); the call exists as the MCP
tool `termlink_remote_call`, which is what the Measured line used and what I used. The criterion's claim and its Expected clause
(method `hub.capabilities`, host .122:9100, both field values) are current and hold literally; only the how-to wrapper in step 1
is stale. No rollback was needed, so the "If not" path does not apply.

## Verdict

green — the recorded output proves the act as the criterion states, and I reproduced it live.
