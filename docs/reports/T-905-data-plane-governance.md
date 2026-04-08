# T-905: Data Plane Governance Subscriber

## Summary

Added a governance subscriber to the data plane that watches Output frames for configurable regex patterns and emits Governance frames for audit/metrics. The subscriber is non-blocking, opt-in, and processes output post-hoc.

## Changes

### termlink-protocol

- **`data.rs`** — Added `Governance = 0x8` frame type to `FrameType` enum. Updated `from_u8` match arm and tests.
- **`governance.rs`** (new) — `GovernanceEvent` struct with `pattern_name`, `match_text`, `timestamp`, `channel_id`. JSON serialization for frame payloads. 3 tests.
- **`lib.rs`** — Registered `governance` module.

### termlink-session

- **`governance_subscriber.rs`** (new) — Core implementation:
  - `PatternRule`: named regex pattern for matching
  - `GovernanceConfig`: list of pattern rules
  - `GovernanceSubscriber`: receives Output frames via broadcast channel, strips ANSI, matches patterns, emits Governance frames via mpsc channel
  - `strip_ansi_codes()`: local ANSI stripping (same algorithm as handler.rs)
  - 9 tests: strip_ansi (4), pattern match/emit (1), no-match (1), ANSI-before-match (1), multi-pattern (1), sequence increment (1)
- **`data_server.rs`** — Added `run_with_governance()` function that spawns a subscriber alongside the normal data plane server and returns governance frame receiver.
- **`lib.rs`** — Registered `governance_subscriber` module.

### Workspace

- **`Cargo.toml`** — Added `regex = "1"` to workspace dependencies.
- **`crates/termlink-session/Cargo.toml`** — Added `regex = { workspace = true }`.

## Architecture

```
Output broadcast channel
        |
        +---> Data plane clients (existing)
        |
        +---> GovernanceSubscriber (new, opt-in)
                |
                +--> strip ANSI
                +--> match regex patterns
                +--> emit Governance frame via mpsc
```

- **Non-blocking**: subscriber gets a copy via `broadcast::Receiver::resubscribe()`
- **Bounded**: mpsc channel (256 capacity), `try_send` drops events if full
- **Opt-in**: activated via `run_with_governance()`, not attached by default

## Test Results

- termlink-protocol: 92 passed
- termlink-session: 250 passed (including 9 governance-specific)
- Full workspace: compiles clean
