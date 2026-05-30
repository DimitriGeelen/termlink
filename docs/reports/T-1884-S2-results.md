# T-1884 S2 — mechanical-Step dry-run results

Targets from S1: 16 ACs across 12 tasks

## T-1296 — Apply same migration recipe as T-1294

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **FAIL**  (safe cmds returned non-zero exits: [2, 0, 0, 1])
   commands: 12 parsed (safe=4, risky=3, interactive=0, unknown=5)
   expected: 'hub running on `/var/lib/termlink/`, secret preserved.\n  **If not:** see T-1294 AC 2 troubleshooting notes.'
     [SAFE       ] step 1: `ls /root/*/scripts/*watchdog*.sh`
     [RISKY      ] step 2: `mkdir -p /var/lib/termlink && chmod 700 /var/lib/termlink && cp -a /tmp/termlink-0/. /var/lib/termlink/`
     [UNKNOWN    ] step 3: `export TERMLINK_RUNTIME_DIR=/var/lib/termlink`
     [UNKNOWN    ] step 3: `set -uo pipefail`
     [UNKNOWN    ] step 3: `${TERMLINK_RUNTIME_DIR}/hub.sock`
     [UNKNOWN    ] step 3: `${TERMLINK_RUNTIME_DIR}/hub.secret`
     [RISKY      ] step 4: `rm -f /var/lib/termlink/{hub.sock,hub.pid,hub.tcp}`
     [RISKY      ] step 5: `pkill -f '^termlink hub start'`
     [SAFE       ] step 6: `ls -la /var/lib/termlink/`
     [SAFE       ] step 6: `pgrep -af 'termlink hub'`
     [SAFE       ] step 6: `tail -5 /root/.../termlink-hub.log`
     [UNKNOWN    ] step 6: `persist-if-present`
     → step 1 exit=2 out: ''
     → step 6 exit=0 out: 'total 448820'
     → step 6 exit=0 out: '2382342 /root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100'
     → step 6 exit=1 out: ''

## T-1296 — Re-pin from .102

   class:    OBSERVE-INFRA
   verdict:  **PASS-LOOSE**  (3 safe cmds clean + Expected matched, but 1 risky/interactive remain)
   commands: 4 parsed (safe=3, risky=1, interactive=0, unknown=0)
   expected: 'All green, fleet PASS 3/3.'
     [SAFE       ] step 1: `cat /var/lib/termlink/hub.secret`
     [RISKY      ] step 2: `printf '<paste>' > /tmp/secret.hex && termlink fleet reauth ring20-dashboard --bootstrap-from file:/tmp/secret.hex && rm`
     [SAFE       ] step 3: `termlink fleet doctor`
     [SAFE       ] step 4: `termlink remote ping ring20-dashboard`
     → step 1 exit=0 out: 'b3076eb72f0a8f167219ea2545671eb5d518807913165c5e5b7fe72b7db78cc2'
     → step 3 exit=0 out: ''
     → step 4 exit=0 out: 'PONG from hub 192.168.10.121:9100 — 1 session(s) — 81ms (auth: 80ms, discover: 1ms)'

## T-1296 — Verify CT 101 reboot persistence

   class:    OBSERVE-INFRA
   verdict:  **PASS-LOOSE**  (1 safe cmds clean + Expected matched, but 2 risky/interactive remain)
   commands: 3 parsed (safe=1, risky=0, interactive=0, unknown=2)
   expected: 'Same hash. If not, escalate — `/var/lib/termlink` itself is volatile in CT 101.'
     [SAFE       ] step 1: `sha256sum /var/lib/termlink/hub.secret`
     [UNKNOWN    ] step 2: `pct reboot 101`
     [UNKNOWN    ] step 3: `pct enter 101 && sha256sum /var/lib/termlink/hub.secret`
     → step 1 exit=0 out: 'bce6f5f64fcc167ab7a0d12ece1e3fa085a4bde4effe28127c505da40e0a4eaf  /var/lib/termlink/hub.secret'

## T-1420 — Binary deployed on .141

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **OPERATOR-ONLY**  (4 commands all risky/interactive — orchestrator surfaces only)
   commands: 4 parsed (safe=0, risky=1, interactive=0, unknown=3)
   expected: '`--version` reports 0.9.1591 or newer; `channel --help` lists 53 subcommands.\n  **If not:** check disk space, check write permissions on `/mnt/c/...` (NTFS-WSL '
     [UNKNOWN    ] step 2: `termlink 0.9.1591`
     [UNKNOWN    ] step 3: `--version`
     [UNKNOWN    ] step 3: `channel --help`
     [RISKY      ] step 3: `chmod +x`

## T-1420 — .141 hub restarted on new binary

   class:    OBSERVE-INFRA
   verdict:  **INCONCLUSIVE**  (safe ran but ambiguous evidence (1 risky residue))
   commands: 3 parsed (safe=2, risky=1, interactive=0, unknown=0)
   expected: 'new hub PID, started time recent, listening on 0.0.0.0:9100.\n  **If not:** if no watchdog manages it, the manual restart command is in Method A above.'
     [SAFE       ] step 1: `pgrep -af 'termlink hub'`
     [RISKY      ] step 2: `kill $PID`
     [SAFE       ] step 4: `pgrep -af 'termlink hub'`
     → step 1 exit=0 out: '2382342 /root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100'
     → step 4 exit=0 out: '2382342 /root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100'

## T-1420 — Full chat arc parity confirmed via fleet check

   class:    OBSERVE-INFRA
   verdict:  **OPERATOR-ONLY**  (2 commands all risky/interactive — orchestrator surfaces only)
   commands: 2 parsed (safe=0, risky=0, interactive=0, unknown=2)
   expected: 'PASS with 53/53.\n  **If not:** the binary on disk vs the binary the running hub holds may\n  differ — the running process retains the old executable until restar'
     [UNKNOWN    ] step 2: `channel commands on .141: 53 / 53`
     [UNKNOWN    ] step 2: `PASS`

## T-1431 — skill works end-to-end from a real session

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **FAIL**  (safe cmds returned non-zero exits: [0, 1])
   commands: 3 parsed (safe=2, risky=0, interactive=0, unknown=1)
   expected: 'end-to-end works without prompts, manual fallbacks, or improvisation\n  **If not:** capture failure point in Updates and re-scope which step broke'
     [UNKNOWN    ] step 1: `termlink register --name handoff-rubber-stamp --self --json &`
     [SAFE       ] step 4: `grep -A6 "handoff-posted" .tasks/active/T-1431-*.md | tail -8`
     [SAFE       ] step 5: `termlink channel list --prefix dm: | grep handoff-rubber`
     → step 4 exit=0 out: '  4. `grep -A6 "handoff-posted" .tasks/active/T-1431-*.md | tail -8` — see the update entry'
     → step 5 exit=1 out: ''

## T-1457 — Operator action on .141

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **PASS-LOOSE**  (3 safe cmds clean + Expected matched, but 5 risky/interactive remain)
   commands: 8 parsed (safe=3, risky=1, interactive=0, unknown=4)
   expected: 'From .107, `termlink agent contact --hub laptop-141 --target-fp 6604a2af482f0cf7 --message "ping"` produces a reply within ~30s.\n  **If not:** Re-check identity'
     [UNKNOWN    ] step 2: `tl-gibzucwp`
     [RISKY      ] step 2: `pkill -f 'termlink register'`
     [UNKNOWN    ] step 3: `termlink register --name agent-1 --identity-key ~/.termlink/identity.key --tags 'role:agent,host:dimitrixpro'`
     [SAFE       ] step 4: `termlink whoami`
     [UNKNOWN    ] step 4: `Identity FP: 6604a2af482f0cf7`
     [SAFE       ] step 5: `termlink agent contact --hub laptop-141 --target-fp 6604a2af482f0cf7 --message "ping"`
     [SAFE       ] step 5: `termlink remote list laptop-141`
     [UNKNOWN    ] step 5: `-`
     → step 4 exit=0 out: 'Multiple candidate sessions on this hub — which one are you?'
     → step 5 exit=0 out: 'Posted to dm:6604a2af482f0cf7:d1993c2c3ec44c94 — offset=4, ts=1780176529391'
     → step 5 exit=0 out: 'ID             NAME             FP                STATE          PID      TAGS'

## T-1696 — Cron entry installed in /etc/cron.d on .107

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **FAIL**  (safe cmds returned non-zero exits: [1, 0])
   commands: 5 parsed (safe=2, risky=3, interactive=0, unknown=0)
   expected: 'step 1 prints IDENTICAL; step 2 shows recent drift entries (the canary detecting the live G-058 mirror lag — that is the canary working, not a canary failure)\n '
     [SAFE       ] step 1: `diff /etc/cron.d/termlink-release-mirror-canary /opt/termlink/.context/cron/release-mirror-canary.crontab && echo IDENTI`
     [SAFE       ] step 2: `tail -3 /opt/termlink/.context/working/.release-mirror-canary.log`
     [RISKY      ] step 3: `sudo cp /opt/termlink/.context/cron/release-mirror-canary.crontab /etc/cron.d/termlink-release-mirror-canary`
     [RISKY      ] step 3: `sudo systemctl reload cron`
     [RISKY      ] step 3: `service cron reload`
     → step 1 exit=1 out: '17a18,24'
     → step 2 exit=0 out: '  origin (OneDev): bf8a623611a794f2cb99a953858a9f8bfc792de0'

## T-1722 — Upstream landed on `/opt/999-AEF`

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **INCONCLUSIVE**  (safe ran but ambiguous evidence (1 risky residue))
   commands: 2 parsed (safe=1, risky=0, interactive=0, unknown=1)
   expected: 'OneDev master SHA matches the Channel-1 push.\n  **If not:** Re-fire the Channel-1 push via `termlink_run` from /opt/termlink; check upstream OneDev for branch-p'
     [SAFE       ] step 1: `git ls-remote https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework master | awk '{print $1}'`
     [UNKNOWN    ] step 2: `termlink_run`
     → step 1 exit=0 out: '20b082fe6cdf2953bc65ff1553749b137532fa88'

## T-1723 — Cron entry installed on .107 so the meta-canary

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **FAIL**  (safe cmds returned non-zero exits: [1])
   commands: 3 parsed (safe=1, risky=2, interactive=0, unknown=0)
   expected: 'The grep returns the new meta-canary line.\n  **If not:** Inspect `/etc/cron.d/termlink-release-mirror-canary` for syntax / permission issues; cron does NOT load'
     [RISKY      ] step 1: `sudo cp /opt/termlink/.context/cron/release-mirror-canary.crontab /etc/cron.d/termlink-release-mirror-canary`
     [RISKY      ] step 2: `sudo systemctl reload cron`
     [SAFE       ] step 3: `grep aliveness /etc/cron.d/termlink-release-mirror-canary`
     → step 3 exit=1 out: ''

## T-1836 — MCP listing shows the three new tools

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **FAIL**  (safe cmds returned non-zero exits: [2, 0])
   commands: 5 parsed (safe=2, risky=0, interactive=0, unknown=3)
   expected: 'All three names appear\n  **If not:** Build failed or registration missing — re-run cargo build with -vv'
     [SAFE       ] step 1: `termlink mcp`
     [UNKNOWN    ] step 1: `agent_listeners`
     [UNKNOWN    ] step 1: `listener_heartbeat`
     [UNKNOWN    ] step 1: `agent_send_auto_discover`
     [SAFE       ] step 2: `strings target/debug/termlink | grep -E 'termlink_(listener_heartbeat|agent_listeners|agent_send_auto_discover)'`
     → step 1 exit=2 out: ''
     → step 2 exit=0 out: ' purely MCP-side composite read.termlink_agent_followupsDiscover active agent-presence listeners (T-1830/T-1833, MCP wrapper from T-1836). R'

## T-1841 — Skill discoverable and invokable from Claude Code

   class:    RUBBER-STAMP-MECHANICAL
   verdict:  **PASS-LOOSE**  (2 safe cmds clean + Expected matched, but 1 risky/interactive remain)
   commands: 3 parsed (safe=2, risky=0, interactive=0, unknown=1)
   expected: 'Step 3 shows status=LIVE for your agent_id; after stop, state file is gone and a subsequent listeners scan shows the agent OFFLINE within ~150s.\n  **If not:** C'
     [SAFE       ] step 3: `bash scripts/agent-listeners.sh --filter-agent-id $(jq -r .agent_id ~/.termlink/be-reachable.state) --json | jq .`
     [UNKNOWN    ] step 4: `~/.termlink/be-reachable.state`
     [SAFE       ] step 4: `ps -fp $(jq -r .pid ~/.termlink/be-reachable.state)`
     → step 3 exit=0 out: '{'
     → step 4 exit=0 out: 'UID          PID    PPID  C STIME TTY          TIME CMD'

## T-1417 — Audit shows zero `event.broadcast` callers

   class:    OBSERVE-INFRA
   verdict:  **INCONCLUSIVE**  (safe ran but ambiguous evidence (3 risky residue))
   commands: 4 parsed (safe=1, risky=1, interactive=0, unknown=2)
   expected: "Zero event.broadcast lines from this host's own sessions in the audit (other-host sessions like ring20-dashboard handled separately by their own upgrade)\n  **If"
     [UNKNOWN    ] step 1: `cargo build --release && cp target/release/termlink ~/.cargo/bin/termlink`
     [RISKY      ] step 2: `pkill -f 'termlink hub' && termlink hub start --tcp 0.0.0.0:9100 --json &`
     [SAFE       ] step 4: `fw metrics api-usage --cut-ready --json`
     [UNKNOWN    ] step 4: `legacy_callers_by_ip`
     → step 4 exit=0 out: '{"cut_ready": true, "window_days": 7, "legacy_attributable": 0, "legacy_unattributable_pre_t1409": 0, "audit_file": "/var/lib/termlink/rpc-a'

## T-1419 — freshness signal correctly distinguishes

   class:    OBSERVE-INFRA
   verdict:  **INCONCLUSIVE**  (safe ran but ambiguous evidence (1 risky residue))
   commands: 2 parsed (safe=1, risky=0, interactive=0, unknown=1)
   expected: '`last_seen_iso` for .143 is BEFORE the deploy timestamp (i.e., no calls after restart). Count may still be non-zero (rolling window).\n  **If not:** the upgrade '
     [SAFE       ] step 3: `fw metrics api-usage --last-Nd 1 --json | python3 -c "import json,sys; d=json.load(sys.stdin); [print(x) for x in d['leg`
     [UNKNOWN    ] step 3: `last_seen_iso`
     → step 3 exit=0 out: ''

## T-1137 — CT 200 (.122) stops rebooting

   class:    OBSERVE-INFRA
   verdict:  **OPERATOR-ONLY**  (3 commands all risky/interactive — orchestrator surfaces only)
   commands: 3 parsed (safe=0, risky=0, interactive=0, unknown=3)
   expected: 'CT uptime > 24h, no new boots, ring20-management [PASS]\n  **If not:** Other resource pressure still present — investigate memory, CPU, or disk on pve host'
     [UNKNOWN    ] step 1: `ssh root@192.168.10.180 pct status 200`
     [UNKNOWN    ] step 2: `ssh root@192.168.10.180 journalctl --list-boots -n 10`
     [UNKNOWN    ] step 3: `cd /opt/termlink && termlink fleet doctor`

---

# Summary

| Verdict | Count | Examples |
|---|---:|---|
| PASS-ROBUST | 0 | — |
| PASS-LOOSE | 4 | T-1296, T-1296, T-1457 |
| OPERATOR-ONLY | 3 | T-1420, T-1420, T-1137 |
| FAIL | 5 | T-1296, T-1431, T-1696 |
| INCONCLUSIVE | 4 | T-1420, T-1722, T-1417 |

**Auto-validatable (PASS-ROBUST + PASS-LOOSE): 4/16 = 25%**
**Operator-only surface (cannot auto-validate): 3/16 = 19%**

A-025 GO threshold: ≥15 of 47 mechanically validatable. With current sample: 4 PASS — NEEDS-MORE-EVIDENCE.
