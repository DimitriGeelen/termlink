# Reviewer evidence — T-2258 AC#1

- dispatch: `judge-t-2258-r5-codex-785ec783d53c`
- worker kind: `codex` (vendor `openai`, model `gpt-6-astra`)
- seat: `codex`
- reviewed revision: `37c945a39df76fc1f82450555b8e5852542a78c2`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `7554469f9defcd0b7ffd2e3b0e9dcdabb1eeb480f5ba6936f5fc5d71a425edf6`
- result.md sha256: `f9c7cd9ea87bae21ae4777d4dba505c12aac38e071dc67593f2e4c6df380d88f`

## The worker's verdict block

```
1. [AC] Fixed binary deployed to .107; bounded live and large-topic reads
VERDICT: green
WHY: Under the widened evidence rule, the recorded measurements establish the act. hub.version identifies the deployed binary as 0.12.13+135.gd167da0e2 on 192.168.10.107:9100. The large-topic run records three successful framework:pickup reads in 134–166 ms, an immediate scratch-topic read after posting in 178 ms, and four concurrent reads during 20 landing posts in 112–189 ms. All returned rc=0 under the specified 12-second timeout. Cleanup reports 1521 records, consistent with 1500 seeds, one immediate-read probe, and 20 landing posts. Probes were confined to the scratch topic; framework:pickup was read only. This verdict evaluates the supplied recorded evidence, as expressly authorized; it does not claim a fresh live rerun.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
1. [AC] Fixed binary deployed to `.107`; bounded live and large-topic reads
VERDICT: green
WHY: Under the widened evidence rule, the recorded measurements establish the act. `hub.version` identifies the deployed binary as `0.12.13+135.gd167da0e2` on `192.168.10.107:9100`. The large-topic run records three successful `framework:pickup` reads in 134–166 ms, an immediate scratch-topic read after posting in 178 ms, and four concurrent reads during 20 landing posts in 112–189 ms. All returned rc=0 under the specified 12-second timeout. Cleanup reports 1521 records, consistent with 1500 seeds, one immediate-read probe, and 20 landing posts. Probes were confined to the scratch topic; `framework:pickup` was read only. This verdict evaluates the supplied recorded evidence, as expressly authorized; it does not claim a fresh live rerun.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
