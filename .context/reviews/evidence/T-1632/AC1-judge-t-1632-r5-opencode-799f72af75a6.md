# Reviewer evidence — T-1632 AC#1

- dispatch: `judge-t-1632-r5-opencode-799f72af75a6`
- worker kind: `opencode` (vendor `zai`, model `zai-coding-plan/glm-5.3`)
- seat: `opencode`
- reviewed revision: `6f6fac33a0fa6822b5301aecd5edec1b93518d1e`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `1114f839682670698f069f36ed10700f57786959b358535e180838226e60b813`
- result.md sha256: `a3fa7acddfd5054e16939247e4c7e0cb8bbca73d76c958f69c81b02f92c7f945`

## The worker's verdict block

```
1. [AC] .122 deploy returns control_plane_version 3 alongside protocol_version 1
VERDICT: green
WHY: The criterion is a widened act-in-the-world criterion (SD-ab145e8e, T-3800): the recorded output under the Measured: line is the evidence, and I checked it against the export. (a) The fenced block is present and identical in the task file (.tasks/active/T-1632-bump-defaultprotocolversion-to-3-t-1166-.md:99-104): hub.capabilities: "protocol_version": 1, "control_plane_version": 3, "hub_version": "0.12.13+135.gd167da0e2", "features": {"legacy_primitives": false} and hub.version: "protocol_version": 1, "control_plane_version": 3, "hub_id": "22c19fedafd73da2" — both required fields present with the expected values, unambiguous. (b) The measurement names its target (192.168.10.122:9100) and its tie to the "next .122 deploy": the operator's runme run of 2026-11-17 08:50Z, whose fleet action quotes "ring20-management serves 0.12.13+135.gd167da0e2 ... hub_id 22c19fedafd73da2 unchanged" — the recorded hub_version equals the deployed build and the recorded hub_id matches the fleet action and every independent repo record of .122's hub id (docs/operations/fleet-join.md:43, T-3366 bundles, T-2762), so the hub queried is the freshly deployed .122. A second task independently records the same deploy (.tasks/active/T-2258...md:168: "hub_version 0.12.13+135.gd167da0e2 deployed by the operator's runme run 2026-11-17 08:50Z"). (c) Precondition "after T-1166 bake clears": the output shows legacy_primitives: false (post-cut state) and the T-3800 review bundles record the bake cleared (cut landed 2026-05-31, 7-day legacy usage 0) long before this deploy. (d) Code-side consistency: CONTROL_PLANE_VERSION: u8 = 3 (crates/termlink-protocol/src/lib.rs:33) emitted in both hub.capabilities and hub.version handlers (crates/termlink-hub/src/router.rs:904,1174). Minor note, not blocking: the Steps name the CLI verb termlink remote call while the measurement used the termlink MCP remote call of the same method (hub.capabilities) on the same hub — same act, same target, transparently stated in the Measured line. My sandbox cannot reach .122, so I judged the recording as the ruling directs, cross-checked rather than taken on trust.
GUIDANCE: none required — the recorded output proves the act. If the operator wants the literal CLI form too, a one-off termlink remote call --hub 192.168.10.122:9100 --method hub.capabilities can be appended, but it adds nothing to what is already proven.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
The evidence block in the criterion matches the task file. Now cross-checking the recorded output's internal consistency against the repo (version string, hub_id, control_plane_version constant).
1. [AC] .122 deploy returns control_plane_version 3 alongside protocol_version 1
VERDICT: green
WHY: The criterion is a widened act-in-the-world criterion (SD-ab145e8e, T-3800): the recorded output under the `**Measured:**` line is the evidence, and I checked it against the export. (a) The fenced block is present and identical in the task file (`.tasks/active/T-1632-bump-defaultprotocolversion-to-3-t-1166-.md:99-104`): `hub.capabilities: "protocol_version": 1, "control_plane_version": 3, "hub_version": "0.12.13+135.gd167da0e2", "features": {"legacy_primitives": false}` and `hub.version: "protocol_version": 1, "control_plane_version": 3, "hub_id": "22c19fedafd73da2"` — both required fields present with the expected values, unambiguous. (b) The measurement names its target (192.168.10.122:9100) and its tie to the "next .122 deploy": the operator's runme run of 2026-11-17 08:50Z, whose fleet action quotes "ring20-management serves 0.12.13+135.gd167da0e2 ... hub_id 22c19fedafd73da2 unchanged" — the recorded `hub_version` equals the deployed build and the recorded `hub_id` matches the fleet action and every independent repo record of .122's hub id (docs/operations/fleet-join.md:43, T-3366 bundles, T-2762), so the hub queried is the freshly deployed .122. A second task independently records the same deploy (`.tasks/active/T-2258...md:168`: "hub_version 0.12.13+135.gd167da0e2 deployed by the operator's runme run 2026-11-17 08:50Z"). (c) Precondition "after T-1166 bake clears": the output shows `legacy_primitives: false` (post-cut state) and the T-3800 review bundles record the bake cleared (cut landed 2026-05-31, 7-day legacy usage 0) long before this deploy. (d) Code-side consistency: `CONTROL_PLANE_VERSION: u8 = 3` (crates/termlink-protocol/src/lib.rs:33) emitted in both `hub.capabilities` and `hub.version` handlers (crates/termlink-hub/src/router.rs:904,1174). Minor note, not blocking: the Steps name the CLI verb `termlink remote call` while the measurement used the termlink MCP remote call of the same method (`hub.capabilities`) on the same hub — same act, same target, transparently stated in the Measured line. My sandbox cannot reach .122, so I judged the recording as the ruling directs, cross-checked rather than taken on trust.
GUIDANCE: none required — the recorded output proves the act. If the operator wants the literal CLI form too, a one-off `termlink remote call --hub 192.168.10.122:9100 --method hub.capabilities` can be appended, but it adds nothing to what is already proven.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
