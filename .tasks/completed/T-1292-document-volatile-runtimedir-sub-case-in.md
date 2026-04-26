---
id: T-1292
name: "Document volatile-runtime_dir sub-case in Hub Auth Rotation Protocol"
description: >
  Add a Special case block under CLAUDE.md Hub Auth Rotation Protocol covering the degenerate scenario where runtime_dir defaults to /tmp/termlink-0 on a host with tmpfs /tmp — symptom is both TLS+secret rotating together on reboot. Encodes the T-1290 finding so future agents recognize the pattern faster. Survives T-1290 spike 1 outcome since the pattern is real regardless of whether .122 specifically turns out to be this case.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [docs, auth, claude-md]
components: []
related_tasks: [T-1290, T-935, T-1051]
created: 2026-04-26T11:51:49Z
last_update: 2026-04-26T11:52:28Z
date_finished: 2026-04-26T11:52:28Z
---

# T-1292: Document volatile-runtime_dir sub-case in Hub Auth Rotation Protocol

## Context

T-1290 (inception, in flight) gathered evidence that ring20-management's recurring secret rotation likely traces to a tmpfs runtime_dir. CLAUDE.md's "Hub Auth Rotation Protocol" enumerates three rotation scenarios but does not call out the volatile-runtime_dir degenerate sub-case. Future agents hitting PL-021 (both-secret-AND-cert rotation) should jump straight to "check runtime_dir" instead of repeating the investigation.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md "Hub Auth Rotation Protocol" gains a "Special case — volatile runtime_dir (T-1290)" block describing symptom (BOTH TLS fingerprint AND HMAC secret rotate together), diagnostic (`ls /tmp/termlink-0/ /var/lib/termlink/` + `mount | grep termlink`), and fix (T-935 systemd-unit migration)
- [x] Block references PL-021 by name so the symptom-recognition path closes
- [x] Block notes that the pattern is real regardless of T-1290 spike 1 outcome (i.e. this doc is not contingent on T-1290's GO/NO-GO)

## Verification

# No shell commands — pure documentation change.

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

### 2026-04-26T11:51:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1292-document-volatile-runtimedir-sub-case-in.md
- **Context:** Initial task creation

### 2026-04-26T11:51:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T11:52:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
