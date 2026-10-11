//! C-11 Claude adapter: the transcript evidence reader (architecture note 6.4;
//! SI-17, FC-11, FC-13). The hook entry points are B10.
//!
//! Evidence, from the B1 spike (`docs/reports/arc011-ph1-hook-spike.md`): when a
//! PostToolUse hook prints `additionalContext`, Claude Code writes a record
//! `{"type":"attachment","attachment":{"type":"hook_additional_context",
//! "content":[<text>], "toolUseID": ...}}` after the tool's result and before the
//! model's next turn. Only that record type is evidence. The same nonce in an
//! assistant record, a user record, a tool result or the `hook_success` echo of
//! the hook's own stdout proves nothing: the model or a tool could have written
//! it, and the echo exists even when the harness dropped the output.
//!
//! Completeness point (CR-17): for a hook that fired after tool call `T`, the
//! harness writes `T`'s tool_result, then that call's hook attachments, then the
//! next assistant record. An assistant record after `T`'s tool_result with no
//! evidence in between proves the frame was not delivered. Without `T`, absence
//! cannot be proven and the attempt stays open.
//!
//! Unreadable means: the file cannot be opened or read, or a line in it is not a
//! JSON object (a partial final line included). Unreadable is never absence.

use serde_json::Value;
use std::io::Read;
use std::path::Path;

/// Largest transcript the reader will scan.
pub const MAX_TRANSCRIPT: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evidence {
    /// The header was found in a harness-written `hook_additional_context` record.
    Found { tool_use_id: Option<String> },
    /// Not found. `complete` is true when the completeness point for the given
    /// tool call was reached, i.e. absence is proven.
    Absent { complete: bool },
    /// The transcript could not be read; nothing can be concluded.
    Unreadable(String),
}

fn attachment_texts(rec: &Value) -> Option<(Vec<&str>, Option<String>)> {
    if rec.get("type").and_then(Value::as_str) != Some("attachment") {
        return None;
    }
    let a = rec.get("attachment")?;
    if a.get("type").and_then(Value::as_str) != Some("hook_additional_context") {
        return None;
    }
    let texts: Vec<&str> = match a.get("content") {
        Some(Value::Array(v)) => v.iter().filter_map(Value::as_str).collect(),
        Some(Value::String(s)) => vec![s.as_str()],
        _ => vec![],
    };
    let tool = a.get("toolUseID").and_then(Value::as_str).map(str::to_owned);
    Some((texts, tool))
}

fn is_tool_result_for(rec: &Value, id: &str) -> bool {
    if rec.get("type").and_then(Value::as_str) != Some("user") {
        return false;
    }
    rec.get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array)
        .is_some_and(|c| {
            c.iter().any(|x| {
                x.get("type").and_then(Value::as_str) == Some("tool_result")
                    && x.get("tool_use_id").and_then(Value::as_str) == Some(id)
            })
        })
}

/// Look for `header_prefix` at the start of a frame in `transcript`.
/// `tool_use_id` is the tool call after which the hook fired, when known.
pub fn find(transcript: &Path, header_prefix: &str, tool_use_id: Option<&str>) -> Evidence {
    let mut f = match std::fs::File::open(transcript) {
        Ok(f) => f,
        Err(e) => return Evidence::Unreadable(format!("cannot open the transcript: {e}")),
    };
    let mut buf = Vec::new();
    if let Err(e) = (&mut f).take(MAX_TRANSCRIPT + 1).read_to_end(&mut buf) {
        return Evidence::Unreadable(format!("cannot read the transcript: {e}"));
    }
    if buf.len() as u64 > MAX_TRANSCRIPT {
        return Evidence::Unreadable(format!("transcript larger than {MAX_TRANSCRIPT} bytes"));
    }
    let text = match std::str::from_utf8(&buf) {
        Ok(t) => t,
        Err(_) => return Evidence::Unreadable("transcript is not UTF-8".into()),
    };
    let mut seen_result = false;
    let mut complete = false;
    for (n, line) in text.split('\n').enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let rec: Value = match serde_json::from_str(line) {
            Ok(v @ Value::Object(_)) => v,
            _ => return Evidence::Unreadable(format!("line {} is not a JSON record", n + 1)),
        };
        if let Some((texts, tool)) = attachment_texts(&rec)
            && texts.iter().any(|t| t.starts_with(header_prefix))
        {
            return Evidence::Found { tool_use_id: tool };
        }
        if let Some(id) = tool_use_id {
            if is_tool_result_for(&rec, id) {
                seen_result = true;
            } else if seen_result && rec.get("type").and_then(Value::as_str) == Some("assistant") {
                // Keep scanning: a later record may still be the evidence
                // (it would be out of order, but evidence wins over absence).
                complete = true;
            }
        }
    }
    Evidence::Absent { complete }
}
