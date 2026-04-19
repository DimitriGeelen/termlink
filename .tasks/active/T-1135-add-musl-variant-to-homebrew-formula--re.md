---
id: T-1135
name: "Add musl variant to Homebrew formula — replace or supplement linux-gnu for LXC compatibility (from T-1070 GO)"
description: >
  From T-1070 inception GO. scripts/update-homebrew-sha.sh currently hashes 4 targets (darwin x86_64/aarch64 + linux-gnu x86_64/aarch64). Add musl variants (T-1019 artifacts) or swap linux-gnu → linux-musl for better LXC compatibility. Homebrew users on Linux LXC containers (no glibc) currently fall back to cargo build because the gnu binary fails to run. Touch: scripts/update-homebrew-sha.sh + homebrew-tap formula template. Verify: brew install termlink inside a fresh LXC succeeds.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [homebrew, install, T-1070, distribution]
components: []
related_tasks: []
created: 2026-04-18T23:02:30Z
last_update: 2026-04-19T13:56:52Z
date_finished: null
---

# T-1135: Add musl variant to Homebrew formula — replace or supplement linux-gnu for LXC compatibility (from T-1070 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Formula on_linux x86_64 branch points at `termlink-linux-x86_64-static` (musl, T-1019 artifact)
- [x] Formula aarch64 branch remains gnu (no musl aarch64 artifact is released yet)
- [x] `scripts/update-homebrew-sha.sh` hashes and writes the `-static` sha256 for the linux x86_64 entry
- [x] No formula sha256 or url line references `termlink-linux-x86_64` (non-static) for x86_64 Linux anymore

### Decisions

### 2026-04-19 — Homebrew linux x86_64 target choice
- **Chose:** Point Homebrew's linux x86_64 url at the musl static variant (`termlink-linux-x86_64-static`)
- **Why:** The release workflow produces both gnu and musl; Homebrew has no native way to pick per-host libc. Static works on both glibc and musl, including LXC minimal images where the gnu binary silently fails to run.
- **Rejected:** Keep gnu (breaks LXC — exact failure mode T-1070 documented). Ruby-side libc detection (brittle; outside formula convention).

### Human
- [ ] [REVIEW] `brew install termlink` works inside a fresh LXC
  **Steps:**
  1. On a minimal LXC (the .121 / .122 / ring20-management test hosts): `brew install DimitriGeelen/termlink/termlink`
  2. `termlink version`
  **Expected:** binary runs; version reports the installed tag
  **If not:** Note the LXC distro, `uname -m`, and error output; file back via termlink inject

## Verification

grep -q 'termlink-linux-x86_64-static' /opt/termlink/homebrew/Formula/termlink.rb
bash -c '! grep -E "termlink-linux-x86_64\"" /opt/termlink/homebrew/Formula/termlink.rb'
grep -q 'termlink-linux-x86_64-static' /opt/termlink/scripts/update-homebrew-sha.sh

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

### 2026-04-18T23:02:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1135-add-musl-variant-to-homebrew-formula--re.md
- **Context:** Initial task creation

### 2026-04-19T13:56:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
