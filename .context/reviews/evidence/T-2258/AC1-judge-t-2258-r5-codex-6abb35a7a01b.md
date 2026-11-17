# Reviewer evidence — T-2258 AC#1

- dispatch: `judge-t-2258-r5-codex-6abb35a7a01b`
- worker kind: `codex` (vendor `openai`, model `gpt-6-astra`)
- seat: `codex`
- reviewed revision: `ccea0a0717d238c5b7974bfa6a28fac0b87c8341`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `5f473658b026190a9a50f42c4b555ce31ec559ef669fe0d4a836400b05442af5`
- result.md sha256: `31c4443efba0cc399fe4251468dd4515cb7e3df24090aed29eb9fffca2993654`

## The worker's verdict block

```
1. [AC] Fixed binary deployed to .107; live large-topic reads complete within 12 seconds
VERDICT: green
WHY: Under the widened recorded-evidence rule, the supplied measurements satisfy the restated criterion. hub.version identifies deployed build 0.12.13+135.gd167da0e2; the deployment record reports restart with secret, certificate and hub ID unchanged. All recorded framework:pickup reads returned rc=0 within 0.38 seconds, without probe posts there. On the scratch topic seeded with 1500 posts, the immediate read after posting returned rc=0 in 178 ms; four concurrent reads during 20 landing posts returned rc=0 in 112–189 ms. Deletion reported 1521 records, corroborating the stated topic size. Every recorded read was bounded by timeout -s KILL 12. This verdict assesses the supplied recorded evidence; I did not rerun the live operations.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
1. [AC] Fixed binary deployed to `.107`; live large-topic reads complete within 12 seconds
VERDICT: green
WHY: Under the widened recorded-evidence rule, the supplied measurements satisfy the restated criterion. `hub.version` identifies deployed build `0.12.13+135.gd167da0e2`; the deployment record reports restart with secret, certificate and hub ID unchanged. All recorded `framework:pickup` reads returned rc=0 within 0.38 seconds, without probe posts there. On the scratch topic seeded with 1500 posts, the immediate read after posting returned rc=0 in 178 ms; four concurrent reads during 20 landing posts returned rc=0 in 112–189 ms. Deletion reported 1521 records, corroborating the stated topic size. Every recorded read was bounded by `timeout -s KILL 12`. This verdict assesses the supplied recorded evidence; I did not rerun the live operations.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
