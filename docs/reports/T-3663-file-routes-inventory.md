# T-3663: inventory of routes that move file bytes between machines (2026-11-03)

Read-only inventory. No route was changed and no file was moved. Secrets were not read or printed: this report gives
paths and mechanisms only. The rule it is measured against is **O-1** (T-3353 Decisions, 2026-11-03; architecture doc
6.4.1 and 6.4.6). Under O-1, a file that crosses to another machine must not be readable there by anyone except its
recipient. That means it is encrypted to the recipient, or it travels end to end without an intermediate store that can
read it. O-1 currently binds only new `artifact put --hub`. This inventory covers everything else.

Hosts: .107 (this workstation, hub `:9100`), .121 ring20-dashboard, .122 ring20-management (hub `:9100`, node1-pool hub
`:19100`), .132 greenfield laptop, .141 laptop, NAS .210, node1 (VPS incus CT), plus .201 (OneDev / Docker host), which
the answers showed is also on routes.

Evidence types: **M** = measured on .107 today (command or file:line). **P** = a peer's answer on conversation
`file-routes-inventory-2026-11-03`, not verified by us.

Candidate way under O-1: **encrypt-to** (wrap to the recipient, as in 6.4.3); **keep+TLS/SSH** (with the reason); **retire**.

## A. TermLink routes (our code)

| # | Route | Users | Payload | Transport / protection | Where bytes rest, who can read them | Evidence | Candidate |
|---|---|---|---|---|---|---|---|
| A1 | `termlink file send` / `remote send-file` (artifact path first, then legacy `file.init`/`file.chunk` events) | any agent or operator; T-3661 users | arbitrary files | TLS to the hub (TCP listener is always TLS, TOFU-pinned) | **plaintext** in the hub's artifact store `<runtime>/bus/artifacts/` (on .107: under `/var/lib/termlink` 0700 root; objects 0644; 6 objects, 64 MB today). An offline target's chunks are spooled as plaintext JSON in `<runtime>/inbox/<target>/`. Readable by hub root and by any client that holds a hub secret or token and knows the sha256. The receiver writes with `fs::write`, so the file gets umask mode in a dir the caller chooses. | `crates/termlink-cli/src/commands/file.rs:128` (try artifact), `:582`; `crates/termlink-hub/src/channel.rs:188`; `crates/termlink-hub/src/inbox.rs:53`; `crates/termlink-hub/src/server.rs:497`; `stat`/`find` on `/var/lib/termlink` | **encrypt-to**: same wrapper as 6.4.3. This is the route the O-1 "left open" note names first |
| A2 | `termlink artifact put/get --to <peer>` (same hub) | sidecar tooling, arc-011 S2, 9 allow-listed writers (6.4.5) | blobs | TLS to the hub | the same plaintext artifact store as A1 | `crates/termlink-cli/src/commands/artifact.rs:110,241`; `crates/termlink-hub/src/artifact.rs:1-13` | **encrypt-to** (B-11 wrapper; the writer check already exists, `scripts/check-artifact-writers.sh`) |
| A3 | `artifact put --hub <addr>` (cross-hub, T-3614) | ring20 asked for it | blobs | not built yet (no `--hub` flag in 0.12.1526) | n/a | `termlink artifact put --help` | **already ruled by O-1**: encrypted only |
| A4 | `termlink remote push <hub> <session> <file>` | operators, older scripts | text files (`read_to_string`) | file bytes base64-encoded **inside a remote-exec shell command**; TLS to the hub | target host `/tmp/termlink-inbox/<name>` written by `base64 -d >`, so it gets the default umask (normally world-readable in /tmp) and is never removed. While the command runs, the payload is also visible in the target's process list. | `crates/termlink-cli/src/commands/push.rs:9` (`INBOX_DIR=/tmp/termlink-inbox`), `:89-91` (write_cmd) | **retire** (replace with A1 after encrypt-to) |
| A5 | `termlink remote exec` stream (generic) | runme, fleet scripts, operators | command text plus stdout: any file a command `cat`s comes back this way | TLS to the target's hub | transient; the hub's RPC audit logs no params (per `scripts/fleet-token-remint.sh:23`) | `crates/termlink-cli/src/commands/remote.rs:2170` | **keep+TLS** as a control channel; stop using it as a file route (see A6, A7) |
| A6 | runme **action 6** fleet hub upgrade (`fleet-deploy-binary.sh`) | runme on .107 → ring20-management .122, ring20-dashboard .121, greenfield .132 | termlink musl binary (public content) | base64 chunks over A5, TLS; sha256 compared before streaming; afterwards version, `fleet doctor`, and `tofu verify` | `/tmp/.fleet-deploy.<pid>/` chunks (removed), `/tmp/termlink.new`, then `install -m 755` to the unit's binary path | `runme.sh:1144` (hubs), `:1151` (STAGED), `:1207`, `:1215`; `scripts/fleet-deploy-binary.sh:123,161,193` | **keep+TLS**. The content is public and integrity matters, not secrecy. Later: a signed binary checked on the target |
| A7 | **fleet-token-remint** (cron daily 03:40 on .107; runme 22b) | .107 → each fleet hub host | Interact-scope hub token (a secret) | carried **inside the remote-exec command** (TLS on the wire; visible in the host's process list while it runs, a residual the script documents) | host `/root/.termlink/secrets/107-hub.token` 0600, plus a token-only `hubs.toml`; scratch on .107 is 0700 and removed | `.context/cron/fleet-token-remint.crontab` (`40 3 * * *`); `scripts/fleet-token-remint.sh:9-23,71,109` | **keep+TLS** (the secret has to arrive as plaintext to be used). Move it off the command line (stdin), or **encrypt-to** the host key once K-24 keys exist |
| A8 | channel posts with inline payloads (`pool:in` judge bundles base64 ≤ 393,216 B to node1-pool; screenshots in base64 parts to greenfield) | ring20 / .107 → node1-pool hub (.122:19100), → .132 | bundles, images | TLS to the hub | **plaintext** in the hub topic store (bus log), readable by subscribers who hold a token; node1 also keeps them in `/var/lib/orch` | P (ring20 items 4, 6); hub TLS M (`server.rs:497`) | **encrypt-to** (this is FX-5A growth; the message-field size cap in 6.4.5 is what stops tunnelling) |
| A9 | `scripts/deploy-remote.sh` (legacy) | operator, manual | termlink binary + systemd unit | `scp` / `ssh cat >` as root; it reads the remote `hub.secret` back over ssh into a shell variable | target root fs | `scripts/deploy-remote.sh:56,67,98`; last touched 2026-11-01 (210d95951); no caller in runme | **retire** (action 6 replaces it). If kept, stop pulling `hub.secret` back |
| A10 | 055 **cockpit-gate** (ssh forced command, `from="192.168.10.107",restrict`) | 055 cockpit on .107 → fleet machines incl. Windows/WSL/macOS | terminal screens (`pty mirror`), keystrokes (`steer` only if `STEER=1`) | SSH | nothing stored; syslog records verb, session and client, no content | `/opt/055-agentic-fleet-cockpit/deploy/win-gate/cockpit-gate:2-21` | **keep+SSH**: no file verb. But `steer` can `cat` a file onto the screen, so note it as a side channel |

## B. Host-level routes on .107 (not TermLink)

| # | Route | Users | Payload | Transport / protection | Where bytes rest, who can read them | Evidence | Candidate |
|---|---|---|---|---|---|---|---|
| B1 | sshfs `/NasRing20` = `adminDimitri@.210:/` | 1409 (default mount), anche NAS dir (path default), nas-file-service, operator | anything on the NAS | SSH | NAS: plaintext. On .107 the mount has `allow_other` plus `default_permissions`, so every local user gets what the NAS file modes allow, through the NAS **admin** login | `mount`; `findmnt -no OPTIONS /NasRing20`; `systemctl cat nasring20` (drop-in `t2486-key.conf`) | **keep+SSH** for transit. At rest: drop `allow_other` or switch the users to B2. The admin login on a standing mount is the exposure |
| B2 | sshfs `/NasRing20-svc` = `svc-107-backup@.210:/` (root only) | `1409-sicherung` (rsync `--delete` mirror, daily 00:31), `anche-backup` (daily 00:00) via drop-ins `t2486-svc-mount.conf` | raw data mirror (1409), SQLite snapshot (anche) | SSH. **ring20 called this CIFS; it is sshfs (measured)** | NAS: **plaintext** backups, readable by the NAS accounts with access to those paths | `systemctl cat nasring20-svc 1409-sicherung anche-backup`; drop-in `Environment=SICHERUNG_MOUNT=/NasRing20-svc` | **encrypt-to** at rest (age to the operator, as Keycloak in C4 already does). Transit is fine |
| B3 | Pen CIFS `/mnt/nas210-pen-read` (ro), `/mnt/nas210-pen-create` (rw), share `Dimitri_Share_Sync` | 050 Pen (T-2581) | the operator's share files | SMB 3.1.1. **Encryption negotiated AES128-GCM (measured)**, but the mount has no `seal`, so encryption is not *required* | NAS plaintext; on .107 uid 1000, modes 0440/0660 | `mount`; `/proc/fs/cifs/DebugData` ("Encryption: Negotiated cipher (AES128-GCM)") | **keep+SMB3 encryption**: add `seal` so it cannot fall back. These are the operator's own files in their own share |
| B4 | **anche-backup**: `docker -H tcp://192.168.10.201:2375 cp` of the tracker DB → .107 → NAS | `/opt/109-anche-payments/deploy/backup-remote-db.sh`, daily | payments tracker SQLite DB | **Docker TCP API: plain HTTP, no auth** (.201 → .107) | .107 local dir, then NAS (B2) | `backup-remote-db.sh:14,22,33` | **retire this transport**: use `docker -H ssh://` or TLS. **Clearly exposed** |
| B5 | .107's **own Docker TCP API** `*:2375`, UFW allows `192.168.10.0/24` | anyone on the LAN | any file in or out of .107 (`docker cp`, bind mounts) as root | plain HTTP, **no auth** | .107 root filesystem | `ss -tlnp` (dockerd :2375); `ufw status verbose` | **retire** (bind to a unix socket or TLS). **Clearly exposed**; not a "file route" anyone documented, but it is one |
| B6 | nas-file-service `uvicorn 0.0.0.0:8420` (read/write/copy/transfer over the NAS mounts) | local tools (075/nas-file-service) | NAS files | HTTP, **no auth dependency found** in routes; UFW default deny with no 8420 rule, so it is not reachable from the LAN today | NAS (B1) | `ss -tlnp`; `ufw status verbose`; `/opt/nas-file-service/src/nas_file_service/api/routes/files.py:162,216`, `transfers.py:86,118` | **keep, local only**: bind to 127.0.0.1 so one UFW change cannot expose it |
| B7 | inbound ssh to .107 root | keys `claude-dev@ring20`, `proxmox-admin@lxc-205` (comments only) | runme payloads, tar-over-ssh, a screenshot pulled from `~/Downloads` (2026-11-02) | SSH, UFW LAN only | .107 /tmp or as written | `/root/.ssh/authorized_keys` (comments); P (ring20 item 2) | **keep+SSH** |
| B8 | git push from .107 | this repo | code, task files, `.context` | `origin` HTTPS to `onedev.docker.ring20…`; `github` HTTPS | OneDev on .201, GitHub (public repo) | `git remote -v` | **keep+TLS** |

## C. Peer routes (answers received so far; P = unverified by us)

| # | Route | From → to | Payload | Transport | At rest / readers | Evidence | Candidate |
|---|---|---|---|---|---|---|---|
| C1 | git push / fetch | .121 and .122 → OneDev `.201:6610` | full repos | **plain HTTP on the LAN** (token auth, so the token is in clear too) | OneDev, project members | P ring20-dashboard 1, ring20-mgmt 10 | **keep with TLS**: switch the remotes to the HTTPS proxy, as .107 already does. **Exposed** |
| C2 | WireGuard conf download | vpn-api `http://192.168.10.25:5001` → .132 | conf **incl. private key** | **plain HTTP** | `C:\ProgramData\ring20-vpn` (SYSTEM+Admins) | P ring20-mgmt 5 | **retire** plain HTTP (TLS, or deliver by SSH). **Exposed** |
| C3 | PBS sync replica → NAS NFS `/mnt/replica` (CT118) | .122 PBS → .210 | VM/CT backup chunks, **unencrypted**; secret groups (ct/101,105,220,221,301,302) excluded | NFS, no transit encryption | NAS, plaintext | P ring20-mgmt 7 | **encrypt-to** (PBS client-side encryption) or keep with the exclusion as a recorded acceptance. **Exposed in transit** |
| C4 | Keycloak backups | CT105 → .121 (`ssh cat`) → NAS NetBackup | pg_dump + realm, **age-encrypted on the CT** | SSH | ciphertext only; the age identity is not on .121/CT105 | P ring20-dashboard 4, ring20-mgmt 9 | **fine**: this is the model for B2 |
| C5 | Keycloak step scripts; CA key manual scp | .121 → CT105; operator → CT105 | scripts; CA key | SSH (tar, pinned host key, sha256 -c) | root on CT105, key 0600 | P ring20-dashboard 3, 5 | **keep+SSH** |
| C6 | ring20 admin ssh/scp, `pct push`, `qm guest exec --pass-stdin`, `incus file push` | .122 → PVE .180/.81/.185/.195, CTs/VMs, node1, .107, .132 | scripts, configs, tarballs | SSH | root on the targets | P ring20-mgmt 1-3, 5 | **keep+SSH** |
| C7 | Watchtower `.122:3000` | .122 → LAN readers (.121) | repo docs, task JSON | **plain HTTP**, LAN | n/a | P ring20-dashboard 6, ring20-mgmt 11 | **keep with TLS** (documents, read only; low) |
| C8 | Cloudron backups → off-site Synology `212.91.238.106` | Cloudron → internet | backups | sshfs (SSH) | off-site, encryption at rest unverified | P ring20-mgmt 12 | check whether Cloudron backup encryption is on; then keep |
| C9 | AEF upgrade fetch | GitHub → .121 | framework source | HTTPS | root on .121 | P ring20-dashboard 2 | **fine** |

Peers asked (16 projects, one message each, client_msg_ids in `.context/working/t3663/sends.log`). 12 were RECEIVED at
a local receiver and 4 went via the hub. Answered so far: **ring20-dashboard, proxmox-ring20-management**. Receipt only:
0909-aef-greenfield-v22. Still open: 999-AEF, 003, 0235, 050, 0506, 055, 0909-boot, 1023, 1409, 832,
aef-greenfield-test, dimitri-mint-dev, fan-dashboard. Their own scripts (for example 050 Pen's use of B3, 1409's B2,
055's A10) are covered above from .107 measurements. Answers that arrive later go into this table.

## Summary

**Clearly exposed** (readable in transit or by a party that is not the recipient):
1. **B5** Docker TCP API on .107 (`:2375`, LAN, no auth): root file access to .107 from any LAN host.
2. **B4** anche-backup pulls a payments DB over the unauthenticated plain Docker API on .201.
3. **C2** a WireGuard private key served over plain HTTP (vpn-api .25:5001).
4. **C1** git push to OneDev `.201:6610` over plain HTTP from .121/.122 (repos plus the auth token).
5. **C3** PBS replica to the NAS over NFS, chunks unencrypted (secret groups excluded).
6. **A1/A2/A8** TermLink file send, artifact store and inline channel payloads: TLS on the wire, but **plaintext at rest
   on the hub**, readable by hub root and token holders. This is exactly the FX-5A class that O-1 closes for `--hub`.
7. **A4** `remote push` leaves files world-readable in the target's `/tmp/termlink-inbox` and puts the payload in the
   process list.

**Weaker than it should be, not open**: A7 token on the remote command line; B1 NAS admin login on an `allow_other`
mount; B2 plaintext backups at rest on the NAS; B3 SMB encryption negotiated, not required; B6 HTTP file API bound
to 0.0.0.0 behind UFW only; A9 legacy deploy script that reads `hub.secret` back.

**Fine under O-1** (end to end SSH/TLS to the recipient, or ciphertext at rest): A6 binary upgrade (public content,
integrity checked), A10 cockpit gate, B7/C5/C6 SSH admin routes, B8 our HTTPS remotes, C4 age-encrypted Keycloak
backups, C9.

Suggested order for the plan (the operator decides): B5 and B4 (cheap, high exposure) → C2, C1 (peer-owned; tell
ring20) → A1/A2/A8 encrypt-to after B-11 keys exist → A4 retire → A7 stdin → B1/B2/B3 hardening.

## Status updates
- 2026-11-03 15:51Z: **B5 FIXED** (operator ruling A, T-3664, runme action 27): .107 dockerd on its local socket only; :2375 closed (verified ss + docker ps); all 3 containers back; backup /etc/docker/daemon.json.bak-20261103T155056Z.
- 2026-11-03: **C1 OneDev plain HTTP FIXED for 4 projects** (onedev-https-2026-11-03): 050 (HTTPS, .107 only; its CT 250 .129 deploy fetch still http, operator deferred), proxmox-ring20-management (HTTPS, push proven), ring20-dashboard (SSH, push proven; old http remote kept unused for a few days), dimitri-mint-dev (HTTPS, push proven).
## 0506-Voxtype-extention (msg 948500e9, 2026-11-03)
- One cross-machine route: git push over HTTPS from .107 /opt/0506-Voxtype-extention to OneDev (repo voxtype-review):
  source, task files, handovers, small synthetic WAV test fixtures. TLS in transit; at rest readable by anyone with
  access to that OneDev repo.
- Everything else stays on .107: dictation audio goes daemon -> 127.0.0.1:8095 (whisper) and Pen -> 127.0.0.1:8430
  (loopback HTTP only); temp WAVs are mode 600 and deleted after each call. Peer mail goes through the local hub.
  No scp/rsync, no mounts, no termlink file send or artifact uploads.

## 1023-portable-encrypted-chromium-vault (msg cd906dbd)
- In use today between machines: NONE (no git remote; tests on loopback; peer mail via the local hub).
- Designed, not deployed: rsync over SSH over WireGuard, device <-> canonical host (not assigned yet; test guest
  requested from ring20-manager). Payload: gocryptfs ciphertext of a Chromium profile + lease/heartbeat metadata.
  In transit: WireGuard + SSH (forced command, restricted account vaultsync). At rest: ciphertext only; the
  passphrase never leaves the devices.

## 999-Agentic-Engineering-Framework (msg ccacdd35)
1. git push over HTTPS .107 -> GitHub (since 2026-11-03 the SOURCE repo, remote origin; the repo is PUBLIC) and
   .107 -> OneDev (mirror, fw mirror sync every 15 min). Source, tasks, reports. TLS.
2. Consumers pull from GitHub over HTTPS (fw upgrade).
3. TermLink sidecar/hub messages between projects and hosts, sometimes whole files as bodies; at rest in each
   receiver's store under .context. AEF believes the transport is TLS with a pinned hub identity, not verified by AEF.
4. Review seats: briefs and the code they read go to external providers (OpenAI, Z.ai, Anthropic) over HTTPS.
5. The agent session itself: content goes to Anthropic's API over HTTPS.
6. No SMB/NFS, NAS, scp or rsync. One document went to claude.ai by mistake on 2026-11-02 and was deleted the same day.

## dimitri-mint-dev (msg c851c956)
1. SMB 3.1.1 mounts .107 -> NAS .210 (share Dimitri_Share_Sync at /mnt/nas210-pen-create rw and /mnt/nas210-pen-read
   ro): Pen's documents. No seal option, so encryption in transit is NOT verified.
2. sshfs .107 -> .210 as adminDimitri (/NasRing20, allow_other) and svc-107-backup (/NasRing20-svc): backups, SSH.
   allow_other exposes /NasRing20 to every local user on .107.
3. git push over PLAIN HTTP .107 -> 192.168.10.201:6610 (repo dimitri-mint-dev-workstation): NOT encrypted in transit.
4. termlink remote exec .107 -> .132:9100 (greenfield-exec, WSL root): commands and small writes (today an SSH public
   key); hub TLS with a pinned cert.
5. termlink sidecar/pickup messages (text only).
6. Occasional Artifact publish to claude.ai (private by default).
Not used: scp/rsync, NFS, termlink file send.

## 055-agentic-fleet-cockpit (msg d4d318f6)
1. git push over HTTPS .107 -> OneDev: commits and reports; TLS; at rest readable by repo users.
2. scp .107 -> .132 Windows OpenSSH (C:/Users/<win-user>/cockpit-gate-install-<date>/): a small install bundle (gate
   scripts + the cockpit's PUBLIC ssh key), only when the operator runs run-me.sh --laptop/--win-user; SSH.
Not file routes: the cockpit's ssh to remote gates carries terminal show/steer only. The file drop (T-691, live today)
is same-host only (.107 -> <project>/.context/inbox/files; root writes, owned by the project owner, mode 644, not
encrypted at rest). Planned: a cross-machine file drop (T-692, a 'receive' verb in their ssh gate, per machine,
operator-enabled). Nothing uses termlink file send / artifact put, SMB/NFS, the NAS or node1.

## fan-dashboard (msg 6b015c84)
1. git push/fetch .107 -> OneDev 192.168.10.201:6611 over SSH: repo contents; SSH-encrypted; at rest repo users.
2. Dashboard HTTP .107:8087 and Watchtower .107:3571 to LAN clients: JSON status and HTML only, no uploads; plain
   HTTP, not encrypted (status, not files).
3. Sidecar/TermLink peer messages via the local hub, text only.
No scp/rsync/SMB/NFS/termlink file send/artifact put; nothing to other hosts.

## 1409-sprind (msg ad1f2e25)
1. git push to OneDev over HTTPS: the whole repo (code, reports, evidence snapshots, tasks). TLS; at rest repo users.
   Push currently blocked by 2 REF audit fails.
2. Daily raw-data backup: systemd 1409-sicherung.timer runs tools/sicherung_rohdaten.sh --echt, which rsyncs --delete
   evidence raw data to NAS .210 NetBackup/1409-sprind/current via the sshfs mount (svc-107-backup). SSH in transit.
3. Read-only source corpus: sources/508-Sprin-D is a symlink into NAS .210 Dimitri_Share_Sync (sshfs as adminDimitri,
   allow_other); never written. SSH in transit.
4. node1: no direct route; one judge-bundle probe ran via the ring20 pool client, not by 1409.
5. TermLink sidecar: text only.
6. Outbound HTTPS paid reviews (OpenRouter/Mistral, tools/externe_review.py): task file + diff + named reports,
   operator-approved per round, logged with sha256.
None to .121/.122/.132/.141; no secrets moved (api-keys.enc is machine-bound).

## 050-email-archive / Pen (msg 2284f4c8; ignore its earlier 'test-resolve')
1. SMB/CIFS .107 -> NAS .210 Dimitri_Share_Sync: /mnt/nas210-pen-read (ro, pen-read, whole share) and
   /mnt/nas210-pen-create (rw, pen-create; writes only into the grant folders in policy/access-grants.yaml). Report
   drafts, receipts (PDF/DOCX/MD). SMB 3.1.1, no 'seal' option: encryption in transit depends on the NAS setting, not
   verified. At rest on the NAS share (its users) with btrfs snapshots.
2. sshfs .107 -> NAS .210: /NasRing20 (adminDimitri, the operator's admin mount; the agent never writes) and
   /NasRing20-svc (svc-107-backup, backups). SSH.
3. git push over plain HTTP .107 -> OneDev .201:6610 (email-archive.git): source + framework/task files, NOT encrypted
   in transit; token via a credential helper, not stored.
4. OneDev CI -> CT 250 .129: ssh as deploy, then git fetch from OneDev on .129 (prod /opt/penelope-prod, dev
   /opt/penelope): code only; the ssh leg is encrypted, the fetch leg is plain http.
5. scripts/pg-migrate-to-ring20.sh (scp/rsync of a PG dump to Ring20): exists, one-off, not routine.
6. termlink: messages/DMs only, via hubs .107/.122; no file send in routine use.
None to .121/.132/.141/node1.

## 832-Workflow-designer (msg 2f17dfaf)
1. git push/fetch over SSH .107 -> .201:6611: source, release artifacts (dist/), task and context files. SSH; at rest
   repo users.
2. Sidecar/TermLink messages via the hub, text only; peers sometimes send file contents as message text (owner
   approved). At rest in .context/sidecar/ on .107; client files copied only to a git-ignored build/peer-data/ dir
   (mode 600/700), never committed.
3. Watchtower HTTP on .107 itself only.
Not used: scp, rsync, SMB/NFS (the /mnt/nas210-* mounts are not touched by 832), termlink file send, artifact put.
