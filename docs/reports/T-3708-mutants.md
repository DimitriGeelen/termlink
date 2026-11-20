# T-3708 mutants (P-24 slice a)

Runner: `python3 crates/termlink-session/tests/mutants/run-t3708.py` (run 2026-11-20T12:11:58Z, HEAD 649145b92). Tests run per mutant: sidecar lib fields::/host::, policy_tests t3708, t3708_status, cargo build -p termlink, t3708_live. The runner rebuilds the clean binary at the end.

| mutant | what | verdict |
|---|---|---|
| m1-cached-without-stamp | E-19/E-55 control: a field past its bound is served as fresh | killed |
| m2-heartbeat-is-surfacing | E-56 control (arch 6.7.5): pending mail without a hand-over still reads CLEAR | killed |
| m3-idle-age-threshold | E-57 control: an idle agent turns bad by age alone | killed |
| m4-per-agent-yes-on-root | E-58 control: per_agent_accounts claims yes on a root host | killed |
| m5-wall-time-deadlines | E-59 control: process time follows the wall clock | killed |
| m6-unknown-counts-fresh | FCW-121: an unknown field does not hold CLEAR | killed |
| m7-failed-pass-stamps-hub | FCW-120: a pass that could not reach the hub stamps `hub` fresh | killed |
| m8-restart-is-fresh | 06a 2.13.f: a field never measured since start reads fresh | killed |
| m9-switch-default-off | design 3.0: the new keys are hidden by default | killed |
| m10-sent-handover-is-ours | a recipient's hand-over of a message we SENT counts as our surfacing | killed |
| m11-v8-from-ready-only | V-8: a message stored after READY is overdue counted from READY | killed |
| m12-hook-time-overwritten | V-8: a later hook call resets the first missed time | killed |
| m13-e127-ignores-narrowing | E-127 reader: an unverified list past PN-13 fires even while narrowing | killed |
| m14-e127-ignores-held-time | E-127 reader: a message stored before the key was held is flagged | killed |
| m15-skew-last-not-median | V-5: one slow round trip flags clock skew | killed |
| m16-skew-threshold-30s | V-5: clock skew flagged only past 30 s | killed |
| m17-ntp-guess | V-5: with no tools the NTP state is claimed synchronised | killed |
| m18-stale-revocation-ok | AL-29: a stale revocation list is not a bad field | killed |
| m19-second-consumer-ok | AL-29: more than one consumer is not a bad field | killed |
| m20-downgrade-reason-never-stale | carried T-3703 CC N4: a stale-gate downgrade names the allow-list | killed |
| m21-no-skew-from-version | V-5: an agent with an empty inbox is never measured for skew | killed |
| m22-hub-version-no-time | V-5: the hub's version reply carries no clock | killed |
| m23-latency-not-logged | E-55/E-134: status calls leave no latency line | killed |
| m24-pass-not-stamped | FCW-120: a successful pass does not stamp `hub` | killed |
| m25-hub-predates-is-fine | no verified list (`hub predates`) does not hold CLEAR | killed |
| m26-invariant-ignored | a broken revocation invariant does not hold CLEAR | killed |

26/26 killed
rc=0
