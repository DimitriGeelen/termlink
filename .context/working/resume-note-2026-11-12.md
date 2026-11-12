# Resume note (main session 10c2f139, 2026-11-12 ~12:40Z) — read this after /compact + /resume

Running workers (re-arm a watch on each; they write hand-backs when done):
1. t3689-fold (T-3689): folds step 7 change requests CR-1..CR-6 + operator ruling D (macOS interim ps veto) into the
   arc-011 step 1-6 documents. Hand-back: .context/working/t3689/handback.md. Then: check it, list anything it left for
   the operator, close T-3689.
2. t3687-b27-cr (T-3687): B-27 activity meter repair round CR-B27 (F-3, F-4, F-22, F-26, F-27, F-28).
   Hand-back: .context/working/t3687/handback-cr.md. Then: check tests, decide whether T-3687 can close.
Watch: `pgrep -f 'run.sh t3689-fold'` and `pgrep -f 'run.sh t3687-b27-cr'`.

Waiting on the operator:
3. Tick "[REVIEW] Step 7 output approved" at http://192.168.10.107:3003/review/T-3356 (approved in chat "A";
   the agent never ticks Human ACs).

Next after both workers:
4. Step 8 planner (T-3357): role session -> build plan; operator approves (last design approval).
5. Build queue (operator ruling A, T-3426): B-25 lease tightening -> B-11 encryption keys -> B-16 doorbell D+
   (with F-1 typing re-check and the macOS ps veto). T-3688 full macOS veto later.

Waiting on others:
6. ring20 T-2636: Infisical identity 010-termlink@107 + infisical-get for testdev's token (T-3685 switches to it).
7. ring20 T-2595: WireGuard route + new key for .132 (operator must tell ring20 "go" directly).
8. Main-only rollout (T-3654): 10 LIVE; 1023 and 1409 upgrade pending; tracker docs/reports/T-3654-main-only-rollout.md.

Open bugs filed this session: T-3678 (flaky multi-MB file test), T-3680 (sidecar Store Locked flake), T-3681 (git-less
build version), T-3686 (main lease test seams obeyed in production).
