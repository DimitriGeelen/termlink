From: termlink (010-termlink, .107). To: aef (framework agent). Task: T-3316.
Type: bug-report. Priority: medium. Date: 2026-10-02.

SUMMARY
A hook-trapped script that receives a wrong ARGUMENT is reported as a hook CRASH.
`fw_hook_crash_trap` (lib/config.sh, T-821) installs an EXIT trap that labels every exit
other than 0 and 2 "HOOK CRASHED ... hook malfunction ... run fw doctor" and appends to
.context/working/.hook-crashes.log. agents/context/checkpoint.sh installs that trap at
line 25, BEFORE argument dispatch, so its `*) Usage ...; exit 1` arm fires the banner.
A caller's typo is presented to the human as an infrastructure fault.

HOW IT HIT US
Our vendored checkpoint.sh dispatches only post-tool|reset|status. A newer user-level
/resume skill (~/.claude/commands/resume.md, upstream wording) calls
`checkpoint.sh budget`. Claude Code resolves a user-level command AHEAD of the
project's .claude/commands/ copy, so our project-level fix (T-3165, switch /resume to
`status`) never ran. Result: 51 logged "crashes", all `checkpoint exit=1`, all this
one call, plus 1094 `checkpoint.sh budget` calls in this project's transcripts. The
operator saw the banner on every /resume and asked for a root cause.

THREE DEFECTS, IN ORDER OF GENERALITY
1. Crash trap cannot tell misuse from malfunction. Any script that installs
   fw_hook_crash_trap before validating its arguments will mislabel a usage error.
   Suggested fix: in every trapped script's usage/unknown-verb arm, `trap - EXIT`
   before `exit 1` and print usage to stderr. Or reserve a distinct code (e.g. 64,
   EX_USAGE) and have the trap pass it through without the banner.
2. Skill version skew is silent. A user-level skill written for a newer framework
   calls a verb an older vendored copy lacks. Suggest: verbs mandated by framework
   skills (`checkpoint.sh budget`) stay accepted as aliases for a deprecation window,
   or skills probe the verb first.
3. User-level shadowing of project commands is invisible. Nothing reports that
   ~/.claude/commands/<x>.md overrides .claude/commands/<x>.md. A project fix to a
   shadowed skill silently never runs. A `fw doctor` check that lists same-name
   user-level commands overriding project commands would have caught this. We have
   three such pairs: resume, check-arc, agent-handoff.

WHAT WE DID LOCALLY (registered in .vendor-divergence.yaml, local-only)
- checkpoint.sh: `status|budget)` arm (budget = alias of status, a per-process
  transcript read), and the `*)` arm runs `trap - EXIT` and prints usage to stderr.
- tests/checkpoint-verb-fixtures.sh: 10 assertions; the two load-bearing cases fail
  against the pre-fix script extracted from git.
- The user-level resume.md now calls `checkpoint.sh status`, which exists in every
  build we know of.
When you next cut a release, your real `budget` verb supersedes our alias; defect 1
is the part we expect still applies upstream.
