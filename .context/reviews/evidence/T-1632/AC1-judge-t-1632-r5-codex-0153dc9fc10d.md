# Reviewer evidence — T-1632 AC#1

- dispatch: `judge-t-1632-r5-codex-0153dc9fc10d`
- worker kind: `codex` (vendor `openai`, model `gpt-6-astra`)
- seat: `codex`
- reviewed revision: `6f6fac33a0fa6822b5301aecd5edec1b93518d1e`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `20e00883a809d4d88db527c65b419a9ee2649c267c25fa78cc71919eb06f1be9`
- result.md sha256: `190541432f1d3e4a2684cca206dd7f6a7026b0c5cb889b03da247278b6962385`

## The worker's verdict block

```
1. [AC] Deployed .122 hub reports control plane version 3 and protocol version 1
VERDICT: green
WHY: Checked the recorded measured evidence dated 2026-11-17 for 192.168.10.122:9100. It identifies the post-upgrade hub.capabilities call and records both required values: "protocol_version": 1 and "control_plane_version": 3. The reported hub_version matches the deployed build, 0.12.13+135.gd167da0e2. The hub.version output corroborates both version fields and the unchanged hub identity. This proves the required post-deploy result under the widened criterion’s explicit allowance for recorded measured evidence.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
1. [AC] Deployed .122 hub reports control plane version 3 and protocol version 1
VERDICT: green
WHY: Checked the recorded measured evidence dated 2026-11-17 for 192.168.10.122:9100. It identifies the post-upgrade hub.capabilities call and records both required values: "protocol_version": 1 and "control_plane_version": 3. The reported hub_version matches the deployed build, 0.12.13+135.gd167da0e2. The hub.version output corroborates both version fields and the unchanged hub identity. This proves the required post-deploy result under the widened criterion’s explicit allowance for recorded measured evidence.
GUIDANCE: None.

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
