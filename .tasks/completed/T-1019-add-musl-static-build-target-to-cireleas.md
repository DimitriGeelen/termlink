---
id: T-1019
name: "Add musl static build target to CI/release pipeline"
description: >
  Deploying termlink to Debian 12 hosts (.109, .121) required a manual musl static build because the glibc-linked binary needs glibc 2.38+ (Ubuntu) while Debian 12 has 2.36. The CI/release pipeline should produce a musl static binary alongside the dynamic one so cross-distro deployment works out of the box.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:01:23Z
last_update: 2026-04-23T18:41:43Z
date_finished: 2026-04-13T12:12:22Z
---

# T-1019: Add musl static build target to CI/release pipeline

## Context

Add x86_64-unknown-linux-musl to the GitHub Actions release workflow so that cross-distro deployment works without glibc version issues. The musl binary is statically linked and works on any Linux distro.

## Acceptance Criteria

### Agent
- [x] release.yml includes x86_64-unknown-linux-musl in the Linux build matrix
- [x] musl build installs musl-tools and uses the musl target
- [x] Release job includes termlink-linux-x86_64-static in the artifacts list
- [x] YAML is valid

### Human
- [x] [RUBBER-STAMP] Verify musl binary appears in next release after tagging — ticked by user direction 2026-04-23. Evidence: live curl of GitHub release v0.9.1 checksums.txt returns sha256 e5e0ded0...288 for termlink-linux-x86_64-static. musl variant is published. Verified 2026-04-23T17:50Z.
  **Steps:**
  1. Tag a release: `cd /opt/termlink && git tag v0.X.Y && git push origin --tags`
  2. Check GitHub Actions for successful build
  3. Verify release page has `termlink-linux-x86_64-static`
  **Expected:** Static binary available alongside dynamic binaries
  **If not:** Check Actions logs for musl build failures


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, live-termlink, musl-static-release):** Live: `curl -sfL https://github.com/DimitriGeelen/termlink/releases/download/v0.9.1/checksums.txt | grep static` → `e5e0ded04d6e0c5d2257e844416ca7b296135fcad19c0309760abe41a7f2e288  termlink-linux-x86_64-static`. musl-static variant is published in the release. RUBBER-STAMPable.

## Verification

python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
grep -q "x86_64-unknown-linux-musl" .github/workflows/release.yml
grep -q "termlink-linux-x86_64-static" .github/workflows/release.yml

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

### 2026-04-13T12:01:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1019-add-musl-static-build-target-to-cireleas.md
- **Context:** Initial task creation

### 2026-04-13T12:12:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T21:04:36Z — programmatic-evidence [T-1090]
- **Evidence:** x86_64-unknown-linux-musl target present in .github/workflows/release.yml with musl-tools build deps
- **Verified by:** automated command execution
