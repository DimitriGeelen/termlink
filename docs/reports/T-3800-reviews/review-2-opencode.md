T-1415: PASS — `legacy_primitives: false` shows the flag off now, and the 604800s window (`total_legacy: 0`, `last_legacy_ts_ms: null`, audit present) plus fleet "CLEAN (7d)" on the production hubs shows the ≥7-day bake and clean metric; laptop-141 is not indicated to be a production hub.

T-1632: PASS — the criterion's observable is exactly `hub.capabilities` returning `control_plane_version: 3` alongside `protocol_version: 1`, and that is measured live on the management hub, which also implies the deploy occurred.

T-2258: UNCLEAR — fix commit ancestry in the running binary is proven and reads are fast, but today's run deliberately skipped the write-then-read/concurrent variant, and the only full live-repro PASS is 2026-07-05 on 0.11.324, not the current 0.12.13+32 build; missing is the actual repro re-run against the deployed binary.

T-3685: FAIL — the runme log explicitly records "skip": the token was not created at the terminal; a pre-existing stored token answering one prompt does not satisfy "create testdev's token at the terminal."

Remarks: T-2258's gap is disclosed but real — commit ancestry plus a reduced spot-check is not the live repro on the current build. T-1632 leans PASS only because the capabilities response is itself the criterion; the .122/T-1166 preconditions are inferred, not evidenced. "Skip with a workaround" should never close a criterion phrased as a creation step.
