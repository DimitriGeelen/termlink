# T-3654 — main-only delivery: rollout tracker (operator 2026-11-02: "yes, 1 and 2")

Goal (focus anchor): every project's peer mail reaches only its ONE main session.
Requirements per project: termlink with `termlink main` (0.12.1494+, installed on .107 and the .122/.121/.132 hubs by
operator runme 2026-11-02 17:22Z) + AEF v1.8.16-be.26 or later (T-5014 hook). Notice sent on conversation
`main-only-rollout-2026-11-02` (2026-11-02). LIVE = project reported `termlink main status` held by its main session and
`am-i` exit 1 for another session id.

| Project | Host | Status | Evidence / date |
|---|---|---|---|
| 999-Agentic-Engineering-Framework | .107 | LIVE | msg 7990ad29: main gen 1, am-i 0/1, mail surfaced in main (2026-11-02) |
| proxmox-ring20-management | .122 | LIVE | msg 25bff094: main afce1d16 gen 1, am-i 0/1, first delivery HANDED_OVER to main |
| 010-termlink | .107 | PENDING | termlink live, main gen 1 (17:22Z); be.26 re-vendor running (T-3659) |
| 003-Vailliant-diagnosis | .107 | WAITS OPERATOR GO | on 1.8.15; asking its operator (msg 91c67274) |
| 0235-voice-features | .107 | NOTIFIED | |
| 050-email-archive | .107 | WAITS OPERATOR GO | be.25 -> be.26; asking its operator (msg 5b0046ac) |
| 0506-Voxtype-extention | .107 | NOTIFIED | |
| 055-agentic-fleet-cockpit | .107 | WAITS DEPLOY + OPERATOR GO | deploy in flight, then operator go (msg 72bcaecb) |
| 0909-aef-greenfield-v22 | ? | NOTIFIED | |
| 0909-boot-into-timeshift-snapshots | ? | NOTIFIED | |
| 1023-portable-encrypted-chromium-vault | .107 | WAITS OPERATOR GO | on 1.8.12; needs the operator's go in its own session (msg aaef341a) |
| 1409-sprind | .107 | NOTIFIED | |
| 832-Workflow-designer | .107 | WAITS OPERATOR GO | on be.15; operator job (msg 8cf1f171) |
| aef-greenfield-test | .132 | NOTIFIED | |
| dimitri-mint-dev | .107 | WAITS .107 REBOOT | mid disk migration (T-213); cutover + reboot of .107 pending; re-vendor after, with operator OK (msg c7d43c64) |
| fan-dashboard | .107 | WAITS OPERATOR GO | on 1.8.15+3; termlink ok, lease vacant (msg 972b2085) |
| ring20-dashboard | .121 | NOTIFIED | |
| claude-shared-toolkit | ? | NOT REACHED | send refused (no receiver); check whether it is an agent project at all |
