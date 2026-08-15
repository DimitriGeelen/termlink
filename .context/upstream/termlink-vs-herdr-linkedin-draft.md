# LinkedIn draft — TermLink and herdr

**Status: DRAFT for human review. Not published. Nothing here should go out
without you reading the note at the bottom first.**

---

## The post

**Everyone's building runtimes for coding agents. Almost nobody's building the
part that comes after.**

herdr hit #1 on GitHub Trending recently and it deserves it — a single Rust
binary that keeps your agents' terminals alive through a closed lid, a dropped
network, an SSH disconnect. If you're running several agents and want to *watch*
them work, it's excellent, and the terminal-correctness work in it is real.

We've been building TermLink alongside it, and we ended up somewhere different —
so different that when we evaluated adopting herdr last week, the honest
conclusion was that we're not competitors so much as two halves of a problem.

The distinction is one line:

**herdr models a terminal a human is watching. TermLink models a command a
program consumes.**

That sounds academic until you try to build on it. Two things fall out
immediately:

→ **Exit codes.** TermLink's `exec` runs a real child process and returns its
real status. Not a marker string echoed to a screen and scraped back — the
actual exit code. If you've ever tried to build automation on top of a terminal
multiplexer, you know why that one line matters.

→ **Signals.** We deliver SIGINT/SIGTERM/SIGKILL to a specific process. Not
keystrokes into a tty and hope the foreground process is the one you meant.

Once you have those, you can build things that aren't possible otherwise:

• **Durable messaging between agents** — retention policies, acks, an offline
queue that absorbs a hub outage and replays exactly once when it comes back.
Not "prompt each other" — a message bus with delivery guarantees.

• **Lease-based work claims** — an agent claims a unit of work for a bounded
time, renews while it's still working, and releases when done. If it dies, the
lease lapses and the work reopens. No lost units, no double-processing.

• **Presence as a protocol, not a guess** — we know an agent is alive because it
heartbeats on a contract, with a canary watching for the case where it stops.
Not by reading its screen and inferring intent from what the last prompt looked
like.

None of that is a criticism of herdr. It's a different design centre, honestly
arrived at. They optimised for a human watching many agents; we optimised for
agents coordinating with each other and with the systems around them.

The part I'd actually argue for, though, isn't a feature.

Every one of those guarantees is backed by something that *fails loudly when it
breaks*. Seventeen scheduled canaries. Six static checks that run in CI and
block a release. An affirmative prover for each of our four core operations that
can be run on demand to answer "does this actually work right now?" — not "did
the tests pass at merge time."

Last week one of those checks caught a guard that had been reporting a populated
audit log as empty. Another proved that a coverage threshold in our own tooling
was mathematically unsatisfiable — it could never pass, no matter how much work
we did. We fixed the first and filed the second upstream.

That's the thing I'd want people to take from this. In a field moving this fast,
the interesting question isn't what your system does when it works. It's what it
does when it doesn't — and whether you'd find out.

TermLink is Apache-2.0-friendly, MCP-native, and runs anywhere you can run a
binary.

#AIAgents #DeveloperTools #Rust #DistributedSystems #OpenSource

---

## OPTIONAL seniority line — recommend AGAINST, decide for yourself

If you want the "we've been around longer" angle, this is the only version I'd
put your name on:

> "We started TermLink on 8 March 2026, a few weeks before herdr's first commit —
> not that it matters much at this age. What matters is that we spent those
> months on delivery guarantees rather than on the terminal."

**Why I'd cut it entirely:**

- The real gap is **19 days** (TermLink 2026-03-08, herdr 2026-03-27). Both
  projects are ~5 months old.
- Both repos are public. Anyone can check both first-commit dates in ten seconds.
- A 19-day seniority claim against a 29,000-star project reads as reaching, and
  the correction — if someone makes it in the comments — becomes the story
  instead of the exit-codes-and-signals argument, which is the one you actually
  win on.

Seniority is your weakest available claim. The technical distinction is your
strongest, it's fully evidenced, and it doesn't depend on anyone else looking bad.

## Accuracy notes before publishing

Every technical claim above traces to the evaluation in
`herdr-evaluation-synthesis-2026-08-15.md`. Two things to confirm are still true
at publication time, since they are herdr-side and it ships every ~3 days:

1. **"cannot return exit codes" / "cannot deliver signals"** — verified against
   their socket API docs on 2026-08-15. Re-check before posting; a project
   releasing this fast could add either.
2. **The #1 GitHub Trending and star figures** — 29,247 stars as of 2026-08-15.

Deliberately NOT claimed, because we cannot substantiate them: any TermLink
adoption or user numbers, any performance comparison, and anything about herdr's
reliability in practice. Note their star count dwarfs ours; the post therefore
never competes on traction, which is the one axis where a reader can check us and
find us wanting.

The tone is deliberately gracious. Punching sideways at a well-liked open-source
project is bad positioning even when you're right, and here we're not
even in conflict — we genuinely concluded they're solving a different problem.
