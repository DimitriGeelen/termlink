# T-3708 mutants (P-24 slice a)

Runner: `python3 crates/termlink-session/tests/mutants/run-t3708.py`. Full run after review round 2 fixes: 38/40 killed; m5e survived (test gap: the clock test's earlier jumps summed to zero) and m34 did not compile (an invalid mutant). Fixed (a direct registration check in t3708_clock; m34 rewritten) and rerun: 2/2 killed. Total 40/40. Not in the list: m35 (a failed held-since write is reported), which cannot be triggered while the tests run as root.

| mutant | what | verdict |
|---|---|---|
| m1-cached-without-stamp | E-19/E-55 control: a field past its bound is served as fresh | killed |
| m2-heartbeat-is-surfacing | E-56 control (arch 6.7.5): pending mail without a hand-over still reads CLEAR | killed |
| m3-idle-age-threshold | E-57 control: an idle agent turns bad by age alone | killed |
| m4-per-agent-yes | E-58 control (r1 MAJOR 3): per_agent_accounts claims yes | killed |
| m4b-root-host-separated | E-58 control: a root host reads as separated | killed |
| m5-wall-time-deadlines | E-59 control (r1 BLOCKER/MAJOR 9): deadline ages follow the wall clock | killed |
| m5b-jumps-by-wall-range | E-59: a stamp's side of a jump is guessed from wall values (fails after a backward jump) | killed |
| m5c-deliver-wall-deadline | E-59: the receiver's hand-over deadline uses plain wall age | killed |
| m5d-deliver-orders-wall-stamps | r2 MAJOR 2: the later of storing and readiness is picked by wall value | killed |
| m6-unknown-counts-fresh | FCW-121: an unknown field does not hold CLEAR | killed |
| m7-failed-pass-stamps-hub | FCW-120: a pass that could not reach the hub stamps `hub` fresh | killed |
| m8-restart-is-fresh | 06a 2.13.f: a field never measured since start reads fresh | killed |
| m9-switch-default-off | design 3.0: the new keys are hidden by default | killed |
| m10-sent-handover-is-ours | a recipient's hand-over of a message we SENT counts as our surfacing | killed |
| m11-v8-from-ready-only | V-8: a message stored after READY is overdue counted from READY | killed |
| m12-hook-time-overwritten | V-8: a later hook call resets the first missed time | killed |
| m27-pending-window-one-hour | r1 MAJOR 2: mail pending for more than an hour drops out of CLEAR | killed |
| m28-unknown-revocation-ok | r1 MAJOR 5: an unknown revocation line reads fine | killed |
| m29-heartbeat-age-ignored | r1 MAJOR 5: an old heartbeat file with fresh component ages reads fine | killed |
| m30-consumers-stamped-on-error | r1 MAJOR 4: a consumers check the hub did not answer stamps the field | killed |
| m31-revocation-off-unstamped | FCW-120: a switched-off revocation pass leaves the field unstamped (reads unknown, never CLEAR) | killed |
| m32-peer-time-in-rows | r1 MAJOR 6: V-8 and CLEAR read the peer's clock | killed |
| m33-hook-miss-in-memory | r1 MAJOR 7: the hook-miss record is not kept | killed |
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
| m10b-status-calls-the-hub | design mutant 10 (r2 MAJOR 5): status makes a hub call (blocks on a silent hub) | killed |
| m5e-stamps-not-registered | r2 MAJOR 1: chain stamps are not registered when written | killed |
| m34-unanchored-pending-ignored | r2 MAJOR 4: pending mail with nothing to measure from does not hold CLEAR | killed |
