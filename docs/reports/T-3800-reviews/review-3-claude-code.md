T-1415: UNCLEAR — the bundle shows the legacy code is gone from source and usage is zero on six hubs, but not which binaries the production hubs have actually run for the last 7 days, and laptop-141's bake metric was not evaluated.

T-1632: PASS — the hub at .122 (address mapping shown) returns `control_plane_version: 3` alongside `protocol_version: 1` now, and its own 7-day legacy usage is 0.

T-2258: PASS — the running .107 hub binary is `023fbc762`, which contains the fix commit `db7c10a56`, and the write-then-read and 4-way concurrent reads all returned rc=0 in 152ms or less under a 12s kill timeout.

T-3685: PASS — action 28 (the paste-at-terminal step) went from FAILED to OK across the two 2026-11-11 runs, and since a working stored token is logged as "skip", an OK means the paste path ran and verified.

Remarks:
- T-1415: the handler grep as quoted uses `|` without `-E`, which plain grep treats literally, so "0 matches" may be an artifact. The `router.rs` excerpt is the stronger source evidence, and a per-hub `legacy_primitives: false` probe or version/deploy-date list would settle the criterion.
- T-1632: the bundle asserts, but does not show, that the T-1166 bake is the legacy-usage bake; I passed it on the observable result, so quote T-1166's status if that precondition matters.
- T-2258: the repro results are summarised without the commands, and the original repro's topic size is not given, so I took 1500 rows as representative.
