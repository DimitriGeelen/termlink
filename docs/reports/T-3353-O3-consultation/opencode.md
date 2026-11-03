I'm in **plan mode** (read-only), so I cannot create the file yet. Here is the plan, plus the full review content I'll write upon approval.

## Plan
1. `mkdir -p docs/reports/T-3353-O3-consultation` (directory doesn't exist yet; `docs/reports/` does)
2. Write the review below verbatim to `docs/reports/T-3353-O3-consultation/opencode.md`
3. Touch nothing else

## Proposed file content (≈420 words, per the brief's structure)

> ### T-3580 independent review: who may become "main" before signed eligibility profiles exist
>
> **1. Choice: A — with one tightening.** The core invariant is "exactly one session handles peer mail," and only A preserves it unconditionally, with no config prerequisite and no operator action required. B is degenerate in this deployment: since all sessions of a project share one agent id, B's filter passes exactly what A passes — zero added discrimination — while adding a total-outage mode (unnamed or stale address-book entry → no main ever). B is strictly A minus robustness. C swaps a wrong-main failure for a no-main failure; wrong-main under A is bounded (an idle session stops renewing, lease lapses in 15 min, a live session takes over), whereas C's silent queue is unbounded and invisible. Note also that A ⊇ C's safety property on demand: the pin exists and overrides everything, so a cautious operator gets C's behavior by pinning; C can never recover A's liveness. The team's recommendation and the operator's no-main argument are correct.
>
> **2. Strongest argument against A.** It retains the exact failure class that motivated this design: an unattended session holding main and starving mail. The lease only bounds this if the holder goes quiet — a background session that prompts on a timer (cron, watcher) renews forever without any operator looking, and the mitigation collapses back to "the operator notices and pins," the same vigilance assumption that failed for 17 days.
>
> **3. Failure modes.**
> - **A:** auto-prompting background sessions hold the lease indefinitely; first-to-claim races favor automation over the operator's session; safety rests on operator attention.
> - **B:** missing/wrong address-book entry → permanent no-main, delivery stops entirely; with shared agent ids it filters nothing, creating false confidence; a quiet config dependency becomes a single point of failure.
> - **C:** unpinned projects silently queue forever; operators forget pins per project; peer senders time out with no accountable holder; availability equals operator presence.
>
> **4. What the options miss.**
> - Prompt origin matters more than session identity here: treat claim-on-start as provisional and count only human-initiated prompts as renewals (or flag non-human renewals in the audit stream). This kills the timer-driven renewal failure without profiles.
> - Make queue state peer-visible ("mail waiting, main not reading") so senders can escalate instead of waiting silently — valuable under every option.
> - If B or C were ever chosen, a no-main state must page loudly, not queue quietly.
> - Above all: A is a stopgap that recreates the incident class until signed profiles land; ship profiles with priority, and treat the prompt-origin tightening as the bridge.

Say the word (e.g., "proceed") and I'll write it to the file.
