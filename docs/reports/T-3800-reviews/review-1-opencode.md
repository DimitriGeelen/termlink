T-2858: PASS — all three known paths report the identical version string and matching sha256 prefix, directly showing no drift.

T-2938: PASS — `cmp` shows the deployed file is byte-identical to the tracked source, and the drift-check script independently reports 0 non-matching entries.

T-2939: PASS — `ls` shows the canary installed in /etc/cron.d, `cmp` shows it identical to the tracked source, and the heartbeat mtime (2026-11-15T07:37:23, measurement day) matches the stated daily 07:37 schedule, showing it actually ran.

T-3178: PASS — offset 202 on `framework:pickup` explicitly references 050-email-archive's P-006/T-2257 and states "This confirms... Applied and tested here," which is the required reply reporting reproduction; offset 206 amends only the patch detail, not the confirmation.

General remarks:
- T-2939's "scheduled" component relies on the heartbeat timestamp plus the annotation that the cron line runs at 07:37; the cron line content itself is never quoted, but the same-day heartbeat at 07:37:23 corroborates it.
- T-3178's quote is truncated after the confirmation statement; the truncation does not affect the part that satisfies the claim.
