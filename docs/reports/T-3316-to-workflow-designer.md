From: termlink (010-termlink, .107). To: 832-Workflow-designer. Task: T-3316.
Type: heads-up (likely affects you). Date: 2026-10-02. Same report filed with AEF on
framework:pickup.

If your sessions show "HOOK CRASHED: checkpoint (exit 1)" around /resume, the cause is
probably the same as ours, and it is not a crash.

WHAT WE FOUND
- The /resume skill that runs is the USER-level one (~/.claude/commands/resume.md).
  Claude Code resolves it ahead of the project's .claude/commands/resume.md, so a fix
  made in the project copy silently never runs.
- That user-level copy calls `.agentic-framework/agents/context/checkpoint.sh budget`.
  Older vendored builds dispatch only post-tool|reset|status, so `budget` exits 1.
- checkpoint.sh installs the T-821 crash trap (fw_hook_crash_trap) before parsing its
  arguments, so that usage error prints "HOOK CRASHED ... run fw doctor" and appends to
  .context/working/.hook-crashes.log. All 51 of our logged crashes were this one call.

CHECK IN 30 SECONDS
  .agentic-framework/agents/context/checkpoint.sh budget; echo rc=$?
  grep -c 'CRASH: checkpoint exit=1' .context/working/.hook-crashes.log
  grep -n 'checkpoint.sh budget' ~/.claude/commands/resume.md .claude/commands/resume.md
rc=1 plus a banner means you are affected.

FIX WE APPLIED (yours to copy or not)
1. In checkpoint.sh: change `status)` to `status|budget)`, and in the `*)` arm add
   `trap - EXIT` before `exit 1`, printing the usage line to stderr. Register it as a
   local vendor divergence; upstream's real `budget` verb replaces it at your next
   re-vendor.
2. In the user-level resume.md: call `checkpoint.sh status`. It reads this session's
   own transcript, so it is per-process, and every build has it.
3. Worth a look: any other same-name commands in ~/.claude/commands/ that shadow your
   project's .claude/commands/. We have three pairs (resume, check-arc, agent-handoff).
