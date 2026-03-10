# termlink-protocol Review

**Control/data plane split is sound.** Control plane uses JSON-RPC 2.0 (text, flexible, extensible) for session management and commands; data plane uses a compact 22-byte binary frame header for high-throughput terminal I/O. This is the right separation — control messages are infrequent and benefit from schema flexibility, while output/input streams need low overhead.

**Wire format (data plane) is well-designed.** Magic bytes ("TL") for sync, big-endian fields, version nibble, bitflags for compression/urgency/binary, 8-byte sequence numbers, channel multiplexing via `channel_id`, 2 reserved bytes for future use. Payload size is bounded (16 MiB). Encode/decode are symmetric with good error reporting.

**Versioning is minimal but adequate for v1.** Version is a 4-bit nibble in the data plane header (max 15 versions) and a `protocol_version: u8` in control plane capabilities. Hard-reject on mismatch (`UnsupportedVersion`) means no negotiation — acceptable now, but version negotiation will be needed before v2.

**Extensibility is reasonable.** Control plane inherits JSON-RPC's natural extensibility (add methods/params freely). Data plane has reserved bytes, spare frame types (8-15 unused), and spare flag bits (4 unused). The `features` vec in `Capabilities` allows capability advertisement.

**Error handling is thorough.** `ProtocolError` covers all parse/decode failures with descriptive messages. Control plane defines 10 domain-specific error codes (-32001 to -32010) alongside standard JSON-RPC codes. `from_bits_truncate` on flags is a pragmatic forward-compat choice.

**Minor concerns:** (1) No checksum/CRC on data frames — relies on transport integrity. (2) `CommonParams.timestamp` is a `String`, not a typed timestamp — risks format inconsistency. (3) `jsonrpc` field is a `String` rather than a validated constant, allowing malformed values.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-069 (event schema v2)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
