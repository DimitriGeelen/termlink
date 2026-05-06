# T-1166 Cut-Flip Projection — 2026-05-06

**Status as of 2026-05-06T18:59Z:** `cut_ready: false` (legacy_pct = 2.904%, gate = 1.0%)

**Projected cut-flip date:** **2026-05-10** (4 days from observation)

**Conclusion:** Legacy emissions effectively stopped on 2026-05-04 after the
T-1418 binary deploy. The remaining 4451 in-window legacy events are
**historical residue** that will roll out of the 7-day window between
2026-05-09 and 2026-05-10. No further code action is required — the cut is
purely time-gated.

---

## Current State (live data)

```
windows[0]:
  days: 7
  total: 156714
  legacy: 4552
  legacy_attributable: 4460
  legacy_unattributable_pre_t1409: 92
  legacy_pct: 2.9047%
  passing: false
gate_pct: 1.0
```

Source: `fw metrics api-usage --last-Nd 7 --json` against
`/var/lib/termlink/rpc-audit.jsonl` on ring20-management (.102).

## Per-Day Legacy Decay (last 7 days)

| Date          | Legacy | Total  | Pct     | Notes                                         |
|---------------|-------:|-------:|--------:|-----------------------------------------------|
| 2026-04-29    |    223 |   3542 |  6.30%  | Window start                                  |
| 2026-04-30    |   1005 |   8247 | 12.19%  |                                               |
| 2026-05-01    |   1443 |  13927 | 10.36%  | Peak                                          |
| 2026-05-02    |   1389 |  13294 | 10.45%  |                                               |
| 2026-05-03    |    486 |  13333 |  3.65%  | Drop begins                                   |
| **2026-05-04**| **0**  |  25698 |  0.00%  | **Effective cutover (post-T-1418 deploy)**    |
| 2026-05-05    |      1 |  35397 |  0.00%  | Likely .122 fallback (event.broadcast)        |
| 2026-05-06    |      4 |  43296 |  0.01%  | Likely .122 fallback                          |

**Reading:** The May 4 zero-line is the operational cutover. Volumes since
have been negligible (5 emissions / 3 days). The Apr 29 - May 3 totals
(4546) are the residue draining out of the rolling window.

## Live Residue Sources (last 7 days)

| Source           | Method            | Count | Last Seen           |
|------------------|-------------------|------:|---------------------|
| 192.168.10.143   | `inbox.status`    |  2949 | 2026-05-02T06:13Z   |
| 192.168.10.121   | `inbox.status`    |  1502 | 2026-05-03T08:04Z   |
| **192.168.10.122** | `event.broadcast` |    6 | **2026-05-06T13:46Z** |
| 192.168.10.121   | `inbox.list`      |     1 | 2026-05-03T20:31Z   |
| (unattributable, pre-T-1409) | various |    92 | (rolling)         |

The .143 and .121 entries last fired 3-4 days ago and will exit the 7-day
window on 2026-05-09 / 2026-05-10. The .122 ~1/day rate (6 hits across 7
days) is the only ongoing emission and is too small to block the gate.

## Projection (no further legacy emissions assumed)

| Day-ahead | Date        | Legacy | Total~  | Pct%    | State          |
|-----------|-------------|-------:|--------:|--------:|----------------|
| Today     | 2026-05-06  |   4551 |  156734 |  2.904% | BLOCKED        |
| +1        | 2026-05-07  |   4328 |  153192 |  2.825% | BLOCKED        |
| +2        | 2026-05-08  |   3323 |  144945 |  2.293% | BLOCKED        |
| +3        | 2026-05-09  |   1880 |  131018 |  1.435% | BLOCKED        |
| **+4**    | **2026-05-10** | **491** | **117724** | **0.417%** | **READY ← cut-flip** |
| +5        | 2026-05-11  |      5 |  104391 |  0.005% | READY          |
| +6        | 2026-05-12  |      5 |   78693 |  0.006% | READY          |
| +7        | 2026-05-13  |      4 |   43296 |  0.009% | READY          |

**Sensitivity:** If the .122 fallback continues at ~1-2 emissions/day, by
May 10 the in-window total would be ~496-498 (vs the 491 projected).
0.42% → 0.43%. Still well under the 1.0% gate.

## Operator Authorization Runbook

### Pre-flight (anytime)

```bash
cd /opt/termlink && bin/fw metrics api-usage --last-Nd 7 --json | python3 -c "import json,sys; d=json.load(sys.stdin); w=d['windows'][0]; print(f\"cut_ready={d.get('gate',{}).get('passing')}, legacy_pct={w['legacy_pct']:.3f}%, gate={d['gate_pct']}%\")"
```

Expected today: `cut_ready=False, legacy_pct=2.904%, gate=1.0%`
Expected on/after 2026-05-10: `cut_ready=True, legacy_pct<1.0%, gate=1.0%`

### On 2026-05-10 (or later)

1. **Verify cut-readiness:**
   ```bash
   cd /opt/termlink && .agentic-framework/bin/fw metrics api-usage --cut-ready --json
   ```
   Look for `"cut_ready": true` and `"legacy_pct"` under 1.0%.

2. **Confirm .122 is the only live source** (sanity check — should be
   the only entry with a `last_seen_iso` younger than 24h):
   ```bash
   cd /opt/termlink && .agentic-framework/bin/fw metrics api-usage --last-Nd 1 --json | python3 -c "import json,sys; d=json.load(sys.stdin); [print(r) for r in d.get('legacy_callers_by_ip', [])]"
   ```

3. **Authorize cut** by re-engaging T-1166 to schedule the legacy-RPC removal
   patch (per T-1166 task body for the actual cut sequence).

### If cut_ready is still False on 2026-05-10

Inspect the per-day decay against the projection table above. If totals
are higher than projected, a new legacy-emitting source has appeared — open
a fresh task to identify it before authorising. Do **not** force the cut
with `--scope-reduction-acknowledged` while a fresh source is live.

## Methodology

- Audit log: `/var/lib/termlink/rpc-audit.jsonl` on ring20-management (.102).
- Window: rolling 7 days from observation timestamp (2026-05-06T18:59Z).
- Legacy methods: `event.broadcast`, `inbox.status`, `inbox.list`,
  `inbox.push`, `file.send`, `file.receive`.
- Projection assumption: no new legacy emissions from observation onward
  (validated against May 4-6 trajectory: 5 emissions / 3 days, all
  consistent with .122 fallback only).

## Related

- T-1166 — primary cut task
- T-1418 — binary deploy that produced the May 4 zero-line
- T-1419 — `last_seen_iso` field this projection relies on
- T-1416 — `--cut-ready` gate flag
- T-1409 — peer attribution that distinguished attributable from pre-attribution residue
