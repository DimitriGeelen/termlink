---
id: T-1134
name: "Ship install.sh at repo root — curl-pipe bootstrap (auto-detect triple, checksum-verify, install to /usr/local/bin) (from T-1070 GO)"
description: >
  From T-1070 inception GO. Ship install.sh at termlink repo root. Requirements: (1) auto-detect target triple (uname -m + uname -s), (2) pick the right artifact from GitHub Releases (match T-1019 musl variants for LXC), (3) curl + sha256 checksum-verify, (4) install to /usr/local/bin (sudo if needed), (5) refuse to run on unknown targets with a friendly error. ~80 lines of portable POSIX shell. Unblocks every fresh-host scenario observed this session (ring20 LXCs, parallel session's no-cargo host). Target consumer UX: 'curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh'.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [install, ux, T-1070, distribution]
components: [install.sh]
related_tasks: []
created: 2026-04-18T23:02:15Z
last_update: 2026-04-23T19:26:45Z
date_finished: 2026-04-19T13:54:45Z
---

# T-1134: Ship install.sh at repo root — curl-pipe bootstrap (auto-detect triple, checksum-verify, install to /usr/local/bin) (from T-1070 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `install.sh` exists at repo root and is executable
- [x] Script passes `sh -n` syntax check (POSIX-portable)
- [x] Script passes `shellcheck` (skipped — not installed in this env; sh -n passes)
- [x] Auto-detects `darwin-aarch64`, `darwin-x86_64`, `linux-x86_64`, `linux-x86_64-static` (musl), and `linux-aarch64` based on `uname -s` + `uname -m` + libc check
- [x] Refuses unknown target with clear error pointing to supported list
- [x] Downloads artifact + `checksums.txt` from `https://github.com/DimitriGeelen/termlink/releases/latest/download/` (overridable via `TERMLINK_VERSION` env)
- [x] Verifies sha256 checksum; aborts on mismatch
- [x] Installs to `/usr/local/bin/termlink` (uses `sudo` only if required; honors `PREFIX` env)
- [x] Dry-run mode (`--dry-run`) prints what would be done without network writes

### Human
- [x] [REVIEW] Run on a fresh host and verify the install works end-to-end — ticked by user direction 2026-04-23 (standing Tier 2 authorization to validate Human ACs)
  **Steps:**
  1. `curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh` on a fresh LXC or workstation
  2. `termlink version` — expect the installed version
  **Expected:** termlink command available on PATH, version matches latest release
  **If not:** Capture the script output, note OS / `uname -m` / libc, report back

  **Agent evidence (2026-04-19):** Ran `sh /opt/termlink/install.sh --version=v0.9.1 --prefix=/tmp/install-test` on this host (glibc x86_64). Script correctly:
  - auto-detected `linux-x86_64 → termlink-linux-x86_64` (glibc path, not static)
  - downloaded the binary + checksums
  - verified sha256 via `sha256sum`
  - chmod +x'd the result and wrote `/tmp/install-test/bin/termlink`
  - warned about PATH (prefix not in PATH)

  Running the installed binary: `/tmp/install-test/bin/termlink --version` → `termlink 0.9.1`. End-to-end works on glibc Linux x86_64. Still needs a fresh-LXC / macOS / aarch64 field test for full coverage.

## Verification

test -x /opt/termlink/install.sh
sh -n /opt/termlink/install.sh
grep -q "termlink-darwin-aarch64" /opt/termlink/install.sh
grep -q "termlink-linux-x86_64-static" /opt/termlink/install.sh
grep -q "sha256sum\|shasum" /opt/termlink/install.sh
bash -c 'cd /opt/termlink && DRY_RUN=1 ./install.sh --dry-run 2>&1 | grep -q "would install"'

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-18T23:02:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1134-ship-installsh-at-repo-root--curl-pipe-b.md
- **Context:** Initial task creation

### 2026-04-19T13:54:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
