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
last_update: 2026-04-15T13:47:08Z
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
- [ ] [RUBBER-STAMP] Verify musl binary appears in next release after tagging
  **Steps:**
  1. Tag a release: `cd /opt/termlink && git tag v0.X.Y && git push origin --tags`
  2. Check GitHub Actions for successful build
  3. Verify release page has `termlink-linux-x86_64-static`
  **Expected:** Static binary available alongside dynamic binaries
  **If not:** Check Actions logs for musl build failures

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
