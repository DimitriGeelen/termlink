use anyhow::{Context, Result};

use termlink_session::client;

use termlink_protocol::events::{
    file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

use crate::cli::{ProfileAction, RemoteInboxAction};
use crate::config::{hubs_config_path, load_hubs_config, save_hubs_config, HubEntry};
use crate::util::{generate_request_id, truncate, DEFAULT_CHUNK_SIZE};

use super::ListDisplayOpts;

/// T-1459: Threshold (seconds) below which a legacy-primitive call counts
/// as ACTIVE (live caller still polling). Beyond this, the call is "decay
/// residue" — historical data within the audit window. Shared between the
/// per-hub display tag and the top-level `compute_cut_readiness_verdict`
/// so they cannot drift.
const ACTIVE_TRAFFIC_THRESHOLD_SECS: u64 = 300;

/// T-1461: Pure aggregator for fleet-wide top_callers. Sums counts across
/// every hub's top_callers list (already each sorted desc, but we re-sort
/// the merged result). Returns Vec<(id, count)> sorted desc by count, ties
/// broken by id (deterministic).
///
/// Empty input → empty output. The operator gets a single clear "this
/// caller dominates fleet-wide residue" line instead of N repeated per-hub
/// lines.
fn aggregate_fleet_top_callers(
    per_hub: &std::collections::BTreeMap<String, Vec<(String, u64)>>,
) -> Vec<(String, u64)> {
    let mut merged: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for callers in per_hub.values() {
        for (id, count) in callers {
            *merged.entry(id.clone()).or_insert(0) += count;
        }
    }
    let mut out: Vec<(String, u64)> = merged.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// T-1467: Derive a per-hub `top_callers` list from a hub's `by_method` block
/// when the hub doesn't ship the native `top_callers` field (i.e. pre-T-1460
/// hubs, which is the entire 0.9.0 fleet at the time of writing).
///
/// `by_method` shape (as produced by `hub.legacy_usage`):
/// ```json
/// { "<method>": { "callers": [ { "from": "<id>", "count": N }, ... ] } }
/// ```
///
/// Sums counts per `from` across all methods, returns a vec sorted by count
/// descending (ties broken alphabetically). Returns an empty vec if `by_method`
/// is not an object, has no recognisable callers entries, or every entry is
/// zero. Schema-additive: post-T-1460 hubs ship `top_callers` directly and the
/// caller in `cmd_fleet_doctor` only invokes this fallback when that field is
/// absent or empty.
fn derive_top_callers_from_by_method(by_method: &serde_json::Value) -> Vec<(String, u64)> {
    let Some(obj) = by_method.as_object() else { return Vec::new() };
    let mut merged: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for method_block in obj.values() {
        let Some(callers) = method_block.get("callers").and_then(|v| v.as_array()) else { continue };
        for c in callers {
            let Some(id) = c.get("from").and_then(|v| v.as_str()) else { continue };
            let Some(count) = c.get("count").and_then(|v| v.as_u64()) else { continue };
            if count == 0 { continue }
            *merged.entry(id.to_string()).or_insert(0) += count;
        }
    }
    let mut out: Vec<(String, u64)> = merged.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// T-1459: Pure verdict aggregator for `fleet doctor --legacy-usage`.
///
/// Inputs are the per-hub buckets already classified by the calling code:
///   * `hubs_with_traffic`: (name, total_legacy_count, last_call_ts_ms)
///   * `hubs_unsupported`:  hubs that returned audit_unsupported (pre-T-1432)
///   * `hubs_no_audit`:     hubs with audit_present=false (fresh runtime_dir)
///   * `hubs_clean`:        hubs with audit_present=true AND total_legacy=0
///   * `now_ms`:            current Unix time in milliseconds
///
/// Returns one of: `CUT-READY`, `CUT-READY-DECAYING`, `WAIT`, `UNCERTAIN`.
/// See the verdict-semantics block at the call site for the full rationale.
fn compute_cut_readiness_verdict(
    hubs_with_traffic: &[(String, u64, u128)],
    hubs_unsupported: &[String],
    hubs_no_audit: &[String],
    hubs_clean: &[String],
    now_ms: u128,
) -> &'static str {
    let any_active = hubs_with_traffic.iter().any(|(_, _, last_ts)| {
        *last_ts > 0
            && now_ms > *last_ts
            && (now_ms - *last_ts) / 1000 < ACTIVE_TRAFFIC_THRESHOLD_SECS as u128
    });
    let any_traffic = !hubs_with_traffic.is_empty();
    let any_uncertain = !hubs_unsupported.is_empty() || !hubs_no_audit.is_empty();
    let any_clean = !hubs_clean.is_empty();

    if any_active {
        "WAIT"
    } else if any_traffic {
        // Residue exists but no live callers. If some hubs are unmeasurable
        // (pre-T-1432 or audit_present=false) we cannot rule out hidden
        // active traffic on those, so degrade to UNCERTAIN.
        if any_uncertain {
            "UNCERTAIN"
        } else {
            "CUT-READY-DECAYING"
        }
    } else if any_uncertain {
        "UNCERTAIN"
    } else if any_clean {
        "CUT-READY"
    } else {
        // No reachable hubs at all — caller has nothing to act on.
        "UNCERTAIN"
    }
}

/// T-1465: Map a cut-readiness verdict string to a process exit code so
/// shell scripts and CI pipelines can gate on it without parsing JSON.
/// Mapping picked to keep CUT-READY-DECAYING in the success bucket (it is
/// safe to cut from there) while distinguishing WAIT (live caller, retry)
/// from UNCERTAIN (operator action — upgrade or wait for traffic).
/// Anything outside the four documented verdicts is treated as UNCERTAIN.
pub(crate) fn verdict_to_exit_code(verdict: &str) -> i32 {
    match verdict {
        "CUT-READY" | "CUT-READY-DECAYING" => 0,
        "WAIT" => 10,
        _ => 11, // UNCERTAIN and any unknown future verdict
    }
}

/// T-1462: Diff between two `legacy_summary` snapshots (prior vs current).
/// All fields are signed deltas (current - prior) except the elapsed time and
/// rate, which are computed only when both snapshots embedded `_snapshot_ts_ms`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LegacyDiff {
    pub total_fleet_delta: i64,
    /// Per-hub deltas. `None` for `prior_count` means the hub appeared after
    /// the prior snapshot; `None` for `current_count` means it vanished.
    pub per_hub: Vec<HubDelta>,
    /// Per-caller deltas computed from `top_callers_fleet`. Same vanish/appear
    /// semantics as per_hub.
    pub per_caller: Vec<CallerDelta>,
    /// Elapsed milliseconds between snapshots, when both timestamps were
    /// embedded. None means the prior snapshot predated T-1462.
    pub elapsed_ms: Option<u64>,
    /// Average legacy calls per minute over the elapsed interval. None when
    /// elapsed_ms is None or zero.
    pub rate_per_min: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HubDelta {
    pub hub: String,
    pub prior_count: Option<u64>,
    pub current_count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CallerDelta {
    pub id: String,
    pub prior_count: Option<u64>,
    pub current_count: Option<u64>,
}

impl HubDelta {
    pub fn delta(&self) -> i64 {
        self.current_count.unwrap_or(0) as i64 - self.prior_count.unwrap_or(0) as i64
    }
}

impl CallerDelta {
    pub fn delta(&self) -> i64 {
        self.current_count.unwrap_or(0) as i64 - self.prior_count.unwrap_or(0) as i64
    }
}

/// T-1462: Compute diff between two legacy_summary objects (the JSON values
/// produced by `cmd_fleet_doctor` under the `legacy_summary` key).
///
/// `prior_ts_ms` and `current_ts_ms` are the top-level `_snapshot_ts_ms`
/// values; pre-T-1462 snapshots will not have one, so prior_ts_ms is Option.
/// Pure function — no I/O, no clock — for testability.
pub(crate) fn compute_legacy_diff(
    prior: &serde_json::Value,
    current: &serde_json::Value,
    prior_ts_ms: Option<u64>,
    current_ts_ms: u64,
) -> LegacyDiff {
    let prior_total = prior
        .get("total_legacy_fleet")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let current_total = current
        .get("total_legacy_fleet")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_fleet_delta = current_total as i64 - prior_total as i64;

    // Per-hub: union of (clean ∪ with_traffic) sets in both snapshots.
    let prior_hubs = collect_hub_counts(prior);
    let current_hubs = collect_hub_counts(current);
    let mut all_names: std::collections::BTreeSet<String> =
        prior_hubs.keys().cloned().collect();
    all_names.extend(current_hubs.keys().cloned());
    let mut per_hub: Vec<HubDelta> = all_names
        .into_iter()
        .map(|name| HubDelta {
            prior_count: prior_hubs.get(&name).copied(),
            current_count: current_hubs.get(&name).copied(),
            hub: name,
        })
        .collect();
    // Sort by absolute delta descending, then by name for determinism.
    per_hub.sort_by(|a, b| b.delta().abs().cmp(&a.delta().abs()).then_with(|| a.hub.cmp(&b.hub)));

    // Per-caller: from top_callers_fleet array on each side.
    let prior_callers = collect_caller_counts(prior);
    let current_callers = collect_caller_counts(current);
    let mut all_ids: std::collections::BTreeSet<String> =
        prior_callers.keys().cloned().collect();
    all_ids.extend(current_callers.keys().cloned());
    let mut per_caller: Vec<CallerDelta> = all_ids
        .into_iter()
        .map(|id| CallerDelta {
            prior_count: prior_callers.get(&id).copied(),
            current_count: current_callers.get(&id).copied(),
            id,
        })
        .collect();
    per_caller.sort_by(|a, b| b.delta().abs().cmp(&a.delta().abs()).then_with(|| a.id.cmp(&b.id)));

    let elapsed_ms = prior_ts_ms.and_then(|p| {
        if current_ts_ms > p {
            Some(current_ts_ms - p)
        } else {
            None
        }
    });
    let rate_per_min = elapsed_ms.and_then(|e| {
        if e == 0 {
            None
        } else {
            // Rate uses the absolute delta of legacy traffic (calls added in
            // the interval); negative deltas (decay) yield a negative rate
            // which is also useful — operators read it as "calls/min disappearing".
            Some((total_fleet_delta as f64) / (e as f64 / 60_000.0))
        }
    });

    LegacyDiff {
        total_fleet_delta,
        per_hub,
        per_caller,
        elapsed_ms,
        rate_per_min,
    }
}

fn collect_hub_counts(
    legacy_summary: &serde_json::Value,
) -> std::collections::BTreeMap<String, u64> {
    let mut out = std::collections::BTreeMap::new();
    if let Some(arr) = legacy_summary.get("hubs_clean").and_then(|v| v.as_array()) {
        for n in arr {
            if let Some(s) = n.as_str() {
                out.insert(s.to_string(), 0);
            }
        }
    }
    if let Some(arr) = legacy_summary.get("hubs_with_traffic").and_then(|v| v.as_array()) {
        for h in arr {
            if let (Some(name), Some(count)) = (
                h.get("hub").and_then(|v| v.as_str()),
                h.get("count").and_then(|v| v.as_u64()),
            ) {
                out.insert(name.to_string(), count);
            }
        }
    }
    out
}

fn collect_caller_counts(
    legacy_summary: &serde_json::Value,
) -> std::collections::BTreeMap<String, u64> {
    let mut out = std::collections::BTreeMap::new();
    if let Some(arr) = legacy_summary.get("top_callers_fleet").and_then(|v| v.as_array()) {
        for c in arr {
            if let (Some(id), Some(count)) = (
                c.get("id").and_then(|v| v.as_str()),
                c.get("count").and_then(|v| v.as_u64()),
            ) {
                out.insert(id.to_string(), count);
            }
        }
    }
    out
}

/// T-1462: Render diff to JSON for `--json` output. Mirrors LegacyDiff fields
/// in a stable, schema-additive shape.
pub(crate) fn legacy_diff_to_json(d: &LegacyDiff) -> serde_json::Value {
    serde_json::json!({
        "total_fleet_delta": d.total_fleet_delta,
        "elapsed_ms": d.elapsed_ms,
        "rate_per_min": d.rate_per_min,
        "per_hub": d.per_hub.iter().map(|h| serde_json::json!({
            "hub": h.hub,
            "prior_count": h.prior_count,
            "current_count": h.current_count,
            "delta": h.delta(),
        })).collect::<Vec<_>>(),
        "per_caller": d.per_caller.iter().map(|c| serde_json::json!({
            "id": c.id,
            "prior_count": c.prior_count,
            "current_count": c.current_count,
            "delta": c.delta(),
        })).collect::<Vec<_>>(),
    })
}

/// T-1462: Print the human-readable diff block under cut-readiness output.
pub(crate) fn print_legacy_diff_block(d: &LegacyDiff, snapshot_label: &str) {
    eprintln!();
    eprintln!("=== T-1166 cut-readiness DIFF vs {snapshot_label} ===");
    let arrow = if d.total_fleet_delta > 0 {
        "↑"
    } else if d.total_fleet_delta < 0 {
        "↓ (decay)"
    } else {
        "→ (no change)"
    };
    let elapsed_str = match d.elapsed_ms {
        Some(ms) => {
            let secs = ms / 1000;
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3600 {
                format!("{}m{}s", secs / 60, secs % 60)
            } else if secs < 86400 {
                format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
            } else {
                format!("{}d{}h", secs / 86400, (secs % 86400) / 3600)
            }
        }
        None => "(prior snapshot has no _snapshot_ts_ms; pre-T-1462)".to_string(),
    };
    eprintln!(
        "  fleet total_legacy: {} {arrow}  (elapsed: {})",
        d.total_fleet_delta, elapsed_str
    );
    if let Some(rate) = d.rate_per_min {
        let tag = if rate > 0.0 {
            "growing"
        } else if rate < 0.0 {
            "decaying"
        } else {
            "flat"
        };
        eprintln!("  rate: {:+.2} calls/min ({tag})", rate);
    }
    let mut hub_lines: Vec<String> = Vec::new();
    for h in &d.per_hub {
        // Suppress zero-information rows: delta=0 covers both stable hubs and
        // clean→absent / absent→clean transitions (information-free).
        if h.delta() == 0 {
            continue;
        }
        let line = match (h.prior_count, h.current_count) {
            (None, Some(c)) => format!("    {}: NEW → {} (+{})", h.hub, c, c),
            (Some(p), None) => format!("    {}: {} → VANISHED (-{})", h.hub, p, p),
            (Some(p), Some(c)) => format!("    {}: {} → {} ({:+})", h.hub, p, c, h.delta()),
            (None, None) => continue,
        };
        hub_lines.push(line);
    }
    if !hub_lines.is_empty() {
        eprintln!("  Per-hub change:");
        for l in hub_lines {
            eprintln!("{l}");
        }
    }
    let mut caller_lines: Vec<String> = Vec::new();
    for c in d.per_caller.iter().take(5) {
        if c.delta() == 0 {
            continue;
        }
        let line = match (c.prior_count, c.current_count) {
            (None, Some(n)) => format!("    {}: NEW → {} (+{})", c.id, n, n),
            (Some(p), None) => format!("    {}: {} → VANISHED (-{})", c.id, p, p),
            (Some(p), Some(n)) => format!("    {}: {} → {} ({:+})", c.id, p, n, c.delta()),
            (None, None) => continue,
        };
        caller_lines.push(line);
    }
    if !caller_lines.is_empty() {
        eprintln!("  Top-caller change:");
        for l in caller_lines {
            eprintln!("{l}");
        }
    }
}

/// T-1468: a single point in the legacy-traffic time-series — one snapshot's
/// fleet total + the signed delta from the prior point in the series.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TrendPoint {
    pub label: String,
    pub ts_ms: Option<u64>,
    pub total: u64,
    pub delta_from_prior: Option<i64>,
}

/// T-1468: aggregated trend trajectory across N snapshots. `Trajectory` is the
/// caller-friendly verdict on the series:
/// - `Decreasing` — net change is negative (residue is clearing)
/// - `Increasing` — net change is positive (operator should investigate)
/// - `Flat`       — series totals are constant (single point or stable plateau)
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Trajectory {
    Decreasing,
    Increasing,
    Flat,
}

impl Trajectory {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Trajectory::Decreasing => "decreasing",
            Trajectory::Increasing => "increasing",
            Trajectory::Flat => "flat",
        }
    }
}

/// T-1468: compute a decay trend across N legacy_summary snapshots.
///
/// Input: ordered slice of (label, &snapshot_json) pairs — caller is responsible
/// for sorting (the cron convention is filename = `YYYY-MM-DD.json`, lex sort
/// = chronological). The snapshot_json must be the FULL fleet doctor JSON
/// document (top-level `legacy_summary` and `_snapshot_ts_ms` fields read
/// from it). Snapshots without `legacy_summary.total_legacy_fleet` are skipped
/// silently — pre-T-1459 outputs.
///
/// Returns a vec of TrendPoint with deltas computed pairwise. The first point
/// has `delta_from_prior=None`. Trajectory is derived from net change between
/// first and last total: positive=Increasing, negative=Decreasing, zero=Flat.
pub(crate) fn compute_legacy_trend(
    snapshots: &[(String, &serde_json::Value)],
) -> (Vec<TrendPoint>, Trajectory) {
    let mut points: Vec<TrendPoint> = Vec::new();
    for (label, doc) in snapshots {
        let Some(total) = doc
            .get("legacy_summary")
            .and_then(|ls| ls.get("total_legacy_fleet"))
            .and_then(|v| v.as_u64())
        else {
            continue;
        };
        let ts_ms = doc.get("_snapshot_ts_ms").and_then(|v| v.as_u64());
        points.push(TrendPoint {
            label: label.clone(),
            ts_ms,
            total,
            delta_from_prior: None,
        });
    }
    // Compute deltas pairwise.
    for i in 1..points.len() {
        let prev = points[i - 1].total as i64;
        let cur = points[i].total as i64;
        points[i].delta_from_prior = Some(cur - prev);
    }
    let trajectory = match (points.first(), points.last()) {
        (Some(first), Some(last)) if points.len() >= 2 => {
            let net = last.total as i64 - first.total as i64;
            if net < 0 {
                Trajectory::Decreasing
            } else if net > 0 {
                Trajectory::Increasing
            } else {
                Trajectory::Flat
            }
        }
        _ => Trajectory::Flat,
    };
    (points, trajectory)
}

/// T-1468: render a Unicode block sparkline (▁▂▃▄▅▆▇█) normalized to the max
/// in the series. Returns empty string when input is empty or all-zero
/// (zero-bucket sparklines are visually meaningless). Single-value input
/// returns the lowest non-empty bucket so the operator sees a tick.
pub(crate) fn render_sparkline(values: &[u64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = *values.iter().max().unwrap();
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    for &v in values {
        // Map [0, max] → [0, BLOCKS.len()-1]. Use BLOCKS.len()-1 (= 7) as the
        // top index so the maximum value always renders as the tallest block,
        // and zero renders as the shortest. Saturating cast handles edge case
        // where max overflows the index range (impossible here, defensive).
        let idx = ((v as u128 * (BLOCKS.len() as u128 - 1)) / max as u128) as usize;
        out.push(BLOCKS[idx.min(BLOCKS.len() - 1)]);
    }
    out
}

/// T-1470: forecast for when total_legacy_fleet hits zero, derived from a
/// least-squares linear fit on (ts_ms, total) trend points.
///
/// `slope_per_day` is in calls/day (negative for decay). `target_ms` is the
/// forecast unix-ms when the fitted line crosses zero. `days_from_now` is
/// `target_ms` relative to `now_ms` in days (can be fractional, never
/// negative — the helper returns `None` when the line crosses zero in the
/// past).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EtaForecast {
    pub slope_per_day: f64,
    pub days_from_now: f64,
    pub target_ms: u64,
}

/// T-1470: compute ETA-to-zero from a series of trend points.
///
/// Returns `None` (forecast not applicable) when ANY of:
/// - fewer than 2 points carry a `ts_ms`
/// - all points have the same total (zero slope, division by zero)
/// - slope is non-negative (flat or increasing — no zero crossing in the future)
/// - the current (last) total is already 0
///
/// Otherwise: ordinary least squares on the timestamped points (the synthesized
/// `(current)` point's missing `ts_ms` is filled with `now_ms` so today's
/// observation always anchors the line). The line `y = m*x + b` is solved for
/// `y=0`: `target_ms = -b / m`. `days_from_now` is the gap from `now_ms` in days.
pub(crate) fn compute_eta_to_zero(points: &[TrendPoint], now_ms: u64) -> Option<EtaForecast> {
    // Build (x, y) pairs as f64; supply `now_ms` for any point whose ts_ms is
    // missing (the trailing "(current)" point produced by cmd_fleet_doctor).
    let xys: Vec<(f64, f64)> = points
        .iter()
        .map(|p| {
            let x = p.ts_ms.unwrap_or(now_ms) as f64;
            let y = p.total as f64;
            (x, y)
        })
        .collect();
    if xys.len() < 2 {
        return None;
    }
    let n = xys.len() as f64;
    let mean_x: f64 = xys.iter().map(|(x, _)| *x).sum::<f64>() / n;
    let mean_y: f64 = xys.iter().map(|(_, y)| *y).sum::<f64>() / n;
    let mut num = 0.0_f64; // Σ (xi - x̄)(yi - ȳ)
    let mut den = 0.0_f64; // Σ (xi - x̄)²
    for (x, y) in &xys {
        let dx = x - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    if den == 0.0 {
        // All x values equal — degenerate, no fit possible.
        return None;
    }
    let slope = num / den; // calls per ms
    // Reject non-decay slopes — no zero crossing in the future.
    if slope >= 0.0 {
        return None;
    }
    let intercept = mean_y - slope * mean_x;
    let target_ms_f = -intercept / slope;
    if !target_ms_f.is_finite() || target_ms_f <= now_ms as f64 {
        // Already crossed zero in the past — likely caused by a noisy fit
        // when current total is already low; not actionable.
        return None;
    }
    let days_from_now = (target_ms_f - now_ms as f64) / 86_400_000.0;
    let slope_per_day = slope * 86_400_000.0; // ms → day scale
    let last_total = points.last().map(|p| p.total).unwrap_or(0);
    if last_total == 0 {
        return None;
    }
    Some(EtaForecast {
        slope_per_day,
        days_from_now,
        target_ms: target_ms_f as u64,
    })
}

/// T-1468 + T-1470: render trend JSON for `--json` output. Includes the optional
/// `eta_zero` block when the linear fit produces an actionable forecast.
pub(crate) fn legacy_trend_to_json(
    points: &[TrendPoint],
    trajectory: &Trajectory,
    eta: Option<&EtaForecast>,
) -> serde_json::Value {
    let mut out = serde_json::json!({
        "trajectory": trajectory.label(),
        "points": points.iter().map(|p| serde_json::json!({
            "snapshot": p.label,
            "ts_ms": p.ts_ms,
            "total_legacy_fleet": p.total,
            "delta_from_prior": p.delta_from_prior,
        })).collect::<Vec<_>>(),
    });
    if let Some(e) = eta {
        out["eta_zero"] = serde_json::json!({
            "target_ms": e.target_ms,
            "days_from_now": e.days_from_now,
            "slope_per_day": e.slope_per_day,
        });
    } else {
        out["eta_zero"] = serde_json::Value::Null;
    }
    out
}

/// T-1468 + T-1470: human-readable trend block printed under cut-readiness.
pub(crate) fn print_legacy_trend_block(
    points: &[TrendPoint],
    trajectory: &Trajectory,
    eta: Option<&EtaForecast>,
) {
    if points.is_empty() {
        eprintln!();
        eprintln!("=== T-1166 cut-readiness TREND ===");
        eprintln!("  no parseable legacy_summary snapshots found in --trend dir");
        return;
    }
    eprintln!();
    eprintln!("=== T-1166 cut-readiness TREND (last {} snapshot{}) ===",
        points.len(),
        if points.len() == 1 { "" } else { "s" },
    );
    for p in points {
        let delta_s = match p.delta_from_prior {
            Some(d) if d > 0 => format!(" ({:+})", d),
            Some(d) if d < 0 => format!(" ({:+})", d),
            Some(_) => "  (—)".to_string(),
            None => String::new(),
        };
        eprintln!("  {:>20}  total={:>8}{}", p.label, p.total, delta_s);
    }
    let totals: Vec<u64> = points.iter().map(|p| p.total).collect();
    let spark = render_sparkline(&totals);
    if !spark.is_empty() {
        eprintln!("  sparkline: {spark}");
    }
    eprintln!("  trajectory: {}", trajectory.label());
    // T-1470: ETA line when the fit produces an actionable forecast. Format
    // the date in ISO-8601 (YYYY-MM-DD) so it sorts and parses naturally.
    if let Some(e) = eta {
        let secs = (e.target_ms / 1000) as i64;
        // Convert the unix timestamp to ISO date via NaiveDateTime arithmetic.
        // Avoid pulling in chrono just for this — manual conversion using the
        // standard library suffices for a YYYY-MM-DD render.
        let date_iso = unix_secs_to_iso_date(secs);
        eprintln!(
            "  ETA to zero: {} ({:.1} days at {:.0}/day)",
            date_iso, e.days_from_now, e.slope_per_day
        );
    }
}

/// T-1470: convert a unix epoch timestamp (seconds) to a YYYY-MM-DD string
/// using Howard Hinnant's civil-from-days algorithm. Avoids a chrono
/// dependency for what amounts to one date format. Years before 1970 not
/// supported (returns "1970-01-01" as a degenerate fallback).
pub(crate) fn unix_secs_to_iso_date(secs: i64) -> String {
    if secs <= 0 {
        return "1970-01-01".to_string();
    }
    let days = (secs / 86_400) + 719_468; // shift to 0000-03-01 epoch
    let era = if days >= 0 { days / 146_097 } else { (days - 146_096) / 146_097 };
    let doe = (days - era * 146_097) as u64; // [0, 146097)
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 400)
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp.wrapping_sub(9) }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", year, m, d)
}

/// Options for remote inject command.
pub(crate) struct RemoteInjectOpts<'a> {
    pub session: &'a str,
    pub text: &'a str,
    pub enter: bool,
    pub key: Option<&'a str>,
    pub delay_ms: u64,
    pub json: bool,
    pub timeout_secs: u64,
}

/// Connection parameters for a remote hub.
pub(crate) struct RemoteConn<'a> {
    pub hub: &'a str,
    pub secret_file: Option<&'a str>,
    pub secret_hex: Option<&'a str>,
    pub scope: &'a str,
}

/// Connect to a remote hub via TOFU TLS and authenticate.
/// Returns an authenticated client ready for RPC calls.
pub(crate) async fn connect_remote_hub(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
) -> Result<client::Client> {
    use termlink_session::auth::{self, PermissionScope};

    // --- Parse hub address ---
    let parts: Vec<&str> = hub.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid hub address '{}'. Expected format: host:port", hub);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("Invalid port in '{}'", hub))?;

    // --- Read secret ---
    let hex = if let Some(path) = secret_file {
        std::fs::read_to_string(path)
            .context(format!("Secret file not found: {}", path))?
            .trim()
            .to_string()
    } else if let Some(h) = secret_hex {
        h.to_string()
    } else {
        anyhow::bail!("Either --secret-file or --secret is required");
    };

    // --- Parse hex to bytes ---
    if hex.len() != 64 {
        anyhow::bail!("Secret must be 64 hex characters (32 bytes), got {} characters", hex.len());
    }
    let secret_bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Secret contains invalid hex characters")?;
    let secret: auth::TokenSecret = secret_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Secret must be exactly 32 bytes"))?;

    // --- Parse scope ---
    let perm_scope = match scope {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => anyhow::bail!("Invalid scope '{}'. Use: observe, interact, control, execute", scope),
    };

    // --- Generate auth token ---
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    // --- Connect via TOFU TLS (T-1677: bounded to 10s — unreachable hubs
    // would otherwise hang for the OS TCP retry budget, 30–60s.) ---
    let addr = termlink_protocol::TransportAddr::Tcp { host, port };
    let mut rpc_client = client::Client::connect_addr_with_timeout(
        &addr,
        std::time::Duration::from_secs(10),
    )
    .await
    .context(format!("Cannot connect to {} — is the hub running?", hub))?;

    // --- Authenticate ---
    match rpc_client.call("hub.auth", serde_json::json!("auth"), serde_json::json!({"token": token.raw})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Authentication failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Authentication error: {}", e);
        }
    }

    Ok(rpc_client)
}

/// Interactive remote session picker — connects to hub, lists sessions, prompts user.
/// Returns the selected session name/ID.
pub(crate) async fn pick_remote_session(
    conn: &RemoteConn<'_>,
) -> Result<String> {
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("No session specified and stdin is not a terminal (cannot prompt)");
    }

    let mut rpc_client = connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await?;

    let resp = rpc_client
        .call("session.discover", serde_json::json!("discover"), serde_json::json!({}))
        .await;

    let sessions = match resp {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            r.result["sessions"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Discover failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Discover error: {}", e);
        }
    };

    if sessions.is_empty() {
        anyhow::bail!("No active sessions on {}.", conn.hub);
    }

    if sessions.len() == 1 {
        let name = sessions[0]["display_name"].as_str().unwrap_or("?");
        let id = sessions[0]["id"].as_str().unwrap_or("?");
        eprintln!("Auto-selecting: {} ({})", name, id);
        return Ok(name.to_string());
    }

    eprintln!("Sessions on {}:", conn.hub);
    eprintln!(
        "  {:<4} {:<20} {:<12} {:<10} TAGS",
        "#", "NAME", "STATE", "PID"
    );
    eprintln!("  {}", "-".repeat(60));
    for (i, s) in sessions.iter().enumerate() {
        let name = s["display_name"].as_str().unwrap_or("?");
        let state = s["state"].as_str().unwrap_or("?");
        let pid = s["pid"].as_u64().unwrap_or(0);
        let tags = s["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        eprintln!(
            "  {:<4} {:<20} {:<12} {:<10} {}",
            i + 1,
            truncate(name, 19),
            state,
            pid,
            tags
        );
    }
    eprintln!();
    eprint!("Select session [1-{}]: ", sessions.len());

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input: {}", e))?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection: '{}'", input.trim()))?;

    if choice < 1 || choice > sessions.len() {
        anyhow::bail!(
            "Selection out of range: {} (expected 1-{})",
            choice,
            sessions.len()
        );
    }

    let selected = &sessions[choice - 1];
    let name = selected["display_name"].as_str().unwrap_or("?");
    let id = selected["id"].as_str().unwrap_or("?");
    eprintln!("→ {} ({})", name, id);
    Ok(name.to_string())
}

/// Resolve a remote session target: if provided, return it; if None, prompt interactively.
pub(crate) async fn resolve_remote_target(
    session: Option<String>,
    conn: &RemoteConn<'_>,
) -> Result<String> {
    if let Some(s) = session {
        return Ok(s);
    }
    pick_remote_session(conn).await
}

pub(crate) fn cmd_remote_profile(action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Add { name, address, secret_file, secret, scope, bootstrap_from, json } => {
            if !address.contains(':') {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": "Address must be in host:port format (e.g., 192.168.10.107:9100)"}));
                }
                anyhow::bail!("Address must be in host:port format (e.g., 192.168.10.107:9100)");
            }
            // T-1291: validate scheme up-front so a typo in `profile add
            // --bootstrap-from foo:bar` fails loud now, not at heal time.
            if let Some(b) = &bootstrap_from
                && !(b.starts_with("file:") || b.starts_with("ssh:"))
            {
                let msg = format!(
                    "--bootstrap-from must start with 'file:' or 'ssh:' (got: {b}). \
                     Examples: file:/etc/termlink/hub.secret  |  ssh:192.168.10.122"
                );
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
                }
                anyhow::bail!(msg);
            }
            let mut config = load_hubs_config();
            let is_update = config.hubs.contains_key(&name);
            // T-1651: capture before move so we can emit the heal-readiness tip below.
            let bootstrap_omitted = bootstrap_from.is_none();
            // Derive a host suggestion for the tip from the address.
            let host_for_tip = address.split(':').next().unwrap_or(&address).to_string();
            config.hubs.insert(name.clone(), HubEntry {
                address: address.clone(),
                secret_file,
                secret,
                scope,
                bootstrap_from,
            });
            save_hubs_config(&config)?;
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "action": if is_update { "updated" } else { "added" },
                    "name": name,
                    "address": address,
                    "config": hubs_config_path().display().to_string(),
                }));
            } else {
                if is_update {
                    println!("Updated profile '{}' → {}", name, address);
                } else {
                    println!("Added profile '{}' → {}", name, address);
                }
                println!("  Config: {}", hubs_config_path().display());
                // T-1651: heal-readiness add-time nudge — catch missing bootstrap_from
                // at the moment the profile is introduced, not later via list-review or
                // at incident time. Suppressed in --json (machine consumers don't need it).
                if bootstrap_omitted {
                    println!("  Tip: no `bootstrap_from` declared — add `--bootstrap-from ssh:{}` to enable one-flag heal (T-1291)", host_for_tip);
                }
                // T-1653 (PL-159 mirror of T-1652): add-time perms warning so a
                // world-readable secret_file is caught at configuration moment
                // rather than waiting for the next fleet status / fleet doctor.
                if let Some(secret_path_raw) = config.hubs.get(&name).and_then(|e| e.secret_file.as_deref())
                    && let Some(warning) = secret_file_perms_warning(&expand_secret_file_path(secret_path_raw))
                {
                    println!("  Warning: {}", warning);
                }
            }
            Ok(())
        }
        ProfileAction::List { json, no_header } => {
            let config = load_hubs_config();
            if json {
                let profiles: Vec<serde_json::Value> = {
                    let mut names: Vec<_> = config.hubs.keys().collect();
                    names.sort();
                    names.iter().map(|name| {
                        let entry = &config.hubs[*name];
                        // T-1653 (PL-159 mirror of T-1652): inspection-time perms surfacing.
                        let secret_perms_warning = entry
                            .secret_file
                            .as_deref()
                            .and_then(|p| secret_file_perms_warning(&expand_secret_file_path(p)));
                        serde_json::json!({
                            "name": name,
                            "address": entry.address,
                            "scope": entry.scope,
                            "secret_type": if entry.secret_file.is_some() { "file" }
                                else if entry.secret.is_some() { "inline" }
                                else { "none" },
                            // T-1650: surface declarative heal anchor for parity with T-1291.
                            "bootstrap_from": entry.bootstrap_from,
                            // T-1653: null when perms are 600 or no secret_file — keeps
                            // machine consumers' field set stable; presence is a flag.
                            "secret_perms_warning": secret_perms_warning,
                        })
                    }).collect()
                };
                println!("{}", serde_json::json!({"ok": true, "profiles": profiles}));
                return Ok(());
            }
            if config.hubs.is_empty() {
                println!("No hub profiles configured.");
                println!("  Add one: termlink remote profile add <name> <address> --secret-file <path>");
                return Ok(());
            }
            if !no_header {
                // T-1650: HEAL column added — `auto` when bootstrap_from declared, `-` otherwise.
                println!("{:<12} {:<28} {:<10} {:<10} HEAL", "NAME", "ADDRESS", "SCOPE", "SECRET");
                println!("{}", "-".repeat(72));
            }
            let mut names: Vec<_> = config.hubs.keys().collect();
            names.sort();
            for name in names {
                let entry = &config.hubs[name];
                let scope = entry.scope.as_deref().unwrap_or("-");
                let secret_info = if entry.secret_file.is_some() {
                    "file"
                } else if entry.secret.is_some() {
                    "inline"
                } else {
                    "none"
                };
                let heal = if entry.bootstrap_from.is_some() { "auto" } else { "-" };
                println!("{:<12} {:<28} {:<10} {:<10} {}", name, entry.address, scope, secret_info, heal);
                // T-1653 (PL-159 mirror of T-1652): inspection-time perms warning,
                // rendered as an indented row under the affected profile so the
                // operator sees it without leaving the list view.
                if let Some(secret_path_raw) = entry.secret_file.as_deref()
                    && let Some(warning) = secret_file_perms_warning(&expand_secret_file_path(secret_path_raw))
                {
                    println!("  Warning: {}", warning);
                }
            }
            if !no_header {
                println!();
                println!("{} profile(s) in {}", config.hubs.len(), hubs_config_path().display());
                // T-1650: heal-readiness summary — count profiles lacking bootstrap_from
                // and recommend declaration (proactive ergonomic, mirrors T-1648/T-1649
                // hint emission at idle time so operators see the gap before incident time).
                if let Some(msg) = heal_readiness_footer(&config.hubs) {
                    println!("  {}", msg);
                }
            }
            Ok(())
        }
        ProfileAction::Remove { name, json } => {
            let mut config = load_hubs_config();
            if config.hubs.remove(&name).is_some() {
                save_hubs_config(&config)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "action": "removed", "name": name}));
                } else {
                    println!("Removed profile '{}'", name);
                }
            } else {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Profile '{}' not found", name)}));
                }
                println!("Profile '{}' not found", name);
            }
            Ok(())
        }
    }
}

pub(crate) async fn cmd_remote_ping(
    conn: &RemoteConn<'_>,
    session: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_ping_inner(conn, session, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote ping", timeout_secs);
        }
    }
}

async fn cmd_remote_ping_inner(
    conn: &RemoteConn<'_>,
    session: Option<&str>,
    json: bool,
) -> Result<()> {
    let start = std::time::Instant::now();
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };
    let auth_ms = start.elapsed().as_millis();

    match session {
        Some(target) => {
            let ping_start = std::time::Instant::now();
            let params = serde_json::json!({ "target": target });
            match rpc_client.call("termlink.ping", serde_json::json!("ping"), params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let total_ms = start.elapsed().as_millis();
                    let rpc_ms = ping_start.elapsed().as_millis();
                    if json {
                        println!("{}", serde_json::json!({
                            "ok": true,
                            "hub": conn.hub,
                            "session": target,
                            "id": r.result["id"],
                            "display_name": r.result["display_name"],
                            "state": r.result["state"],
                            "total_ms": total_ms as u64,
                            "auth_ms": auth_ms as u64,
                            "rpc_ms": rpc_ms as u64,
                        }));
                    } else {
                        println!(
                            "PONG from {} ({}) on {} — state: {} — {}ms (auth: {}ms, rpc: {}ms)",
                            r.result["id"].as_str().unwrap_or("?"),
                            r.result["display_name"].as_str().unwrap_or("?"),
                            conn.hub,
                            r.result["state"].as_str().unwrap_or("?"),
                            total_ms, auth_ms, rpc_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                        format!("Session '{}' not found on {}", target, conn.hub)
                    } else {
                        format!("Ping failed: {} {}", e.error.code, e.error.message)
                    };
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": target, "error": msg}));
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": target, "error": format!("Ping error: {e}")}));
                    }
                    anyhow::bail!("Ping error: {}", e);
                }
            }
        }
        None => {
            let discover_start = std::time::Instant::now();
            match rpc_client.call("session.discover", serde_json::json!("discover"), serde_json::json!({})).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let total_ms = start.elapsed().as_millis();
                    let discover_ms = discover_start.elapsed().as_millis();
                    let count = r.result["sessions"].as_array().map(|a| a.len()).unwrap_or(0);
                    if json {
                        println!("{}", serde_json::json!({
                            "ok": true,
                            "hub": conn.hub,
                            "sessions": count,
                            "total_ms": total_ms as u64,
                            "auth_ms": auth_ms as u64,
                            "discover_ms": discover_ms as u64,
                        }));
                    } else {
                        println!(
                            "PONG from hub {} — {} session(s) — {}ms (auth: {}ms, discover: {}ms)",
                            conn.hub, count, total_ms, auth_ms, discover_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = format!("Hub ping failed: {} {}", e.error.code, e.error.message);
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": msg}));
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Hub ping error: {e}")}));
                    }
                    anyhow::bail!("Hub ping error: {}", e);
                }
            }
        }
    }
}

pub(crate) async fn cmd_remote_list(
    conn: &RemoteConn<'_>,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    display: &ListDisplayOpts,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_list_inner(conn, name, tags, roles, cap, display)).await {
        Ok(result) => result,
        Err(_) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote list", timeout_secs);
        }
    }
}

async fn cmd_remote_list_inner(
    conn: &RemoteConn<'_>,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    display: &ListDisplayOpts,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut params = serde_json::json!({});
    if let Some(n) = name {
        params["name"] = serde_json::json!(n);
    }
    if let Some(t) = tags {
        let tag_list: Vec<&str> = t.split(',').map(|s| s.trim()).collect();
        params["tags"] = serde_json::json!(tag_list);
    }
    if let Some(r) = roles {
        let role_list: Vec<&str> = r.split(',').map(|s| s.trim()).collect();
        params["roles"] = serde_json::json!(role_list);
    }
    if let Some(c) = cap {
        let cap_list: Vec<&str> = c.split(',').map(|s| s.trim()).collect();
        params["capabilities"] = serde_json::json!(cap_list);
    }

    match rpc_client.call("session.discover", serde_json::json!("discover"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let sessions = r.result["sessions"].as_array();
            let sessions = sessions.map(|a| a.as_slice()).unwrap_or(&[]);

            if display.first {
                if let Some(s) = sessions.first() {
                    if display.json {
                        let mut wrapped = serde_json::json!({"ok": true});
                        if let Some(obj) = s.as_object() {
                            for (k, v) in obj {
                                wrapped[k] = v.clone();
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&wrapped)?);
                    } else {
                        println!("{}", s["display_name"].as_str().unwrap_or("?"));
                    }
                } else {
                    if display.json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": "No matching sessions"}));
                    }
                    std::process::exit(1);
                }
                return Ok(());
            }

            if display.count {
                if display.json {
                    println!("{}", serde_json::json!({"ok": true, "count": sessions.len()}));
                } else {
                    println!("{}", sessions.len());
                }
                return Ok(());
            }

            if display.names {
                for s in sessions {
                    println!("{}", s["display_name"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if display.ids {
                for s in sessions {
                    println!("{}", s["id"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if display.json {
                println!("{}", serde_json::json!({"ok": true, "sessions": sessions}));
                return Ok(());
            }

            if sessions.is_empty() {
                if !display.no_header {
                    println!("No sessions on {}.", conn.hub);
                }
                return Ok(());
            }

            if !display.no_header {
                // T-1441: surface identity_fingerprint between NAME and STATE
                // so operators copy-paste it into `--target-fp` for chat-arc
                // cross-host contact (T-1429 / T-1431).
                println!(
                    "{:<14} {:<16} {:<17} {:<14} {:<8} TAGS",
                    "ID", "NAME", "FP", "STATE", "PID"
                );
                println!("{}", "-".repeat(80));
            }

            for s in sessions {
                let id = s["id"].as_str().unwrap_or("?");
                let display_name = s["display_name"].as_str().unwrap_or("?");
                let state = s["state"].as_str().unwrap_or("?");
                let pid = s["pid"].as_u64().unwrap_or(0);
                let fp = s["identity_fingerprint"].as_str().unwrap_or("-");
                let tags_arr = s["tags"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
                    .unwrap_or_default();
                println!(
                    "{:<14} {:<16} {:<17} {:<14} {:<8} {}",
                    truncate(id, 13),
                    truncate(display_name, 15),
                    fp,
                    state,
                    pid,
                    tags_arr,
                );
            }

            if !display.no_header {
                println!();
                println!("{} session(s) on {}", sessions.len(), conn.hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = format!("Discover failed: {} {}", e.error.code, e.error.message);
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Discover error: {e}")}));
            }
            anyhow::bail!("Discover error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_status(
    conn: &RemoteConn<'_>,
    session: &str,
    json: bool,
    short: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_status_inner(conn, session, json, short)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote status", timeout_secs);
        }
    }
}

async fn cmd_remote_status_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    json: bool,
    short: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let params = serde_json::json!({
        "target": session,
    });

    match rpc_client.call("query.status", serde_json::json!("status"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let result = &r.result;

            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", wrapped);
                return Ok(());
            }

            if short {
                println!("{} {} {}",
                    result["display_name"].as_str().unwrap_or("?"),
                    result["state"].as_str().unwrap_or("?"),
                    result["pid"].as_u64().unwrap_or(0),
                );
                return Ok(());
            }

            println!("Session: {} (on {})", result["id"].as_str().unwrap_or("?"), conn.hub);
            println!("  Name:        {}", result["display_name"].as_str().unwrap_or("?"));
            println!("  State:       {}", result["state"].as_str().unwrap_or("?"));
            println!("  PID:         {}", result["pid"]);
            println!("  Created:     {}", result["created_at"].as_str().unwrap_or("?"));
            println!("  Heartbeat:   {}", result["heartbeat_at"].as_str().unwrap_or("?"));
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
                if !cap_strs.is_empty() {
                    println!("  Capabilities: {}", cap_strs.join(", "));
                }
            }
            if let Some(tags) = result.get("tags").and_then(|t| t.as_array())
                && !tags.is_empty() {
                    let tag_strs: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                    println!("  Tags:        {}", tag_strs.join(", "));
                }
            if let Some(roles) = result.get("roles").and_then(|r| r.as_array())
                && !roles.is_empty() {
                    let role_strs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                    println!("  Roles:       {}", role_strs.join(", "));
                }
            if let Some(mode) = result.get("terminal_mode") {
                let raw = mode["raw"].as_bool().unwrap_or(false);
                let canonical = mode["canonical"].as_bool().unwrap_or(false);
                let echo = mode["echo"].as_bool().unwrap_or(false);
                let alt_screen = mode["alternate_screen"].as_bool().unwrap_or(false);
                let mode_label = if raw { "raw" }
                    else if canonical && echo { "canonical+echo" }
                    else if canonical { "canonical" }
                    else { "cooked" };
                print!("  Term Mode:   {}", mode_label);
                if alt_screen { print!(" (alternate screen)"); }
                println!();
            }
            if let Some(meta) = result.get("metadata")
                && let Some(shell) = meta.get("shell").and_then(|s| s.as_str()) {
                    println!("  Shell:       {}", shell);
                }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Status query failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Status query error: {e}")}));
            }
            anyhow::bail!("Status query error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_inject(
    conn: &RemoteConn<'_>,
    opts: &RemoteInjectOpts<'_>,
) -> Result<()> {
    let RemoteInjectOpts { session, text, enter, key, delay_ms, json, timeout_secs } = *opts;
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_inject_inner(conn, session, text, enter, key, delay_ms, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote inject", timeout_secs);
        }
    }
}

async fn cmd_remote_inject_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    text: &str,
    enter: bool,
    key: Option<&str>,
    delay_ms: u64,
    json: bool,
) -> Result<()> {
    let mut client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut keys = Vec::new();
    if let Some(key_name) = key {
        keys.push(serde_json::json!({ "type": "key", "value": key_name }));
    } else {
        keys.push(serde_json::json!({ "type": "text", "value": text }));
    }
    if enter {
        keys.push(serde_json::json!({ "type": "key", "value": "Enter" }));
    }

    let inject_params = serde_json::json!({
        "target": session,
        "keys": keys,
        "inject_delay_ms": delay_ms,
    });

    match client.call("command.inject", serde_json::json!("inject"), inject_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = r.result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
            } else {
                let bytes = r.result["bytes_len"].as_u64().unwrap_or(0);
                println!("Injected {} bytes into '{}' on {}", bytes, session, conn.hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Inject failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Inject error: {e}")}));
            }
            anyhow::bail!("Inject error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_send_file(
    conn: &RemoteConn<'_>,
    session: &str,
    path: &str,
    chunk_size: usize,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_send_file_inner(conn, session, path, chunk_size, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote file transfer", timeout_secs);
        }
    }
}

async fn cmd_remote_send_file_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    path: &str,
    chunk_size: usize,
    json: bool,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let file_path = std::path::Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to read file: {path}: {e}")}));
            }
            return Err(e).context(format!("Failed to read file: {}", path));
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let chunk_sz = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total_chunks = file_data.len().div_ceil(chunk_sz) as u32;
    let transfer_id = generate_request_id().replace("req-", "xfer-");

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    let mut client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    // T-1249: Try the new channel.post + artifact.put path first against the
    // remote hub. On LegacyOnly fall through to the 3-phase event-emit below.
    {
        use termlink_session::artifact::{
            send_artifact_via_client, ArtifactManifest, SendOutcome, SendPath,
        };
        use termlink_session::hub_capabilities::shared_cache;
        use termlink_session::inbox_channel::FallbackCtx;

        let identity = super::channel::load_identity_or_create()?;
        let cache = shared_cache();
        let mut ctx = FallbackCtx::new();
        let manifest = ArtifactManifest {
            filename: filename.clone(),
            size,
            from: format!("remote-cli-{}", std::process::id()),
            transfer_id: Some(transfer_id.clone()),
            content_type: None,
        };
        match send_artifact_via_client(
            &mut client,
            conn.hub,
            session,
            &file_data,
            &manifest,
            &identity,
            cache,
            &mut ctx,
        )
        .await
        {
            Ok(SendOutcome::Sent {
                channel_offset,
                path: used_path,
                ..
            }) => {
                let path_label = match used_path {
                    SendPath::Inline => "channel.inline",
                    SendPath::Chunked => "channel.artifact",
                };
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "filename": filename,
                            "size": size,
                            "via": path_label,
                            "spooled": false,
                            "chunks": total_chunks,
                            "transfer_id": transfer_id,
                            "sha256": sha256,
                            "target": session,
                            "channel_offset": channel_offset,
                            "artifact_sha256": sha256,
                            "hub": conn.hub,
                        })
                    );
                } else {
                    eprintln!(
                        "Sent '{}' ({} bytes) to '{}' on {} via {} → channel.offset={}, sha256={}",
                        filename, size, session, conn.hub, path_label, channel_offset, sha256
                    );
                }
                return Ok(());
            }
            Ok(SendOutcome::LegacyOnly) => {
                tracing::debug!(
                    hub = %conn.hub,
                    target = %session,
                    "T-1249: remote hub doesn't advertise artifact.put — falling back to legacy events"
                );
            }
            Err(e) => {
                tracing::warn!(
                    hub = %conn.hub,
                    target = %session,
                    error = %e,
                    "T-1249: remote new-path send failed — falling back to legacy events"
                );
            }
        }
    }

    eprintln!(
        "Sending '{}' ({} bytes, {} chunks) to '{}' on {}",
        filename, size, total_chunks, session, conn.hub
    );

    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: filename.clone(),
        size,
        total_chunks,
        from: format!("remote-cli-{}", std::process::id()),
    };
    let init_payload = serde_json::to_value(&init)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::INIT,
        "payload": init_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Failed to emit file.init: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit file.init: {e}")}));
            }
            anyhow::bail!("Failed to emit file.init: {}", e);
        }
    }

    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in file_data.chunks(chunk_sz).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let chunk_payload = serde_json::to_value(&chunk)?;
        let emit_params = serde_json::json!({
            "target": session,
            "topic": file_topic::CHUNK,
            "payload": chunk_payload,
        });
        match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
            Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
            Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                let msg = format!("Failed to emit chunk {}/{}: {} {}", i + 1, total_chunks, e.error.code, e.error.message);
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
                }
                anyhow::bail!("{}", msg);
            }
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e)}));
                }
                anyhow::bail!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e);
            }
        }
        if total_chunks > 1 {
            eprint!("\r  Chunk {}/{}", i + 1, total_chunks);
        }
    }
    if total_chunks > 1 {
        eprintln!();
    }

    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: sha256.clone(),
    };
    let complete_payload = serde_json::to_value(&complete)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::COMPLETE,
        "payload": complete_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = format!("Failed to emit file.complete: {} {}", e.error.code, e.error.message);
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit file.complete: {e}")}));
            }
            anyhow::bail!("Failed to emit file.complete: {}", e);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "transfer_id": transfer_id,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "sha256": sha256,
            "hub": conn.hub,
            "session": session,
        }));
    } else {
        eprintln!("Transfer complete. SHA-256: {}", sha256);
        println!("Sent '{}' ({} bytes) to '{}' on {}", filename, size, session, conn.hub);
    }

    Ok(())
}

pub(crate) async fn cmd_remote_events(
    conn: &RemoteConn<'_>,
    topic_filter: Option<&str>,
    targets_csv: Option<&str>,
    interval_ms: u64,
    max_count: u64,
    json: bool,
    payload_only: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let targets: Vec<&str> = targets_csv
        .map(|t| t.split(',').map(|s| s.trim()).collect())
        .unwrap_or_default();

    eprintln!("Watching events on {}. Press Ctrl+C to stop.", conn.hub);
    if let Some(t) = topic_filter {
        eprintln!("  Topic filter: {}", t);
    }
    if !targets.is_empty() {
        eprintln!("  Targets: {}", targets.join(", "));
    }
    eprintln!();

    let subscribe_timeout_ms = interval_ms.max(500);
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                eprintln!();
                eprintln!("Stopped. {} event(s) collected.", total_received);
                break;
            }
            collect_result = async {
                let mut params = serde_json::json!({
                    "timeout_ms": subscribe_timeout_ms,
                });
                if !targets.is_empty() {
                    params["targets"] = serde_json::json!(targets);
                }
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                if !cursors.as_object().is_none_or(|m| m.is_empty()) {
                    params["since"] = cursors.clone();
                }

                rpc_client.call("event.collect", serde_json::json!("collect"), params).await
            } => {
                match collect_result {
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                        if let Some(events) = r.result["events"].as_array() {
                            for event in events {
                                total_received += 1;

                                if payload_only {
                                    let payload = &event["payload"];
                                    if !payload.is_null() {
                                        println!("{}", serde_json::to_string(payload).unwrap_or_default());
                                    }
                                } else if json {
                                    let mut wrapped = serde_json::json!({"ok": true});
                                    if let Some(obj) = event.as_object() {
                                        for (k, v) in obj {
                                            wrapped[k] = v.clone();
                                        }
                                    }
                                    println!("{}", serde_json::to_string(&wrapped).unwrap_or_default());
                                } else {
                                    let session_name = event["session_name"].as_str().unwrap_or("?");
                                    let seq = event["seq"].as_u64().unwrap_or(0);
                                    let topic = event["topic"].as_str().unwrap_or("?");
                                    let payload = &event["payload"];
                                    let ts = event["timestamp"].as_u64().unwrap_or(0);

                                    if payload.is_null()
                                        || payload.as_object().is_some_and(|o| o.is_empty())
                                    {
                                        println!("[{session_name}#{seq}] {topic} (t={ts})");
                                    } else {
                                        println!(
                                            "[{session_name}#{seq}] {topic}: {} (t={ts})",
                                            serde_json::to_string(payload).unwrap_or_default()
                                        );
                                    }
                                }
                            }
                        }

                        if let Some(new_cursors) = r.result.get("cursors")
                            && let Some(obj) = new_cursors.as_object()
                        {
                            for (k, v) in obj {
                                cursors[k] = v.clone();
                            }
                        }

                        if max_count > 0 && total_received >= max_count {
                            eprintln!();
                            eprintln!("{} event(s) collected (limit reached).", total_received);
                            break;
                        }
                    }
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                        eprintln!("Collect error: {} {}. Retrying...", e.error.code, e.error.message);
                    }
                    Err(e) => {
                        eprintln!("Hub connection error: {}. Retrying...", e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_remote_exec(
    conn: &RemoteConn<'_>,
    session: &str,
    command: &str,
    timeout: u64,
    cwd: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut params = serde_json::json!({
        "target": session,
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    match rpc_client.call("command.execute", serde_json::json!("exec"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let result = &r.result;

            if json {
                let exit_code = result["exit_code"].as_i64().unwrap_or(0);
                let mut wrapped = serde_json::json!({"ok": exit_code == 0});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", wrapped);
                if exit_code != 0 {
                    std::process::exit(exit_code as i32);
                }
                return Ok(());
            }

            let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
            let stdout = result["stdout"].as_str().unwrap_or("");
            let stderr = result["stderr"].as_str().unwrap_or("");

            if !stdout.is_empty() {
                print!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }

            if exit_code != 0 {
                std::process::exit(exit_code as i32);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Execution failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Execution error: {e}")}));
            }
            anyhow::bail!("Execution error: {}", e);
        }
    }
}

/// Remote inbox operations — query/clear inbox on a remote hub via RPC (T-1009).
pub(crate) async fn cmd_remote_inbox(
    conn: &RemoteConn<'_>,
    action: RemoteInboxAction,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_inbox_inner(conn, action)).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!("Timeout after {}s waiting for remote inbox RPC", timeout_secs),
    }
}

async fn cmd_remote_inbox_inner(
    conn: &RemoteConn<'_>,
    action: RemoteInboxAction,
) -> Result<()> {
    let mut rpc_client = connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope)
        .await
        .context("Failed to connect to remote hub")?;

    match action {
        RemoteInboxAction::Status { json } => {
            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            let status = termlink_session::inbox_channel::status_via_channel_with_client(
                &mut rpc_client,
                conn.hub,
                cache,
                &mut ctx,
            )
            .await
            .context("inbox.status (channel-aware) failed")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else if status.total_transfers == 0 {
                println!("Inbox on {}: empty (no pending transfers)", conn.hub);
            } else {
                println!(
                    "Inbox on {}: {} pending transfer(s)",
                    conn.hub, status.total_transfers
                );
                for t in &status.targets {
                    println!("  {} — {} transfer(s)", t.target, t.pending);
                }
            }
        }
        RemoteInboxAction::List { target, json } => {
            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            let entries = termlink_session::inbox_channel::list_via_channel_with_client(
                &mut rpc_client,
                conn.hub,
                &target,
                cache,
                &mut ctx,
            )
            .await
            .context("inbox.list (channel-aware) failed")?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "transfers": entries }))?
                );
            } else if entries.is_empty() {
                println!("No pending transfers for '{}' on {}", target, conn.hub);
            } else {
                println!("Pending transfers for '{}' on {}:", target, conn.hub);
                for e in &entries {
                    println!("  {} — {} ({} bytes)", e.transfer_id, e.filename, e.size);
                }
            }
        }
        RemoteInboxAction::Clear { target, all, json } => {
            let scope = if all {
                termlink_session::inbox_channel::ClearScope::All
            } else if let Some(ref t) = target {
                termlink_session::inbox_channel::ClearScope::Target(t.clone())
            } else {
                anyhow::bail!("Specify a target name or use --all");
            };
            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            let result = termlink_session::inbox_channel::clear_via_channel_with_client(
                &mut rpc_client,
                conn.hub,
                scope,
                cache,
                &mut ctx,
            )
            .await
            .context("inbox.clear (channel-aware) failed")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "Cleared {} transfer(s) for '{}' on {}",
                    result.cleared, result.target, conn.hub
                );
            }
        }
    }
    Ok(())
}

/// T-1650: heal-readiness footer for `remote profile list`. Returns a one-line
/// recommendation when any profile lacks `bootstrap_from`, naming the count.
/// Returns None when all profiles declare it (suppress nag once configured).
pub(crate) fn heal_readiness_footer(
    hubs: &std::collections::HashMap<String, crate::config::HubEntry>,
) -> Option<String> {
    let undeclared = hubs.values().filter(|e| e.bootstrap_from.is_none()).count();
    if undeclared == 0 {
        return None;
    }
    Some(format!(
        "{} profile(s) lack `bootstrap_from` — declare with `termlink remote profile add --bootstrap-from ssh:<host>` (T-1291) to enable one-flag heal",
        undeclared
    ))
}

/// T-1649: format the per-hub HMAC-mismatch diagnosis surfaced by `fleet doctor`'s
/// layered probe (cmd_fleet_doctor L3 AUTH failure). Carries the profile name and
/// the declared-channel-aware heal-source argument, so operators get a copy-pasteable
/// incantation instead of `<profile>` + `ssh:<host>` literal placeholders.
pub(crate) fn format_hmac_mismatch_diagnosis(
    profile_name: &str,
    entry: &crate::config::HubEntry,
) -> String {
    format!(
        "HMAC secret mismatch — run: termlink fleet reauth {} {}",
        profile_name, heal_bootstrap_hint(entry, &entry.address)
    )
}

/// T-1648: pick the right `--bootstrap-from` argument to recommend in a heal hint.
/// If the profile declares `bootstrap_from` in hubs.toml, suggest `auto` (T-1291);
/// otherwise fall back to the literal `ssh:<host>` form and append a one-line tip
/// pointing the operator at the declarative path for next time.
pub(crate) fn heal_bootstrap_hint(
    entry: &crate::config::HubEntry,
    address: &str,
) -> String {
    if entry.bootstrap_from.is_some() {
        "--bootstrap-from auto".to_string()
    } else {
        let host = address.split(':').next().unwrap_or(address);
        format!(
            "--bootstrap-from ssh:{host}  (tip: add `bootstrap_from = \"ssh:{host}\"` to this profile in hubs.toml to use `--bootstrap-from auto` next time)"
        )
    }
}

/// T-1652: warn when `secret_file`'s Unix perms grant group or world access.
/// Returns `Some(remediation)` for any mode where the bottom 6 bits are set
/// (group rwx or other rwx), `None` for 0o600 (or owner-only variants like
/// 0o400/0o700), and `None` when metadata cannot be read — the "secret file
/// missing" path elsewhere produces the right error for absent files; this
/// helper stays silent there to keep the display clean.
///
/// Closes G-011 sub-point #4 (the 2026-04 incident: a peer-shared secret
/// observed at chmod 644 sitting world-readable in a home directory).
#[cfg(unix)]
pub(crate) fn secret_file_perms_warning(path: &std::path::Path) -> Option<String> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).ok()?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        Some(format!(
            "secret_file perms 0o{:03o} expose secret to group/world — run: chmod 600 {}",
            mode,
            path.display()
        ))
    } else {
        None
    }
}

#[cfg(not(unix))]
pub(crate) fn secret_file_perms_warning(_path: &std::path::Path) -> Option<String> {
    None
}

/// T-1652: best-effort `~/` → `$HOME/` expansion for `secret_file` values from
/// hubs.toml. Returns the original string if no leading `~/` or HOME is unset
/// — the perms helper silently returns None on stat failure, so an unexpanded
/// path becomes a no-op warning rather than a false alarm.
pub(crate) fn expand_secret_file_path(raw: &str) -> std::path::PathBuf {
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home).join(rest);
        }
    }
    std::path::PathBuf::from(raw)
}

/// T-1102: One-screen fleet overview for human operators.
/// Shows each hub's status, session count, version, latency, and actionable fixes.
pub(crate) async fn cmd_fleet_status(
    json: bool,
    timeout_secs: u64,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;

    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({
                "ok": true,
                "fleet": [],
                "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
                "actions": []
            }))?);
        } else {
            eprintln!("No hubs configured. Add hubs with: termlink remote profile add <name> <host:port> --secret-file <path>");
        }
        return Ok(());
    }

    let mut hub_entries: Vec<serde_json::Value> = Vec::new();
    let mut actions: Vec<String> = Vec::new();
    let mut up_count = 0u32;
    let mut down_count = 0u32;
    let mut auth_fail_count = 0u32;

    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();

    for name in &hub_names {
        let entry = &config.hubs[*name];
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let connect_start = std::time::Instant::now();

        // T-1652: surface insecure secret_file perms regardless of whether
        // the hub itself comes up. The risk (32-byte HMAC sitting
        // world-readable) is independent of reachability — flagging it here
        // means the next `fleet status` run after a chmod regression fires
        // immediately, not when an auth incident finally reveals the leak.
        if let Some(path_raw) = entry.secret_file.as_deref()
            && let Some(warning) = secret_file_perms_warning(&expand_secret_file_path(path_raw))
        {
            actions.push(format!("{}: {}", name, warning));
        }

        let result = tokio::time::timeout(
            timeout_dur,
            connect_remote_hub(
                &entry.address,
                entry.secret_file.as_deref(),
                entry.secret.as_deref(),
                entry.scope.as_deref().unwrap_or("execute"),
            ),
        ).await;

        match result {
            Ok(Ok(mut client)) => {
                let latency = connect_start.elapsed().as_millis();
                up_count += 1;

                // Query session count and optionally names
                let (session_count, session_names) = match client.call(
                    "session.discover", json!("fleet-sd"), json!({}),
                ).await {
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                        let sessions = r.result["sessions"].as_array();
                        let count = sessions.map(|s| s.len()).unwrap_or(0);
                        let names: Vec<String> = sessions
                            .map(|s| s.iter()
                                .filter_map(|sess| sess["display_name"].as_str().map(String::from))
                                .collect())
                            .unwrap_or_default();
                        (count, names)
                    }
                    _ => (0, Vec::new()),
                };

                let mut hub_entry = json!({
                    "hub": name,
                    "address": entry.address,
                    "status": "up",
                    "latency_ms": latency,
                    "sessions": session_count,
                });
                if verbose {
                    hub_entry["session_names"] = json!(session_names);
                }
                hub_entries.push(hub_entry);

                if !json {
                    eprintln!("  \x1b[32mUP\x1b[0m    {:<20} {:<24} {:>3} sessions  ({}ms)",
                        name, entry.address, session_count, latency);
                    if verbose && !session_names.is_empty() {
                        for sname in &session_names {
                            eprintln!("         \x1b[2m- {}\x1b[0m", sname);
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                // T-1183: use {:#} (anyhow alternate) so the inner chain is
                // visible to the is_auth substring checks. Same PL-046
                // pattern T-1181 fixed in cmd_fleet_doctor — default Display
                // drops .context() wrappers, collapsing TOFU VIOLATION under
                // the outer "Cannot connect" context.
                let msg = format!("{:#}", e);
                let is_auth = msg.contains("invalid signature")
                    || msg.contains("Token validation failed")
                    || msg.contains("TOFU VIOLATION")
                    || msg.contains("fingerprint changed");

                if is_auth {
                    auth_fail_count += 1;
                    hub_entries.push(json!({
                        "hub": name,
                        "address": entry.address,
                        "status": "auth-fail",
                        "error": &msg,
                    }));
                    if !json {
                        eprintln!("  \x1b[33mAUTH\x1b[0m  {:<20} {:<24} secret mismatch — hub was restarted with a new secret",
                            name, entry.address);
                    }
                    actions.push(format!(
                        "{}: Reauth needed — termlink fleet reauth {} {}",
                        name, name, heal_bootstrap_hint(entry, &entry.address)
                    ));
                } else {
                    down_count += 1;
                    hub_entries.push(json!({
                        "hub": name,
                        "address": entry.address,
                        "status": "down",
                        "error": &msg,
                    }));
                    if !json {
                        eprintln!("  \x1b[31mDOWN\x1b[0m  {:<20} {:<24} {}",
                            name, entry.address, msg);
                    }
                    if msg.contains("Connection refused") {
                        // Specifically: kernel listening, no process bound to port — RST.
                        // Means hub binary isn't running but host is reachable.
                        actions.push(format!(
                            "{}: Hub process not running — start via: ssh root@{} systemctl start termlink-hub",
                            name, entry.address.split(':').next().unwrap_or(&entry.address)
                        ));
                    } else if msg.contains("No route to host")
                        || msg.contains("Network is unreachable")
                        || msg.contains("Cannot connect")
                    {
                        // T-1614: kernel can't even reach the host — different probe path
                        // than Connection-refused. Pre-T-1614 this branch sent operators
                        // chasing systemd on a host they couldn't ssh to.
                        actions.push(classify_unreachable_hint(name, &entry.address));
                    } else if msg.contains("Secret file not found") {
                        // T-1613: stale-test-residue vs genuinely-missing classification.
                        // Cargo's TempDir places per-test fixtures under /tmp/tmp.<rand>/...
                        // — once the test process exits those paths vanish but the
                        // saved hubs.toml profile keeps pointing at them, leaving
                        // permanent DOWN noise in fleet status. Detect and suggest
                        // direct removal. Other "Secret file not found" cases
                        // (operator-supplied path that simply isn't there yet) get
                        // the inspection + reauth incantation instead.
                        if msg.contains("/tmp/tmp.") {
                            actions.push(format!(
                                "{}: Stale test-fixture profile (secret_file under /tmp/tmp.* — cargo TempDir residue). Remove with: termlink remote profile remove {}",
                                name, name
                            ));
                        } else {
                            actions.push(format!(
                                "{}: Secret file missing — inspect profile (`termlink remote profile list {}`), then either fix the secret_file path or run `termlink fleet reauth {} {}`",
                                name, name, name, heal_bootstrap_hint(entry, &entry.address)
                            ));
                        }
                    } else {
                        actions.push(format!("{}: {}", name, msg));
                    }
                }

                // Track failure for learning/concern auto-register
                let _ = maybe_record_auth_mismatch_learning(name, &entry.address, &msg);
                let _ = maybe_track_fleet_failure(name, &entry.address, auth_mismatch_class(&msg));
            }
            Err(_) => {
                down_count += 1;
                hub_entries.push(json!({
                    "hub": name,
                    "address": entry.address,
                    "status": "timeout",
                }));
                if !json {
                    eprintln!("  \x1b[31mDOWN\x1b[0m  {:<20} {:<24} timeout after {}s",
                        name, entry.address, timeout_secs);
                }
                // T-1614: classify by address kind for actionable hint.
                // Generic "check connectivity" restates the symptom; the operator
                // wants to know WHICH probe to run first. Helper distinguishes
                // loopback / RFC5737 / RFC1918 / public.
                actions.push(classify_unreachable_hint(name, &entry.address));
            }
        }
    }

    let total = hub_names.len() as u32;

    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": down_count == 0 && auth_fail_count == 0,
            "fleet": hub_entries,
            "summary": {
                "total": total,
                "up": up_count,
                "down": down_count,
                "auth_fail": auth_fail_count,
            },
            "actions": actions,
        }))?);
    } else {
        eprintln!();
        if up_count == total {
            eprintln!("  FLEET: \x1b[32mall {} hubs operational\x1b[0m", total);
        } else {
            eprintln!("  FLEET: {} hub(s), \x1b[32m{} up\x1b[0m, \x1b[31m{} down\x1b[0m, \x1b[33m{} auth-fail\x1b[0m",
                total, up_count, down_count, auth_fail_count);
        }

        if !actions.is_empty() {
            eprintln!();
            eprintln!("  ACTIONS NEEDED:");
            for (i, action) in actions.iter().enumerate() {
                eprintln!("    {}. {}", i + 1, action);
            }
        }
        eprintln!();
    }

    Ok(())
}

/// T-1667: per-hub state observed at each watch cycle. `(connectivity_status,
/// pin_check_status, total_legacy_invocations)`. Used to compute cycle-over-cycle
/// state diffs. `BTreeMap` (not `HashMap`) keeps output ordering stable across
/// cycles so visual diffs work.
type WatchHubState = (String, Option<String>, Option<u64>);

/// T-1667: continuous-monitoring loop dispatched from `cmd_fleet_doctor` when
/// `--watch <secs>` is set. Re-spawns self via `std::env::current_exe()` with
/// `--json` each cycle, parses the result, tracks per-hub state in a
/// `BTreeMap`, and emits ONLY changes after the baseline cycle.
/// T-1671: append one NDJSON line to `~/.termlink/rotation.log` per per-hub
/// state change. Best-effort: write failures (disk full, permission denied)
/// go to stderr but never crash the watch. Append-only; operators handle log
/// rotation via logrotate or manual truncation.
fn rotation_log_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(home).join(".termlink").join("rotation.log"))
}

/// T-1685: parallel audit log for heal actions. `rotation.log` (T-1671)
/// captures state-transition events; this captures the operator-actionable
/// response. Each `--auto-heal` decision — whether it fired live, skipped
/// for missing anchor, or was a dry-run preview — appends one NDJSON line.
/// Best-effort: write failures emit to stderr but never block the heal or
/// the watch loop.
///
/// Schema (NDJSON):
///   { ts, hub, mode, trigger, action, bootstrap_from }
///
/// mode:    "watch" | "one-shot"
/// trigger: "cert-drift" | "auth-mismatch"
/// action:  "fired" | "skipped-no-anchor" | "dry-run"
fn heal_log_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(home).join(".termlink").join("heal.log"))
}

fn append_heal_log(
    hub: &str,
    mode: &str,
    trigger: &str,
    action: &str,
    bootstrap_from: Option<&str>,
) {
    let Some(path) = heal_log_path() else { return };
    let parent = path.parent();
    if let Some(p) = parent
        && !p.exists()
        && let Err(e) = std::fs::create_dir_all(p)
    {
        eprintln!(
            "{} heal.log mkdir failed: {} (logging skipped)",
            crate::manifest::now_rfc3339(),
            e
        );
        return;
    }
    let entry = serde_json::json!({
        "ts": crate::manifest::now_rfc3339(),
        "hub": hub,
        "mode": mode,
        "trigger": trigger,
        "action": action,
        "bootstrap_from": bootstrap_from,
    });
    let line = format!("{}\n", entry);
    use std::io::Write;
    let res = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = res {
        eprintln!(
            "{} heal.log write failed ({}): {} (entry dropped, heal continues)",
            crate::manifest::now_rfc3339(),
            path.display(),
            e
        );
    }
}

fn append_rotation_log(
    hub: &str,
    kind: &str,
    old_conn: &str,
    new_conn: &str,
    old_pin: &str,
    new_pin: &str,
    old_legacy: &str,
    new_legacy: &str,
) {
    let Some(path) = rotation_log_path() else { return };
    let parent = path.parent();
    if let Some(p) = parent
        && !p.exists()
        && let Err(e) = std::fs::create_dir_all(p)
    {
        eprintln!(
            "{} watch: rotation.log mkdir failed: {} (logging skipped)",
            crate::manifest::now_rfc3339(),
            e
        );
        return;
    }
    let entry = serde_json::json!({
        "ts": crate::manifest::now_rfc3339(),
        "hub": hub,
        "kind": kind,
        "old_conn": old_conn,
        "new_conn": new_conn,
        "old_pin": old_pin,
        "new_pin": new_pin,
        "old_legacy": old_legacy,
        "new_legacy": new_legacy,
    });
    let line = format!("{}\n", entry);
    use std::io::Write;
    let res = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = res {
        eprintln!(
            "{} watch: rotation.log write failed ({}): {} (entry dropped, watch continues)",
            crate::manifest::now_rfc3339(),
            path.display(),
            e
        );
    }
}

/// T-1671: read & filter `~/.termlink/rotation.log`. Returns the path used,
/// plus all entries that passed `since_days` + optional hub-name filter.
/// Lines that fail to parse as JSON are reported on stderr and skipped (so
/// a partially-corrupt file doesn't kill the entire history view).
pub(crate) fn cmd_fleet_history(
    since_days: u32,
    hub: Option<&str>,
    json_out: bool,
    include_heals: bool,
    analyze: bool,
) -> Result<()> {
    if !(1..=365).contains(&since_days) {
        anyhow::bail!("--since: must be 1..=365 days (got {})", since_days);
    }
    let Some(path) = rotation_log_path() else {
        anyhow::bail!("fleet history: cannot resolve $HOME/.termlink/rotation.log");
    };
    let rotation_path = path.clone();
    if !path.exists() && !include_heals {
        if json_out {
            println!("{}", serde_json::json!({
                "ok": true,
                "entries": [],
                "summary": {"total": 0, "per_hub": {}, "log_path": path.display().to_string()},
                "hint": "no rotation history yet — run `fleet doctor --watch` to start capturing"
            }));
        } else {
            println!(
                "no rotation history yet — run `fleet doctor --watch` to start capturing\n  (log path: {})",
                path.display()
            );
        }
        return Ok(());
    }
    let text = if path.exists() {
        std::fs::read_to_string(&path)
            .with_context(|| format!("fleet history: cannot read {}", path.display()))?
    } else {
        String::new()
    };

    // Compute cutoff as a unix epoch in seconds. RFC3339 timestamps written by
    // append_rotation_log have second resolution; compare by parsing each row.
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cutoff_secs = now_secs - (since_days as i64) * 86_400;

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut malformed_lines: usize = 0;
    for (lineno, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut entry: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                malformed_lines += 1;
                if malformed_lines <= 3 {
                    eprintln!(
                        "fleet history: skipping malformed line {} in {}",
                        lineno + 1,
                        path.display()
                    );
                }
                continue;
            }
        };
        let ts_str = entry.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let ts_secs = rfc3339_to_unix_secs(ts_str);
        if ts_secs < cutoff_secs {
            continue;
        }
        if let Some(want) = hub {
            let got = entry.get("hub").and_then(|v| v.as_str()).unwrap_or("");
            if got != want {
                continue;
            }
        }
        // T-1686: tag with event_type so downstream parsers can distinguish.
        if let Some(obj) = entry.as_object_mut() {
            obj.insert("event_type".into(), serde_json::Value::from("rotation"));
        }
        entries.push(entry);
    }

    // T-1686: also harvest heal.log entries when --include-heals.
    let mut heal_malformed: usize = 0;
    if include_heals {
        if let Some(hpath) = heal_log_path()
            && hpath.exists()
        {
            let htext = std::fs::read_to_string(&hpath)
                .with_context(|| format!("fleet history: cannot read {}", hpath.display()))?;
            for (lineno, raw) in htext.lines().enumerate() {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let mut entry: serde_json::Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(_) => {
                        heal_malformed += 1;
                        if heal_malformed <= 3 {
                            eprintln!(
                                "fleet history: skipping malformed heal.log line {} in {}",
                                lineno + 1,
                                hpath.display()
                            );
                        }
                        continue;
                    }
                };
                let ts_str = entry.get("ts").and_then(|v| v.as_str()).unwrap_or("");
                let ts_secs = rfc3339_to_unix_secs(ts_str);
                if ts_secs < cutoff_secs {
                    continue;
                }
                if let Some(want) = hub {
                    let got = entry.get("hub").and_then(|v| v.as_str()).unwrap_or("");
                    if got != want {
                        continue;
                    }
                }
                if let Some(obj) = entry.as_object_mut() {
                    obj.insert("event_type".into(), serde_json::Value::from("heal"));
                }
                entries.push(entry);
            }
        }
    }
    // T-1686: chronologically interleave both sources by ts when --include-heals.
    if include_heals {
        entries.sort_by_key(|e| {
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("");
            rfc3339_to_unix_secs(ts)
        });
    }

    // T-1690: --analyze short-circuits the chronological listing — classify
    // each hub by flap signature and emit the diagnostic verbatim when
    // PL-021 is suspected. Operates only on rotation entries (heal events
    // are diagnostic noise here, not symptom). Exits with code 2 when any
    // PL-021 candidate is found so cron/CI can alert on the structural
    // problem without parsing output.
    if analyze {
        let rotation_only: Vec<&serde_json::Value> = entries
            .iter()
            .filter(|e| e.get("event_type").and_then(|v| v.as_str()) != Some("heal"))
            .collect();
        let report = analyze_pl021(&rotation_only);
        emit_pl021_report(
            &report,
            since_days,
            hub,
            &rotation_path,
            json_out,
        );
        let any_candidate = report.iter().any(|h| h.verdict == HubFlapVerdict::Pl021Candidate);
        if any_candidate {
            std::process::exit(2);
        }
        return Ok(());
    }

    // T-1686: track rotation vs heal counts separately per hub when both
    // sources are merged. Without --include-heals this collapses to the
    // T-1671 rotation-only summary.
    let mut per_hub_rot: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    let mut per_hub_heal: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for e in &entries {
        let h = e
            .get("hub")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let et = e.get("event_type").and_then(|v| v.as_str()).unwrap_or("rotation");
        if et == "heal" {
            *per_hub_heal.entry(h).or_insert(0) += 1;
        } else {
            *per_hub_rot.entry(h).or_insert(0) += 1;
        }
    }

    if json_out {
        for e in &entries {
            println!("{}", e);
        }
        let summary = if include_heals {
            serde_json::json!({
                "total": entries.len(),
                "per_hub_rotation": per_hub_rot,
                "per_hub_heal": per_hub_heal,
                "since_days": since_days,
                "hub_filter": hub,
                "malformed_lines_skipped": malformed_lines,
                "heal_malformed_lines_skipped": heal_malformed,
                "rotation_log_path": rotation_path.display().to_string(),
                "heal_log_path": heal_log_path().map(|p| p.display().to_string()),
            })
        } else {
            serde_json::json!({
                "total": entries.len(),
                "per_hub": per_hub_rot,
                "since_days": since_days,
                "hub_filter": hub,
                "malformed_lines_skipped": malformed_lines,
                "log_path": rotation_path.display().to_string(),
            })
        };
        println!("{}", summary);
    } else if entries.is_empty() {
        let what = if include_heals { "events" } else { "rotation events" };
        println!(
            "no {} in the last {} day(s){}\n  (log path: {})",
            what,
            since_days,
            hub.map(|h| format!(" for hub `{}`", h)).unwrap_or_default(),
            rotation_path.display()
        );
    } else {
        for e in &entries {
            let ts = e.get("ts").and_then(|v| v.as_str()).unwrap_or("?");
            let h = e.get("hub").and_then(|v| v.as_str()).unwrap_or("?");
            let et = e.get("event_type").and_then(|v| v.as_str()).unwrap_or("rotation");
            if et == "heal" {
                let mode = e.get("mode").and_then(|v| v.as_str()).unwrap_or("?");
                let trig = e.get("trigger").and_then(|v| v.as_str()).unwrap_or("?");
                let act = e.get("action").and_then(|v| v.as_str()).unwrap_or("?");
                println!(
                    "{}  {:24} HEAL/{:6} trigger={} action={}",
                    ts, h, mode, trig, act
                );
            } else {
                let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                let oc = e.get("old_conn").and_then(|v| v.as_str()).unwrap_or("-");
                let nc = e.get("new_conn").and_then(|v| v.as_str()).unwrap_or("-");
                let op = e.get("old_pin").and_then(|v| v.as_str()).unwrap_or("-");
                let np = e.get("new_pin").and_then(|v| v.as_str()).unwrap_or("-");
                println!(
                    "{}  {:24} {:11} conn={}→{} pin={}→{}",
                    ts, h, kind, oc, nc, op, np
                );
            }
        }
        println!();
        println!(
            "Summary: {} event(s) in last {} day(s){}:",
            entries.len(),
            since_days,
            hub.map(|h| format!(" for hub `{}`", h)).unwrap_or_default()
        );
        // Merge hub keys from both maps for stable display
        let mut all_hubs: std::collections::BTreeSet<&String> = std::collections::BTreeSet::new();
        for k in per_hub_rot.keys() { all_hubs.insert(k); }
        for k in per_hub_heal.keys() { all_hubs.insert(k); }
        for h in &all_hubs {
            let r = per_hub_rot.get(*h).copied().unwrap_or(0);
            let hl = per_hub_heal.get(*h).copied().unwrap_or(0);
            if include_heals {
                println!("  {:24} rotation={:>2}  heal={:>2}", h, r, hl);
            } else {
                println!("  {:24} {:>3} event(s)", h, r);
            }
        }
        if malformed_lines > 0 {
            println!(
                "  ({} malformed rotation line(s) skipped — see stderr)",
                malformed_lines
            );
        }
        if include_heals && heal_malformed > 0 {
            println!(
                "  ({} malformed heal line(s) skipped — see stderr)",
                heal_malformed
            );
        }
    }
    Ok(())
}

/// T-1690: per-hub flap classification produced by `analyze_pl021`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HubFlapVerdict {
    /// No rotation transitions observed in window.
    Clean,
    /// Cert (TLS) rotation only — operator restart or one-off.
    CertOnly,
    /// Secret (HMAC) rotation only — partial persist or operator regen.
    SecretOnly,
    /// Both cert + secret rotated together, but only once. Could be a
    /// single nuke; insufficient evidence for PL-021 yet.
    SingleDoubleRotation,
    /// PL-021 signature: ≥2 simultaneous cert+secret rotations in window
    /// — strongly indicative of volatile runtime_dir (likely /tmp wipe).
    Pl021Candidate,
}

#[derive(Debug)]
pub(crate) struct HubFlapReport {
    pub hub: String,
    pub verdict: HubFlapVerdict,
    pub cert_transitions: u32,
    pub secret_transitions: u32,
    pub double_rotations: u32,
    pub last_double_rotation: Option<String>,
}

/// T-1690: classify each hub's rotation history into a flap verdict.
///
/// A "transition" entry's `new_pin == "drift"` with `old_pin != "drift"` is a
/// cert rotation. `new_conn == "auth-mismatch"` with `old_conn != "auth-mismatch"`
/// is a secret rotation. A single log row carrying BOTH is a "double
/// rotation" — the PL-021 signature. ≥2 double rotations in the window is
/// the candidate threshold.
pub(crate) fn analyze_pl021(entries: &[&serde_json::Value]) -> Vec<HubFlapReport> {
    use std::collections::BTreeMap;
    let mut per_hub: BTreeMap<String, HubFlapReport> = BTreeMap::new();
    for e in entries {
        if e.get("kind").and_then(|v| v.as_str()) != Some("transition") {
            continue;
        }
        let hub = e
            .get("hub")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let oc = e.get("old_conn").and_then(|v| v.as_str()).unwrap_or("");
        let nc = e.get("new_conn").and_then(|v| v.as_str()).unwrap_or("");
        let op = e.get("old_pin").and_then(|v| v.as_str()).unwrap_or("");
        let np = e.get("new_pin").and_then(|v| v.as_str()).unwrap_or("");
        let cert_now = np == "drift" && op != "drift";
        let secret_now = nc == "auth-mismatch" && oc != "auth-mismatch";
        if !cert_now && !secret_now {
            continue;
        }
        let entry_ts = e.get("ts").and_then(|v| v.as_str()).map(String::from);
        let rep = per_hub.entry(hub.clone()).or_insert_with(|| HubFlapReport {
            hub,
            verdict: HubFlapVerdict::Clean,
            cert_transitions: 0,
            secret_transitions: 0,
            double_rotations: 0,
            last_double_rotation: None,
        });
        if cert_now {
            rep.cert_transitions += 1;
        }
        if secret_now {
            rep.secret_transitions += 1;
        }
        if cert_now && secret_now {
            rep.double_rotations += 1;
            rep.last_double_rotation = entry_ts;
        }
    }
    for rep in per_hub.values_mut() {
        rep.verdict = match (rep.cert_transitions, rep.secret_transitions, rep.double_rotations) {
            (0, 0, _) => HubFlapVerdict::Clean,
            (_, _, n) if n >= 2 => HubFlapVerdict::Pl021Candidate,
            (_, _, 1) => HubFlapVerdict::SingleDoubleRotation,
            (c, 0, 0) if c > 0 => HubFlapVerdict::CertOnly,
            (0, s, 0) if s > 0 => HubFlapVerdict::SecretOnly,
            _ => HubFlapVerdict::SingleDoubleRotation,
        };
    }
    per_hub.into_values().collect()
}

/// T-1690: render the analyzer report. JSON form for machine parsing,
/// human form embeds the volatile-/tmp diagnostic command set verbatim
/// so the operator has a copy-pasteable next step.
pub(crate) fn emit_pl021_report(
    report: &[HubFlapReport],
    since_days: u32,
    hub_filter: Option<&str>,
    log_path: &std::path::Path,
    json_out: bool,
) {
    if json_out {
        let arr: Vec<serde_json::Value> = report
            .iter()
            .map(|r| {
                serde_json::json!({
                    "hub": r.hub,
                    "verdict": match r.verdict {
                        HubFlapVerdict::Clean => "clean",
                        HubFlapVerdict::CertOnly => "cert-only",
                        HubFlapVerdict::SecretOnly => "secret-only",
                        HubFlapVerdict::SingleDoubleRotation => "single-double-rotation",
                        HubFlapVerdict::Pl021Candidate => "pl021-candidate",
                    },
                    "cert_transitions": r.cert_transitions,
                    "secret_transitions": r.secret_transitions,
                    "double_rotations": r.double_rotations,
                    "last_double_rotation": r.last_double_rotation,
                })
            })
            .collect();
        let any_candidate = report.iter().any(|h| h.verdict == HubFlapVerdict::Pl021Candidate);
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "since_days": since_days,
                "hub_filter": hub_filter,
                "log_path": log_path.display().to_string(),
                "hubs": arr,
                "pl021_candidates": any_candidate,
            })
        );
        return;
    }
    let scope = hub_filter.map(|h| format!(" (hub `{}`)", h)).unwrap_or_default();
    println!(
        "PL-021 flap analysis — last {} day(s){}",
        since_days, scope
    );
    println!("  (log path: {})", log_path.display());
    println!();
    if report.is_empty() {
        println!("No rotation transitions in window. Nothing to analyze.");
        return;
    }
    let mut candidates: Vec<&HubFlapReport> = Vec::new();
    let mut single_doubles: Vec<&HubFlapReport> = Vec::new();
    let mut singles: Vec<&HubFlapReport> = Vec::new();
    let mut cleans: Vec<&HubFlapReport> = Vec::new();
    for r in report {
        match r.verdict {
            HubFlapVerdict::Pl021Candidate => candidates.push(r),
            HubFlapVerdict::SingleDoubleRotation => single_doubles.push(r),
            HubFlapVerdict::CertOnly | HubFlapVerdict::SecretOnly => singles.push(r),
            HubFlapVerdict::Clean => cleans.push(r),
        }
    }
    if !candidates.is_empty() {
        println!("PL-021 candidate(s) — BOTH cert + secret rotating, recurring:");
        for r in &candidates {
            println!(
                "  {:24} {} double-rotation(s){}",
                r.hub,
                r.double_rotations,
                r.last_double_rotation
                    .as_deref()
                    .map(|t| format!(" (last: {})", t))
                    .unwrap_or_default()
            );
        }
        println!();
        println!("Recommended next step — confirm volatile runtime_dir:");
        println!("  ls -la /tmp/termlink-0/ /var/lib/termlink/");
        println!("  mount | grep -E ' /tmp |termlink'");
        println!("  cat /usr/lib/tmpfiles.d/tmp.conf /etc/tmpfiles.d/tmp.conf 2>/dev/null");
        println!();
        println!("See CLAUDE.md \"Special case — volatile runtime_dir (T-1290 / T-1294)\"");
        println!("for the full diagnostic + fix (move runtime_dir off /tmp permanently).");
        println!();
    }
    if !single_doubles.is_empty() {
        println!("Single double-rotation (could be one-off, watch for recurrence):");
        for r in &single_doubles {
            println!(
                "  {:24} 1 double-rotation{}",
                r.hub,
                r.last_double_rotation
                    .as_deref()
                    .map(|t| format!(" (at: {})", t))
                    .unwrap_or_default()
            );
        }
        println!();
    }
    if !singles.is_empty() {
        println!("Single-axis rotation (operator restart or partial persist):");
        for r in &singles {
            let axis = match r.verdict {
                HubFlapVerdict::CertOnly => "cert-only",
                HubFlapVerdict::SecretOnly => "secret-only",
                _ => "single",
            };
            println!(
                "  {:24} {} (cert={} secret={})",
                r.hub, axis, r.cert_transitions, r.secret_transitions
            );
        }
        println!();
    }
    if !cleans.is_empty() {
        println!("Clean (no rotation transitions): {} hub(s)", cleans.len());
    }
    if candidates.is_empty() {
        println!("No PL-021 signature detected in window.");
    }
}

/// T-1671: parse an RFC3339 timestamp (as produced by `now_rfc3339()`) into a
/// unix epoch in seconds. Returns 0 on parse failure — the caller treats that
/// as "ancient" so corrupt entries get filtered by the `since` window.
fn rfc3339_to_unix_secs(ts: &str) -> i64 {
    // Expected format: YYYY-MM-DDTHH:MM:SSZ. Parse field-by-field with stdlib
    // — avoid chrono to stay consistent with the rest of this crate.
    if ts.len() < 20 || !ts.ends_with('Z') {
        return 0;
    }
    let bytes = ts.as_bytes();
    let parse_u = |start: usize, len: usize| -> Option<u32> {
        std::str::from_utf8(&bytes[start..start + len])
            .ok()?
            .parse()
            .ok()
    };
    let y = parse_u(0, 4) as Option<u32>;
    let mo = parse_u(5, 2);
    let d = parse_u(8, 2);
    let h = parse_u(11, 2);
    let mi = parse_u(14, 2);
    let s = parse_u(17, 2);
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) = (y, mo, d, h, mi, s) else {
        return 0;
    };
    // Convert (Y,M,D) to days since 1970-01-01 via Howard Hinnant civil-from-days
    // inverse — same algorithm as `unix_secs_to_iso_date` but going the other way.
    let y = y as i64;
    let mo = mo as i64;
    let d = d as i64;
    let y_shift = if mo <= 2 { y - 1 } else { y };
    let era = if y_shift >= 0 { y_shift / 400 } else { (y_shift - 399) / 400 };
    let yoe = y_shift - era * 400;
    let mp = if mo > 2 { mo - 3 } else { mo + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400 + (h as i64) * 3600 + (mi as i64) * 60 + s as i64
}

/// T-1669: spawn an operator-supplied shell command on a per-hub state change.
/// Fire-and-forget: we set the env vars, spawn via `sh -c "$cmd"`, and do NOT
/// await the child. A hanging script must not block the watch loop. The child
/// inherits stdout/stderr so operator output is visible to the same terminal.
fn fire_notify(
    cmd: &str,
    hub: &str,
    kind: &str,
    old_conn: &str,
    new_conn: &str,
    old_pin: &str,
    new_pin: &str,
    old_legacy: &str,
    new_legacy: &str,
    ts: &str,
) {
    let mut child = std::process::Command::new("sh");
    child
        .arg("-c")
        .arg(cmd)
        .env("TERMLINK_WATCH_HUB", hub)
        .env("TERMLINK_WATCH_CHANGE_KIND", kind)
        .env("TERMLINK_WATCH_OLD_CONN", old_conn)
        .env("TERMLINK_WATCH_NEW_CONN", new_conn)
        .env("TERMLINK_WATCH_OLD_PIN", old_pin)
        .env("TERMLINK_WATCH_NEW_PIN", new_pin)
        .env("TERMLINK_WATCH_OLD_LEGACY", old_legacy)
        .env("TERMLINK_WATCH_NEW_LEGACY", new_legacy)
        // T-1676: detection timestamp (RFC3339, UTC). Operator-side log
        // correlation needs the watch-loop's diff-cycle time, not the
        // notify-script's launch time. Set per change event.
        .env("TERMLINK_WATCH_TS", ts);
    match child.spawn() {
        Ok(_) => {} // fire-and-forget; child reaped by OS when it exits
        Err(e) => {
            eprintln!(
                "{} watch: --notify spawn failed for hub={}: {} (watch continues)",
                crate::manifest::now_rfc3339(),
                hub,
                e
            );
        }
    }
}

/// T-1680: built-in auto-heal — re-spawn self as `fleet reauth $hub
/// --bootstrap-from auto`. Fire-and-forget, mirrors `fire_notify`'s shape.
/// The heal sub-process does its own bootstrap-source resolution + secret
/// write, so the watch loop's only job is to launch it and continue.
///
/// Caller is responsible for gating on `new_pin == "drift"` and checking
/// that the profile has declared `bootstrap_from` (R2 — no implicit
/// defaults). This function just spawns once it's invoked.
fn fire_auto_heal(hub: &str, ts: &str) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!(
                "{ts} watch: --auto-heal cannot resolve self path for hub={hub}: {e} (watch continues)"
            );
            return;
        }
    };
    eprintln!("{ts} watch: --auto-heal spawning fleet reauth {hub} --bootstrap-from auto");
    let mut child = std::process::Command::new(&exe);
    child
        .arg("fleet")
        .arg("reauth")
        .arg(hub)
        .arg("--bootstrap-from")
        .arg("auto");
    match child.spawn() {
        Ok(_) => {} // fire-and-forget
        Err(e) => {
            eprintln!(
                "{ts} watch: --auto-heal spawn failed for hub={hub}: {e} (watch continues)"
            );
        }
    }
}

async fn cmd_fleet_doctor_watch(
    secs: u64,
    timeout_secs: u64,
    legacy_usage: bool,
    legacy_window_days: u64,
    topic_durability: bool,
    include_pin_check: bool,
    top_callers: u32,
    notify: Option<String>,
    auto_heal: bool,
    dry_run: bool,
) -> Result<()> {
    if !(5..=3600).contains(&secs) {
        anyhow::bail!("--watch: interval must be 5..=3600 seconds (got {})", secs);
    }
    let exe = std::env::current_exe()
        .context("--watch: cannot determine self path for subprocess re-spawn")?;

    let mut args: Vec<String> = vec!["fleet".into(), "doctor".into(), "--json".into()];
    args.push("--timeout".into());
    args.push(timeout_secs.to_string());
    if legacy_usage {
        args.push("--legacy-usage".into());
        args.push("--legacy-window-days".into());
        args.push(legacy_window_days.to_string());
    }
    if topic_durability {
        args.push("--topic-durability".into());
    }
    if include_pin_check {
        args.push("--include-pin-check".into());
    }
    args.push("--top-callers".into());
    args.push(top_callers.to_string());

    let mut prior: std::collections::BTreeMap<String, WatchHubState> =
        std::collections::BTreeMap::new();
    let mut cycle: u32 = 0;

    eprintln!(
        "{} watch: polling every {}s (include-pin-check={}, legacy-usage={}); ctrl-c to stop",
        crate::manifest::now_rfc3339(),
        secs,
        include_pin_check,
        legacy_usage
    );

    loop {
        let one_cycle = tokio::process::Command::new(&exe).args(&args).output();
        let output = tokio::select! {
            r = one_cycle => r.context("--watch: subprocess spawn failed")?,
            _ = tokio::signal::ctrl_c() => {
                println!("{} watch stopped (sigint, completed {} cycle(s))", crate::manifest::now_rfc3339(), cycle);
                return Ok(());
            }
        };

        let ts = crate::manifest::now_rfc3339();
        let json_doc: serde_json::Value = match serde_json::from_slice(&output.stdout) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "{} watch: failed to parse subprocess JSON ({}): exit={:?}",
                    ts,
                    e,
                    output.status.code()
                );
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => continue,
                    _ = tokio::signal::ctrl_c() => {
                        println!("{} watch stopped (sigint, completed {} cycle(s))", crate::manifest::now_rfc3339(), cycle);
                        return Ok(());
                    }
                }
            }
        };

        let mut current: std::collections::BTreeMap<String, WatchHubState> =
            std::collections::BTreeMap::new();
        if let Some(hubs) = json_doc.get("hubs").and_then(|v| v.as_array()) {
            for hub in hubs {
                let name = hub
                    .get("hub")
                    .and_then(|s| s.as_str())
                    .unwrap_or("?")
                    .to_string();
                // T-1682: bridge JSON status vocabulary (ok/error/timeout)
                // into watch's in-memory conn vocabulary (which T-1681's
                // gate compares against "auth-mismatch"). Extracted for
                // unit-testability — see derive_watch_conn.
                let conn = derive_watch_conn(hub);
                let pin = hub
                    .get("pin_check")
                    .and_then(|p| p.get("status"))
                    .and_then(|s| s.as_str())
                    .map(String::from);
                let legacy = hub
                    .get("legacy_usage")
                    .and_then(|l| l.get("total_legacy"))
                    .and_then(|n| n.as_u64());
                current.insert(name, (conn, pin, legacy));
            }
        }

        cycle += 1;
        if cycle == 1 {
            println!("{} baseline: {} hub(s)", ts, current.len());
            for (name, (conn, pin, legacy)) in &current {
                let pin_str = pin.as_deref().unwrap_or("-");
                let legacy_str = legacy
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "-".into());
                println!(
                    "{}   {} conn={} pin={} legacy={}",
                    ts, name, conn, pin_str, legacy_str
                );
            }
        } else {
            let mut changes = 0u32;
            for (name, new_state) in &current {
                let old = prior.get(name);
                if old != Some(new_state) {
                    let pin_str = new_state.1.as_deref().unwrap_or("-");
                    let legacy_str = new_state
                        .2
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".into());
                    if let Some(o) = old {
                        let old_pin = o.1.as_deref().unwrap_or("-");
                        let old_legacy =
                            o.2.map(|n| n.to_string()).unwrap_or_else(|| "-".into());
                        println!(
                            "{}   {} conn={}→{} pin={}→{} legacy={}→{}",
                            ts, name, o.0, new_state.0, old_pin, pin_str, old_legacy, legacy_str
                        );
                        append_rotation_log(
                            name, "transition",
                            &o.0, &new_state.0,
                            old_pin, pin_str,
                            &old_legacy, &legacy_str,
                        );
                        if let Some(cmd) = notify.as_deref() {
                            fire_notify(
                                cmd, name, "transition",
                                &o.0, &new_state.0,
                                old_pin, pin_str,
                                &old_legacy, &legacy_str,
                                &ts,
                            );
                        }
                        // T-1680 + T-1681: built-in auto-heal on rotation.
                        // Fires on either of two transitions in this cycle:
                        //   (a) cert rotation:   new_pin == "drift"        (T-1680)
                        //   (b) secret-only:     new_conn == "auth-mismatch" (T-1681, PL-162)
                        // Same heal action serves both — bootstrap-from
                        // fetches the new secret, which the auth-mismatch
                        // path needs and the drift path also benefits from
                        // (PL-021's "BOTH rotate" case trips both gates;
                        // dedup below ensures only one heal fires per cycle).
                        if auto_heal {
                            let pin_drift_now =
                                pin_str == "drift" && old_pin != "drift";
                            let auth_mismatch_now = new_state.0 == "auth-mismatch"
                                && o.0 != "auth-mismatch";
                            if pin_drift_now || auth_mismatch_now {
                                // T-1685: prefer cert-drift trigger when
                                // both fire (PL-021's "BOTH rotate" case).
                                let trigger = if pin_drift_now { "cert-drift" } else { "auth-mismatch" };
                                let anchor = crate::config::load_hubs_config()
                                    .hubs
                                    .get(name)
                                    .and_then(|e| e.bootstrap_from.as_deref())
                                    .map(String::from);
                                if let Some(bootstrap) = anchor.as_deref() {
                                    if dry_run {
                                        // T-1684: watch + dry-run.
                                        eprintln!(
                                            "{ts} [DRY-RUN] would fire: termlink fleet reauth {} --bootstrap-from auto",
                                            name
                                        );
                                        append_heal_log(name, "watch", trigger, "dry-run", Some(bootstrap));
                                    } else {
                                        fire_auto_heal(name, &ts);
                                        append_heal_log(name, "watch", trigger, "fired", Some(bootstrap));
                                    }
                                } else {
                                    eprintln!(
                                        "{ts} watch: --auto-heal skipped hub={name}: no bootstrap_from declared (R2 — declare it to enable auto-heal)"
                                    );
                                    append_heal_log(name, "watch", trigger, "skipped-no-anchor", None);
                                }
                            }
                        }
                    } else {
                        println!(
                            "{}   {} NEW conn={} pin={} legacy={}",
                            ts, name, new_state.0, pin_str, legacy_str
                        );
                        append_rotation_log(
                            name, "new",
                            "", &new_state.0,
                            "-", pin_str,
                            "-", &legacy_str,
                        );
                        if let Some(cmd) = notify.as_deref() {
                            fire_notify(
                                cmd, name, "new",
                                "", &new_state.0,
                                "-", pin_str,
                                "-", &legacy_str,
                                &ts,
                            );
                        }
                    }
                    changes += 1;
                }
            }
            for (name, old_state) in &prior {
                if !current.contains_key(name) {
                    println!(
                        "{}   {} REMOVED (was conn={})",
                        ts, name, old_state.0
                    );
                    let old_pin = old_state.1.as_deref().unwrap_or("-");
                    let old_legacy = old_state
                        .2
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".into());
                    append_rotation_log(
                        name, "removed",
                        &old_state.0, "",
                        old_pin, "-",
                        &old_legacy, "-",
                    );
                    if let Some(cmd) = notify.as_deref() {
                        fire_notify(
                            cmd, name, "removed",
                            &old_state.0, "",
                            old_pin, "-",
                            &old_legacy, "-",
                            &ts,
                        );
                    }
                    changes += 1;
                }
            }
            if changes == 0 && cycle.is_multiple_of(10) {
                eprintln!("{} watch: cycle {} (no changes)", ts, cycle);
            }
        }

        prior = current;

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => {},
            _ = tokio::signal::ctrl_c() => {
                println!("{} watch stopped (sigint, completed {} cycle(s))", crate::manifest::now_rfc3339(), cycle);
                return Ok(());
            }
        }
    }
}

pub(crate) async fn cmd_fleet_doctor(
    json: bool,
    timeout_secs: u64,
    legacy_usage: bool,
    legacy_window_days: u64,
    topic_durability: bool,
    include_pin_check: bool,
    diff: Option<std::path::PathBuf>,
    save_snapshot: Option<std::path::PathBuf>,
    exit_code_on_verdict: bool,
    trend: Option<std::path::PathBuf>,
    trend_keep: u32,
    top_callers: u32,
    watch: Option<u64>,
    notify: Option<String>,
    auto_heal: bool,
    dry_run: bool,
) -> Result<()> {
    // T-1669: --notify is meaningless without --watch (single-shot has no diff
    // cycles). Reject loudly so the operator sees the misuse immediately.
    if notify.is_some() && watch.is_none() {
        anyhow::bail!("--notify requires --watch (operator-hook fires on cycle-to-cycle state diffs)");
    }
    // T-1683: `--auto-heal` is supported both with and without `--watch`.
    // With `--watch`: heal fires on per-cycle state transitions (T-1680/T-1681).
    // Without `--watch`: heal fires on current state at end of single sweep
    // (this function, post-loop). Hint when pin-check is missing — without it,
    // only the auth-mismatch path can fire, so a primary use-case (cert drift)
    // is silently disabled.
    if auto_heal && watch.is_none() && !include_pin_check {
        eprintln!(
            "[info] --auto-heal without --include-pin-check: only conn=auth-mismatch heals will fire. \
             Pass --include-pin-check to also heal on cert drift."
        );
    }
    // T-1667: --watch dispatches to the continuous-monitoring loop. The loop
    // re-spawns self via std::env::current_exe() with --json each cycle and
    // emits per-hub state diffs. Reject single-shot-only companion flags here
    // (--diff / --save-snapshot / --exit-code-on-verdict / --trend) so the
    // operator learns immediately, not after the fleet sweep.
    if let Some(secs) = watch {
        if diff.is_some() || save_snapshot.is_some() || exit_code_on_verdict || trend.is_some() {
            anyhow::bail!("--watch is incompatible with --diff, --save-snapshot, --exit-code-on-verdict, --trend (single-shot semantics)");
        }
        return cmd_fleet_doctor_watch(
            secs, timeout_secs, legacy_usage, legacy_window_days,
            topic_durability, include_pin_check, top_callers, notify, auto_heal, dry_run,
        ).await;
    }

    // T-1471: clamp top-callers count to 1..=50. 50 ceiling guards against
    // pathological output (e.g. 10K-distinct-caller hub).
    let top_callers = top_callers.clamp(1, 50) as usize;
    // T-1462: --diff requires --legacy-usage. Reject loudly so operators don't
    // wonder why nothing happens.
    if diff.is_some() && !legacy_usage {
        anyhow::bail!("--diff requires --legacy-usage (the diff is computed against the legacy_summary block)");
    }
    // T-1465: same precondition — verdict mapping requires the verdict.
    if exit_code_on_verdict && !legacy_usage {
        anyhow::bail!("--exit-code-on-verdict requires --legacy-usage (no verdict to map without it)");
    }
    // T-1468: --trend reads N snapshots and assembles a time-series of the
    // legacy_summary.total_legacy_fleet field — no point without --legacy-usage.
    if trend.is_some() && !legacy_usage {
        anyhow::bail!("--trend requires --legacy-usage (the trend is built from legacy_summary blocks across snapshots)");
    }
    // T-1463: validate save-snapshot parent directory exists *before* doing
    // an entire fleet sweep. Failing fast saves operator time.
    if let Some(p) = save_snapshot.as_deref() {
        let parent = p.parent().filter(|s| !s.as_os_str().is_empty());
        if let Some(dir) = parent
            && !dir.exists()
        {
            anyhow::bail!(
                "--save-snapshot: parent directory {} does not exist (create it first)",
                dir.display()
            );
        }
    }
    // T-1462: read prior snapshot up-front so we fail fast on missing/unparseable
    // file rather than after a full RPC sweep. Returns the *whole* prior fleet
    // doctor JSON document; we'll extract `legacy_summary` and `_snapshot_ts_ms`
    // when computing the diff.
    let prior_snapshot: Option<serde_json::Value> = if let Some(path) = diff.as_deref() {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("--diff: cannot read snapshot file {}", path.display()))?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .with_context(|| format!("--diff: snapshot file {} is not valid JSON", path.display()))?;
        if v.get("legacy_summary").is_none() {
            anyhow::bail!(
                "--diff: snapshot file {} has no `legacy_summary` field — was it produced by `fleet doctor --legacy-usage --json`?",
                path.display()
            );
        }
        Some(v)
    } else {
        None
    };
    // T-1468: read trend snapshots up-front. Sort by filename (chronological under
    // the cron convention `YYYY-MM-DD.json`), keep the N most recent (default 7,
    // capped at 30). Each must be a valid fleet-doctor JSON doc with a
    // `legacy_summary` field; malformed files emit a warning to stderr but do
    // not abort the run (the operator already has good current data — bailing
    // on a stale file is anti-helpful).
    let trend_snapshots: Vec<(String, serde_json::Value)> = if let Some(dir) = trend.as_deref() {
        if !dir.is_dir() {
            anyhow::bail!("--trend: {} is not a directory", dir.display());
        }
        let keep = trend_keep.clamp(1, 30) as usize;
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
            .with_context(|| format!("--trend: cannot read directory {}", dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect();
        files.sort();
        let files = files.into_iter().rev().take(keep).collect::<Vec<_>>();
        // Reverse back to chronological (oldest → newest) for the time-series.
        let mut files: Vec<std::path::PathBuf> = files.into_iter().collect();
        files.reverse();
        let mut out: Vec<(String, serde_json::Value)> = Vec::new();
        for p in files {
            let label = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
            let Ok(text) = std::fs::read_to_string(&p) else {
                eprintln!("--trend: cannot read {} (skipped)", p.display());
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
                eprintln!("--trend: {} is not valid JSON (skipped)", p.display());
                continue;
            };
            if v.get("legacy_summary").is_none() {
                eprintln!("--trend: {} has no legacy_summary block (skipped)", p.display());
                continue;
            }
            out.push((label, v));
        }
        out
    } else {
        Vec::new()
    };
    // T-1432: clamp window to documented range. 1 day floor (avoid empty
    // windows from 0), 90 day ceiling (matches T-1166 audit-log retention
    // assumption — older lines may have been pruned).
    let legacy_window_days = legacy_window_days.clamp(1, 90);
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::json!({"ok": true, "hubs": [], "message": "No hubs configured in ~/.termlink/hubs.toml"}));
        } else {
            eprintln!("No hubs configured in ~/.termlink/hubs.toml");
        }
        return Ok(());
    }

    // T-1616: surface CLI version in header so operator can immediately
    // see skew between CLI and hubs (avoids cross-referencing `termlink info`).
    let cli_version = env!("CARGO_PKG_VERSION");
    let cli_commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    if !json {
        eprintln!(
            "Fleet doctor: {} hub(s) configured (CLI {} [{}])\n",
            config.hubs.len(), cli_version, cli_commit
        );
    }

    let mut hub_results: Vec<serde_json::Value> = Vec::new();
    let mut total_pass: u32 = 0;
    // T-1615: was hard-coded to 0 — summary footer reported `0 warn` regardless
    // of how many `[WARN]` lines fired in the body. Now incremented at each
    // WARN-emit site (currently: stale-version detection in PASS branch).
    let mut total_warn: u32 = 0;
    let mut total_fail: u32 = 0;

    // T-1639: surface local outbound-queue health BEFORE per-hub probes.
    // Sender-side stall (BusClient buffering posts that the hub never accepted)
    // was previously invisible to fleet doctor — operators only saw stalls
    // once the destination side noticed. Origin: framework-agent T-1827
    // offset-14 follow-up after the offset-9/10/12 pickup-channel stall.
    let queue_status_obj: serde_json::Value = {
        use termlink_session::offline_queue::{default_queue_path, OfflineQueue};
        let qpath = default_queue_path();
        if !qpath.exists() {
            if !json {
                eprintln!("Outbound queue: 0 pending (no queue file)\n");
            }
            serde_json::json!({
                "queue_path": qpath.display().to_string(),
                "exists": false,
                "pending": 0,
                "oldest_age_secs": 0,
                "warn": false,
            })
        } else {
            match OfflineQueue::open(&qpath) {
                Ok(queue) => {
                    let pending = queue.size().unwrap_or(0);
                    let head = queue.peek_oldest().ok().flatten();
                    let now_ms: i64 = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    let oldest_age_secs: i64 = head
                        .as_ref()
                        .map(|(_, post)| {
                            (now_ms.saturating_sub(post.ts_unix_ms)).max(0) / 1000
                        })
                        .unwrap_or(0);
                    let warn = pending > 0 && oldest_age_secs > 300;
                    if warn {
                        total_warn += 1;
                    }
                    if !json {
                        if pending == 0 {
                            eprintln!("Outbound queue: 0 pending");
                        } else {
                            let topic = head
                                .as_ref()
                                .map(|(_, p)| p.topic.as_str())
                                .unwrap_or("(unknown)");
                            let tag = if warn { "[WARN] " } else { "" };
                            eprintln!(
                                "Outbound queue: {}{} pending, oldest topic={} age={}s",
                                tag, pending, topic, oldest_age_secs
                            );
                            if warn {
                                eprintln!(
                                    "  hint: sender-side stall — local hub may have rejected posts; run `termlink channel queue-status` for head detail"
                                );
                            }
                        }
                        eprintln!();
                    }
                    serde_json::json!({
                        "queue_path": qpath.display().to_string(),
                        "exists": true,
                        "pending": pending,
                        "oldest_age_secs": oldest_age_secs,
                        "oldest_topic": head.as_ref().map(|(_, p)| p.topic.clone()),
                        "warn": warn,
                    })
                }
                Err(e) => {
                    if !json {
                        eprintln!("Outbound queue: read error ({})\n", e);
                    }
                    serde_json::json!({
                        "queue_path": qpath.display().to_string(),
                        "exists": true,
                        "pending": 0,
                        "oldest_age_secs": 0,
                        "warn": false,
                        "error": e.to_string(),
                    })
                }
            }
        }
    };
    // T-1132: aggregate binary versions across the fleet. `unknown` covers hubs
    // that failed to connect or that pre-date the hub.version RPC.
    let mut fleet_versions: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();

    // Sort hub names for deterministic output
    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();

    // T-1666: when --include-pin-check is set, run TLS probes for every hub in
    // parallel BEFORE the per-hub diagnostic loop, then inject results into each
    // hub_obj as we build them. Probing here (rather than serially inside the
    // loop) keeps total wall time bounded by the slowest probe rather than
    // summing them. Reuses the same `probe_cert` + `KnownHubStore` primitives
    // as `fleet verify` (T-1660) — verdict semantics MUST match so the two
    // commands agree on rotation state.
    // Tuple shape: (status, wire, pinned, error).
    type PinCheck = (&'static str, Option<String>, Option<String>, Option<String>);
    let pin_checks: std::collections::HashMap<String, PinCheck> = if include_pin_check {
        let store = termlink_session::tofu::KnownHubStore::default_store();
        // T-1674/T-1675: bound each probe by `timeout_secs` via the
        // centralized `probe_cert_with_timeout` primitive. Without the bound,
        // an unreachable hub holds its tokio::spawn task open for the OS TCP
        // retry budget (30-60+s) and determines slowest-probe latency.
        let probe_timeout = std::time::Duration::from_secs(timeout_secs);
        let probes: Vec<_> = hub_names.iter().map(|name| {
            let address = config.hubs[*name].address.clone();
            tokio::spawn(async move {
                let result = termlink_session::tofu::probe_cert_with_timeout(
                    &address, probe_timeout,
                ).await;
                (address, result)
            })
        }).collect();
        let mut out = std::collections::HashMap::with_capacity(probes.len());
        for handle in probes {
            if let Ok((address, probe_result)) = handle.await {
                let pinned = store.get(&address);
                let entry: PinCheck = match probe_result {
                    Ok((_, wire)) => match &pinned {
                        Some(pin) if pin == &wire => ("match", Some(wire), pinned.clone(), None),
                        Some(_) => ("drift", Some(wire), pinned.clone(), None),
                        None => ("no-pin", Some(wire), None, None),
                    },
                    Err(e) => ("probe-fail", None, pinned.clone(), Some(e)),
                };
                out.insert(address, entry);
            }
        }
        out
    } else {
        std::collections::HashMap::new()
    };

    for name in hub_names {
        let entry = &config.hubs[name];

        if !json {
            eprintln!("--- {} ({}) ---", name, entry.address);
        }

        // Quick connectivity check via connect_remote_hub
        let connect_start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let result = tokio::time::timeout(
            timeout_dur,
            connect_remote_hub(
                &entry.address,
                entry.secret_file.as_deref(),
                entry.secret.as_deref(),
                entry.scope.as_deref().unwrap_or("execute"),
            ),
        ).await;

        // T-1034: Resolve secret source for diagnostics
        let secret_source = entry.secret_file.as_deref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| {
                if entry.secret.is_some() { "inline secret".to_string() }
                else { "none".to_string() }
            });

        // T-1652: compute perms warning once per hub. Independent of probe
        // success — even an UNREACHABLE hub with a world-readable secret_file
        // is leaking the HMAC every time `ls` runs in that home directory.
        let secret_perms_warning: Option<String> = entry
            .secret_file
            .as_deref()
            .and_then(|p| secret_file_perms_warning(&expand_secret_file_path(p)));

        match result {
            Ok(Ok(mut client)) => {
                let latency = connect_start.elapsed().as_millis();
                total_pass += 1;

                // T-1132: probe hub.version; fall back to "unknown" for pre-T-1132 hubs.
                let hub_version = match client
                    .call(
                        "hub.version",
                        serde_json::json!("fleet-doctor-version"),
                        serde_json::json!({}),
                    )
                    .await
                {
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => r
                        .result
                        .get("hub_version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    _ => "unknown".to_string(),
                };
                *fleet_versions.entry(hub_version.clone()).or_insert(0) += 1;

                // T-1432: optional legacy-usage probe per hub. Pre-T-1432 hubs
                // return method-not-found and the per-hub summary records
                // "audit_unsupported" so the operator knows the hub needs an
                // upgrade before its cut-readiness can be measured.
                let legacy_summary = if legacy_usage {
                    match client
                        .call(
                            "hub.legacy_usage",
                            serde_json::json!("fleet-doctor-legacy"),
                            serde_json::json!({"window_seconds": legacy_window_days * 86400}),
                        )
                        .await
                    {
                        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => Some(r.result),
                        Ok(_) | Err(_) => Some(serde_json::json!({
                            "audit_unsupported": true,
                            "hint": "hub predates T-1432 (hub.legacy_usage) — upgrade to >=0.9.1640 to measure cut-readiness on this host",
                        })),
                    }
                } else {
                    None
                };

                // T-1446: optional bus_state probe per hub. Pre-T-1446 hubs
                // return method-not-found and the per-hub summary records
                // `audit_unsupported` so the operator knows the hub needs an
                // upgrade before its durability can be measured.
                let bus_state_summary = if topic_durability {
                    match client
                        .call(
                            "hub.bus_state",
                            serde_json::json!("fleet-doctor-bus-state"),
                            serde_json::json!({}),
                        )
                        .await
                    {
                        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => Some(r.result),
                        Ok(_) | Err(_) => Some(serde_json::json!({
                            "audit_unsupported": true,
                            "hint": "hub predates T-1446 (hub.bus_state) — upgrade to >=0.9.1717 to measure topic-durability on this host",
                        })),
                    }
                } else {
                    None
                };

                let mut hub_obj = serde_json::json!({
                    "hub": name,
                    "address": entry.address,
                    "status": "ok",
                    "latency_ms": latency,
                    "secret_source": &secret_source,
                    "hub_version": &hub_version,
                });
                // T-1652: surface the perms warning even on a healthy hub —
                // the leak risk is structural to the file, not to the probe.
                if let Some(w) = &secret_perms_warning
                    && let Some(obj) = hub_obj.as_object_mut()
                {
                    obj.insert("secret_perms_warning".to_string(), serde_json::Value::String(w.clone()));
                }
                if let Some(ls) = &legacy_summary
                    && let Some(obj) = hub_obj.as_object_mut()
                {
                    obj.insert("legacy_usage".to_string(), ls.clone());
                }
                if let Some(bs) = &bus_state_summary
                    && let Some(obj) = hub_obj.as_object_mut()
                {
                    obj.insert("bus_state".to_string(), bs.clone());
                }
                // T-1475 (T-1458 follow-up): warn when the hub reports the
                // workspace-static "0.9.0" — that means the running hub binary
                // was built BEFORE T-1458's build.rs landed (2026-05-03), so
                // its version string is the hardcoded workspace fallback rather
                // than the git-derived freshness signal. Operator can't tell
                // "I'm running latest" from "I haven't restarted in weeks".
                let version_stale = hub_version == "0.9.0";
                if version_stale {
                    // T-1615: count this as WARN regardless of json/text mode.
                    total_warn += 1;
                    if let Some(obj) = hub_obj.as_object_mut() {
                        obj.insert("version_stale".to_string(), serde_json::Value::Bool(true));
                    }
                }
                hub_results.push(hub_obj);
                if !json {
                    eprintln!("  [PASS] connected in {}ms (version: {})", latency, hub_version);
                    // T-1652: render perms warning under [PASS] so a clean
                    // connectivity result doesn't hide the standing security risk.
                    if let Some(w) = &secret_perms_warning {
                        eprintln!("    [WARN] {}", w);
                    }
                    if version_stale {
                        // T-1616: include CLI version so the WARN explicitly shows
                        // the skew (operator no longer needs to cross-reference info).
                        eprintln!(
                            "    [WARN] hub_version=0.9.0, cli_version={} — running hub binary \
                             predates T-1458 (2026-05-03 build.rs fix). Restart hub with the \
                             newer binary for accurate version reporting; this is a stale-build \
                             signal, not a connectivity issue.",
                            cli_version
                        );
                    }
                    // T-1446: render per-hub bus_state line when --topic-durability is set
                    if let Some(bs) = &bus_state_summary {
                        if bs.get("audit_unsupported").and_then(|v| v.as_bool()) == Some(true) {
                            eprintln!("    [bus_state] audit_unsupported (pre-T-1446 hub)");
                        } else {
                            let rd = bs.get("runtime_dir").and_then(|v| v.as_str()).unwrap_or("?");
                            let vol = bs.get("runtime_dir_volatile").and_then(|v| v.as_bool()).unwrap_or(false);
                            let present = bs.get("audit_present").and_then(|v| v.as_bool()).unwrap_or(false);
                            let size = bs.get("meta_db_size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                            let mtime = bs.get("meta_db_mtime_unix").and_then(|v| v.as_u64()).unwrap_or(0);
                            let verdict = if present && !vol { "DURABLE" } else if vol { "VOLATILE" } else { "MISSING" };
                            eprintln!(
                                "    [bus_state] {} runtime_dir={} meta_db={} bytes mtime={}",
                                verdict, rd, size, mtime
                            );
                        }
                    }
                }
                // T-1053: pass resets the auth-failure streak + re-arms concern gating.
                let _ = maybe_track_fleet_failure(name, &entry.address, None);
            }
            Ok(Err(e)) => {
                total_fail += 1;
                *fleet_versions.entry("unknown".into()).or_insert(0) += 1;
                // T-1181: use {:#} (anyhow alternate) so classify_fleet_error sees
                // the full cause chain. Default Display drops anyhow's .context()
                // wrappers, which was collapsing TOFU VIOLATION under "Cannot
                // connect — is the hub running?" and losing the actionable hint.
                let msg = format!("{:#}", e);
                let diagnostic = classify_fleet_error(&msg, &entry.address);
                let mut hub_obj = serde_json::json!({"hub": name, "address": entry.address, "status": "error", "error": &msg, "secret_source": &secret_source, "diagnostic": &diagnostic});
                if let Some(w) = &secret_perms_warning
                    && let Some(obj) = hub_obj.as_object_mut()
                {
                    obj.insert("secret_perms_warning".to_string(), serde_json::Value::String(w.clone()));
                }
                hub_results.push(hub_obj);
                if !json {
                    eprintln!("  [FAIL] {}", msg);
                    eprintln!("  secret: {}", secret_source);
                    eprintln!("  hint: {}", diagnostic);
                    if let Some(w) = &secret_perms_warning {
                        eprintln!("  [WARN] {}", w);
                    }
                }
                // T-1052: auto-register a learning for auth/TOFU failure classes so drift
                // is detectable by future agents (R1). Silent best-effort — never blocks.
                let _ = maybe_record_auth_mismatch_learning(name, &entry.address, &msg);
                // T-1053: track per-hub streak; register a concern after N failures >24h apart.
                let _ = maybe_track_fleet_failure(name, &entry.address, auth_mismatch_class(&msg));
            }
            Err(_) => {
                total_fail += 1;
                *fleet_versions.entry("unknown".into()).or_insert(0) += 1;
                let diagnostic = "Check network connectivity and that hub is listening on the configured port";
                let mut hub_obj = serde_json::json!({"hub": name, "address": entry.address, "status": "timeout", "secret_source": &secret_source, "diagnostic": diagnostic});
                if let Some(w) = &secret_perms_warning
                    && let Some(obj) = hub_obj.as_object_mut()
                {
                    obj.insert("secret_perms_warning".to_string(), serde_json::Value::String(w.clone()));
                }
                hub_results.push(hub_obj);
                if !json {
                    eprintln!("  [FAIL] Timeout after {}s", timeout_secs);
                    eprintln!("  hint: {}", diagnostic);
                    if let Some(w) = &secret_perms_warning {
                        eprintln!("  [WARN] {}", w);
                    }
                }
                // T-1053: timeouts aren't auth-class failures → reset streak.
                let _ = maybe_track_fleet_failure(name, &entry.address, None);
            }
        }

        if !json {
            eprintln!();
        }
    }

    // T-1666: inject pin_check into each hub_obj (post-loop, after all hub_results
    // are built). pin_checks HashMap was populated up-front via parallel TLS probes;
    // here we just look up by address and attach as a nested object. Pin-drift lines
    // also emit to stderr in plain mode so the operator sees them grouped with the
    // hub's other diagnostics (each per-hub block printed `--- name (address) ---`
    // header during the loop; we deliberately separate pin-check output into a
    // post-loop footer to keep the existing single-pass plain output stable).
    let pin_check_summary: Option<serde_json::Value> = if include_pin_check {
        let mut drift_count = 0u32;
        let mut no_pin_count = 0u32;
        let mut probe_fail_count = 0u32;
        let mut match_count = 0u32;
        let mut drift_hubs: Vec<(String, String, String)> = Vec::new();  // (name, pinned, wire)
        for hub_obj in hub_results.iter_mut() {
            let addr = hub_obj.get("address").and_then(|v| v.as_str()).map(String::from);
            let hub_name = hub_obj.get("hub").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            if let Some(addr) = addr
                && let Some((status, wire, pinned, error)) = pin_checks.get(&addr)
            {
                let pin_obj = serde_json::json!({
                    "status": status,
                    "wire": wire,
                    "pinned": pinned,
                    "error": error,
                });
                match *status {
                    "match" => match_count += 1,
                    "drift" => {
                        drift_count += 1;
                        if let (Some(p), Some(w)) = (pinned.as_ref(), wire.as_ref()) {
                            drift_hubs.push((hub_name.clone(), p.clone(), w.clone()));
                        }
                    },
                    "no-pin" => no_pin_count += 1,
                    "probe-fail" => probe_fail_count += 1,
                    _ => {}
                }
                if let Some(obj) = hub_obj.as_object_mut() {
                    obj.insert("pin_check".to_string(), pin_obj);
                }
            }
        }
        let verdict = if drift_count > 0 { "drift" }
            else if probe_fail_count > 0 { "probe-fail" }
            else if no_pin_count > 0 { "no-pin" }
            else { "match" };

        if !json {
            eprintln!("Pin check: {} (match={}, drift={}, no-pin={}, probe-fail={})",
                verdict, match_count, drift_count, no_pin_count, probe_fail_count);
            for (name, pinned, wire) in &drift_hubs {
                let short = |s: &str| s.chars().skip(7).take(12).collect::<String>();  // "sha256:" + 12 chars
                eprintln!("  [DRIFT] {}: pin={} wire={}", name, short(pinned), short(wire));
            }
            if !drift_hubs.is_empty() {
                eprintln!("  Heal: termlink fleet reauth <profile> --bootstrap-from auto");
                eprintln!("  Re-pin: termlink tofu clear <address>");
            }
            eprintln!();
        }

        Some(serde_json::json!({
            "verdict": verdict,
            "match_count": match_count,
            "drift_count": drift_count,
            "no_pin_count": no_pin_count,
            "probe_fail_count": probe_fail_count,
        }))
    } else {
        None
    };

    // T-1683: single-shot --auto-heal pass.
    //
    // This runs only in the non-watch single-shot path. The --watch path
    // dispatches to cmd_fleet_doctor_watch earlier and handles auto-heal
    // per-transition there (T-1680/T-1681). Here we classify each hub's
    // CURRENT state from hub_results and fire the same heal for any
    // profile that's in drift or auth-mismatch AND has declared
    // bootstrap_from. Same R2 gate, same fire-and-forget semantics.
    //
    // Operator value: page-respond ("doctor flagged drift, fix it") without
    // starting a watch loop. One command, one heal per affected hub.
    if auto_heal {
        let ts = crate::manifest::now_rfc3339();
        let hubs_config = crate::config::load_hubs_config();
        let mut acted = 0u32;
        let mut skipped_no_anchor: Vec<String> = Vec::new();
        for hub_obj in &hub_results {
            let name = hub_obj.get("hub").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            // Derive effective conn class — same vocabulary bridge as the
            // watch parser (T-1682). status=error + auth class → "auth-mismatch".
            let conn = derive_watch_conn(hub_obj);
            let pin = hub_obj
                .get("pin_check")
                .and_then(|p| p.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("-");
            let cert_drift = pin == "drift";
            let auth_mismatch = conn == "auth-mismatch";
            if !(cert_drift || auth_mismatch) {
                continue;
            }
            // T-1685: trigger for the audit line. If both fired (PL-021's
            // "both rotate" case), prefer cert-drift since that's the more
            // commonly visible failure mode in practice.
            let trigger = if cert_drift { "cert-drift" } else { "auth-mismatch" };
            let anchor = hubs_config
                .hubs
                .get(&name)
                .and_then(|e| e.bootstrap_from.as_deref());
            if let Some(bootstrap) = anchor {
                if dry_run {
                    // T-1684: dry-run — describe the intended fire without
                    // spawning anything. Keep the line format stable so
                    // operators can grep / pipe to a reviewer.
                    eprintln!(
                        "[DRY-RUN] would fire: termlink fleet reauth {} --bootstrap-from auto",
                        name
                    );
                    append_heal_log(&name, "one-shot", trigger, "dry-run", Some(bootstrap));
                } else {
                    fire_auto_heal(&name, &ts);
                    append_heal_log(&name, "one-shot", trigger, "fired", Some(bootstrap));
                }
                acted += 1;
            } else {
                append_heal_log(&name, "one-shot", trigger, "skipped-no-anchor", None);
                skipped_no_anchor.push(name);
            }
        }
        if !json && (acted > 0 || !skipped_no_anchor.is_empty()) {
            eprintln!();
            if dry_run {
                eprintln!("Auto-heal: would fire {} (dry-run, T-1684)", acted);
            } else {
                eprintln!("Auto-heal: fired {} (one-shot, T-1683)", acted);
            }
            for name in &skipped_no_anchor {
                eprintln!(
                    "  [SKIP] {}: no bootstrap_from declared (R2 — declare it to enable auto-heal)",
                    name
                );
            }
        }
    }

    // T-1432: aggregate cut-readiness verdict from per-hub legacy_usage payloads.
    // T-1459: split the binary CUT-READY/WAIT into three traffic states so the
    // top-line answers the operator's actual question ("are there live callers?")
    // rather than forcing them to read per-hub last_call_age tags.
    // Verdict semantics:
    //   CUT-READY          — all reachable hubs reported audit_present=true AND total_legacy=0
    //   CUT-READY-DECAYING — total_legacy > 0 but no hub has had a call within
    //                        ACTIVE_TRAFFIC_THRESHOLD_SECS (5 min). The audit window
    //                        will clear naturally; operator may cut now (residue is
    //                        historical) or wait for the window to age out.
    //   WAIT               — at least one hub has had a legacy call in the last 5 min
    //                        (live caller still polling; cut would break it).
    //   UNCERTAIN          — at least one hub returned audit_unsupported (pre-T-1432)
    //                        OR audit_present=false (no traffic recorded yet — fresh runtime_dir)
    //                        — operator must upgrade or wait for traffic before deciding.
    let legacy_summary_obj = if legacy_usage {
        let mut total_legacy_fleet: u64 = 0;
        let mut hubs_unsupported: Vec<String> = Vec::new();
        let mut hubs_no_audit: Vec<String> = Vec::new();
        let mut hubs_with_traffic: Vec<(String, u64, u128)> = Vec::new();
        let mut hubs_clean: Vec<String> = Vec::new();
        // T-1460: per-hub top-callers list, keyed by hub name. Populated from
        // the hub's `top_callers` field when present (post-T-1460 hubs);
        // older hubs leave this empty and the CLI falls back to the existing
        // method/count line silently.
        let mut hub_top_callers: std::collections::BTreeMap<String, Vec<(String, u64)>> =
            std::collections::BTreeMap::new();
        for h in &hub_results {
            let name = h.get("hub").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let Some(lu) = h.get("legacy_usage") else { continue };
            if lu.get("audit_unsupported").and_then(|v| v.as_bool()) == Some(true) {
                hubs_unsupported.push(name);
                continue;
            }
            if lu.get("audit_present").and_then(|v| v.as_bool()) == Some(false) {
                hubs_no_audit.push(name);
                continue;
            }
            let count = lu.get("total_legacy").and_then(|v| v.as_u64()).unwrap_or(0);
            total_legacy_fleet += count;
            if count > 0 {
                let last_ts = lu
                    .get("last_legacy_ts_ms")
                    .and_then(|v| v.as_u64())
                    .map(|t| t as u128)
                    .unwrap_or(0);
                // T-1460: surface top callers if hub provides them.
                let mut parsed: Vec<(String, u64)> = Vec::new();
                if let Some(arr) = lu.get("top_callers").and_then(|v| v.as_array()) {
                    parsed = arr
                        .iter()
                        .filter_map(|c| {
                            let id = c.get("id").and_then(|v| v.as_str())?.to_string();
                            let cnt = c.get("count").and_then(|v| v.as_u64())?;
                            Some((id, cnt))
                        })
                        .collect();
                }
                // T-1467: fallback for pre-T-1460 hubs (no native `top_callers`
                // field). The same caller→count data lives inside `by_method`,
                // just keyed by `from` instead of `id`. Derive it on the
                // client side so the fleet aggregator sees something.
                if parsed.is_empty() {
                    if let Some(by_method) = lu.get("by_method") {
                        parsed = derive_top_callers_from_by_method(by_method);
                    }
                }
                if !parsed.is_empty() {
                    hub_top_callers.insert(name.clone(), parsed);
                }
                hubs_with_traffic.push((name, count, last_ts));
            } else {
                hubs_clean.push(name);
            }
        }
        let now_ms_for_verdict: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let verdict = compute_cut_readiness_verdict(
            &hubs_with_traffic,
            &hubs_unsupported,
            &hubs_no_audit,
            &hubs_clean,
            now_ms_for_verdict,
        );
        if !json {
            eprintln!();
            eprintln!("=== T-1166 cut-readiness ({}d window) ===", legacy_window_days);
            eprintln!("Verdict: {}", verdict);
            eprintln!("  total legacy invocations across fleet: {}", total_legacy_fleet);
            if !hubs_clean.is_empty() {
                eprintln!("  CLEAN ({}d): {}", legacy_window_days, hubs_clean.join(", "));
            }
            if !hubs_with_traffic.is_empty() {
                eprintln!("  WITH TRAFFIC:");
                let now_ms: u128 = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                for (name, count, last_ts) in &hubs_with_traffic {
                    let suffix = if *last_ts > 0 && now_ms > *last_ts {
                        let age_s = (now_ms - *last_ts) / 1000;
                        let age_human = if age_s < 60 {
                            format!("{age_s}s ago")
                        } else if age_s < 3600 {
                            format!("{}m ago", age_s / 60)
                        } else if age_s < 86400 {
                            format!("{}h ago", age_s / 3600)
                        } else {
                            format!("{}d ago", age_s / 86400)
                        };
                        let tag = if age_s < ACTIVE_TRAFFIC_THRESHOLD_SECS as u128 { "ACTIVE" } else { "decay residue" };
                        format!(" — last call {age_human} ({tag})")
                    } else {
                        String::new()
                    };
                    eprintln!("    {name}: {count} legacy invocation(s){suffix}");
                    // T-1460/T-1471: surface top-N callers if hub returned them.
                    // T-1476: annotate (unknown) once per hub block as pre-T-1409 residue.
                    let mut hub_seen_unknown = false;
                    if let Some(callers) = hub_top_callers.get(name) {
                        for (id, c) in callers.iter().take(top_callers) {
                            eprintln!("      └─ {c}× {id}");
                            if id == "(unknown)" {
                                hub_seen_unknown = true;
                            }
                        }
                    }
                    if hub_seen_unknown {
                        eprintln!(
                            "      └─ note: (unknown) entries are pre-T-1409 attribution-gap \
                             residue; the gap was closed 2026-04-29 (peer_addr/peer_pid threaded \
                             into rpc_audit). These historical counts cannot be acted on \
                             individually — track recent traffic via 'ACTIVE' tag instead."
                        );
                    }
                }
            }
            // T-1461/T-1471: fleet-wide top-N callers aggregate (single-line
            // answer to "who's producing the residue?" instead of N repeated
            // lines).
            let fleet_top = aggregate_fleet_top_callers(&hub_top_callers);
            let mut fleet_seen_unknown = false;
            if !fleet_top.is_empty() {
                eprintln!("  Top callers (fleet-wide):");
                for (id, c) in fleet_top.iter().take(top_callers) {
                    eprintln!("    {c}× {id}");
                    if id == "(unknown)" {
                        fleet_seen_unknown = true;
                    }
                }
            }
            // T-1476: same annotation, once for the fleet-wide aggregate.
            if fleet_seen_unknown {
                eprintln!(
                    "    note: (unknown) entries are pre-T-1409 attribution-gap residue; the gap \
                     was closed 2026-04-29 (peer_addr/peer_pid threaded into rpc_audit). \
                     These historical counts cannot be acted on individually — track recent \
                     traffic via 'ACTIVE' tag instead."
                );
            }
            if !hubs_unsupported.is_empty() {
                eprintln!(
                    "  UNSUPPORTED (pre-T-1432, upgrade to measure): {}",
                    hubs_unsupported.join(", ")
                );
            }
            if !hubs_no_audit.is_empty() {
                eprintln!(
                    "  NO AUDIT FILE (fresh runtime_dir or hub never received traffic): {}",
                    hubs_no_audit.join(", ")
                );
            }
            match verdict {
                "CUT-READY" => {
                    eprintln!("  → no live legacy callers (T-1166 cut already landed in T-1415; verdict is informational).");
                }
                "CUT-READY-DECAYING" => {
                    eprintln!(
                        "  → no live legacy callers (no traffic in last {}s); residue is historical.",
                        ACTIVE_TRAFFIC_THRESHOLD_SECS
                    );
                    eprintln!("  → operator may cut now or wait for the audit window to clear naturally.");
                }
                _ => {}
            }
        }
        // T-1461: include fleet-wide top callers aggregate in JSON.
        let fleet_top_callers_json = aggregate_fleet_top_callers(&hub_top_callers);
        Some(serde_json::json!({
            "window_days": legacy_window_days,
            "verdict": verdict,
            "total_legacy_fleet": total_legacy_fleet,
            "hubs_clean": hubs_clean,
            "hubs_with_traffic": hubs_with_traffic.iter().map(|(n, c, t)| serde_json::json!({"hub": n, "count": c, "last_ts_ms": *t as u64})).collect::<Vec<_>>(),
            "hubs_unsupported": hubs_unsupported,
            "hubs_no_audit": hubs_no_audit,
            "top_callers_fleet": fleet_top_callers_json.iter().map(|(id, c)| serde_json::json!({"id": id, "count": c})).collect::<Vec<_>>(),
        }))
    } else {
        None
    };

    // T-1446: aggregate G-050 audit-sweep verdict from per-hub bus_state payloads.
    //   DURABLE   — every reachable hub reports audit_present=true AND runtime_dir_volatile=false
    //   VOLATILE  — at least one hub has runtime_dir_volatile=true (e.g. /tmp/termlink-0)
    //   UNCERTAIN — at least one hub returned audit_unsupported (pre-T-1446)
    //               OR audit_present=false (fresh runtime_dir, no posts yet)
    let bus_state_summary_obj = if topic_durability {
        let mut hubs_durable: Vec<String> = Vec::new();
        let mut hubs_volatile: Vec<(String, String)> = Vec::new(); // (hub, runtime_dir)
        let mut hubs_missing: Vec<(String, String)> = Vec::new();  // (hub, runtime_dir)
        let mut hubs_unsupported: Vec<String> = Vec::new();
        for h in &hub_results {
            let name = h.get("hub").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let Some(bs) = h.get("bus_state") else { continue };
            if bs.get("audit_unsupported").and_then(|v| v.as_bool()) == Some(true) {
                hubs_unsupported.push(name);
                continue;
            }
            let rd = bs.get("runtime_dir").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let vol = bs.get("runtime_dir_volatile").and_then(|v| v.as_bool()).unwrap_or(false);
            let present = bs.get("audit_present").and_then(|v| v.as_bool()).unwrap_or(false);
            if vol {
                hubs_volatile.push((name, rd));
            } else if !present {
                hubs_missing.push((name, rd));
            } else {
                hubs_durable.push(name);
            }
        }
        let verdict = if !hubs_volatile.is_empty() {
            "VOLATILE"
        } else if !hubs_unsupported.is_empty() || !hubs_missing.is_empty() {
            "UNCERTAIN"
        } else if !hubs_durable.is_empty() {
            "DURABLE"
        } else {
            "UNCERTAIN"
        };
        if !json {
            eprintln!();
            eprintln!("=== T-1446 G-050 audit-sweep ===");
            eprintln!("Verdict: {}", verdict);
            if !hubs_durable.is_empty() {
                eprintln!("  DURABLE: {}", hubs_durable.join(", "));
            }
            if !hubs_volatile.is_empty() {
                eprintln!("  VOLATILE (runtime_dir on /tmp/):");
                for (name, rd) in &hubs_volatile {
                    eprintln!("    {name}: runtime_dir={rd}");
                }
                eprintln!("    → migrate runtime_dir off /tmp (see T-1294 / T-1296)");
            }
            if !hubs_missing.is_empty() {
                eprintln!("  NO meta.db (fresh runtime_dir or never posted):");
                for (name, rd) in &hubs_missing {
                    eprintln!("    {name}: runtime_dir={rd}");
                }
            }
            if !hubs_unsupported.is_empty() {
                eprintln!(
                    "  UNSUPPORTED (pre-T-1446, upgrade to measure): {}",
                    hubs_unsupported.join(", ")
                );
            }
        }
        Some(serde_json::json!({
            "verdict": verdict,
            "hubs_durable": hubs_durable,
            "hubs_volatile": hubs_volatile.iter().map(|(n, r)| serde_json::json!({"hub": n, "runtime_dir": r})).collect::<Vec<_>>(),
            "hubs_missing": hubs_missing.iter().map(|(n, r)| serde_json::json!({"hub": n, "runtime_dir": r})).collect::<Vec<_>>(),
            "hubs_unsupported": hubs_unsupported,
        }))
    } else {
        None
    };

    // T-1462: capture current snapshot timestamp once, used both for embedding
    // in JSON output and for diff rate calculation.
    let snapshot_ts_ms: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // T-1462: if a prior snapshot was supplied, render the diff block. Render
    // before JSON serialization so non-JSON output sees it as part of the
    // human-readable summary. For JSON, attach as `legacy_diff` field.
    let legacy_diff_obj: Option<LegacyDiff> = if let (Some(prior), Some(current_ls)) =
        (prior_snapshot.as_ref(), legacy_summary_obj.as_ref())
    {
        let prior_ls = prior.get("legacy_summary").expect("validated up-front");
        let prior_ts_ms = prior.get("_snapshot_ts_ms").and_then(|v| v.as_u64());
        let d = compute_legacy_diff(prior_ls, current_ls, prior_ts_ms, snapshot_ts_ms);
        if !json {
            let label = diff
                .as_deref()
                .and_then(|p| p.file_name().and_then(|s| s.to_str()))
                .unwrap_or("<snapshot>")
                .to_string();
            print_legacy_diff_block(&d, &label);
        }
        Some(d)
    } else {
        None
    };

    // T-1468: if --trend was supplied and we read snapshots, render the trend.
    // Append the *current* snapshot to the time-series so the operator sees
    // today's value alongside the prior days. Trend renders before JSON
    // serialization (mirroring --diff's pattern) so the human-mode output
    // shows the trend block under cut-readiness.
    let legacy_trend_obj: Option<(Vec<TrendPoint>, Trajectory, Option<EtaForecast>)> = if !trend_snapshots.is_empty() {
        // Borrow refs into the snapshots vec, then append a synthesized
        // "current" entry built from this run's legacy_summary + snapshot_ts_ms.
        let mut series: Vec<(String, &serde_json::Value)> = trend_snapshots
            .iter()
            .map(|(label, doc)| (label.clone(), doc))
            .collect();
        let current_doc: serde_json::Value;
        if let Some(ls) = legacy_summary_obj.as_ref() {
            current_doc = serde_json::json!({
                "_snapshot_ts_ms": snapshot_ts_ms,
                "legacy_summary": ls,
            });
            series.push(("(current)".to_string(), &current_doc));
        }
        let (points, trajectory) = compute_legacy_trend(&series);
        // T-1470: forecast ETA-to-zero from the trend points. Returns None
        // when the fit isn't meaningful (flat, growing, single point, ...).
        let eta = compute_eta_to_zero(&points, snapshot_ts_ms as u64);
        if !json {
            print_legacy_trend_block(&points, &trajectory, eta.as_ref());
        }
        Some((points, trajectory, eta))
    } else if trend.is_some() {
        // Caller asked for --trend but no parseable snapshots existed.
        // Print an explanation in non-JSON mode; JSON gets an empty array.
        if !json {
            print_legacy_trend_block(&[], &Trajectory::Flat, None);
        }
        Some((Vec::new(), Trajectory::Flat, None))
    } else {
        None
    };

    // T-1618: compute action-items rollup before json_doc construction so
    // both --json and text paths see the same content. Mirrors T-1617's
    // text rendering. Each entry is a single-line operator-actionable hint
    // grouped by failure class.
    let stale_count = fleet_versions.get("0.9.0").copied().unwrap_or(0);
    let total_hubs_count = hub_results.len() as u32;
    let mut action_items: Vec<String> = Vec::new();
    if stale_count > 0 {
        action_items.push(format!(
            "Version skew: {}/{} hubs on 0.9.0 — restart hub processes to pick up newer binary (CLI is on {})",
            stale_count, total_hubs_count, cli_version
        ));
    }
    if total_fail > 0 {
        action_items.push(format!(
            "Failed hubs: {} hub(s) returned errors above — see [FAIL]/hint lines for per-hub diagnostic",
            total_fail
        ));
    }

    // T-1463: build the JSON document unconditionally when either --json
    // or --save-snapshot was requested, so both paths emit the same value.
    let need_json_doc = json || save_snapshot.is_some();
    let json_doc = if need_json_doc {
        let mut top = serde_json::json!({
            "ok": total_fail == 0,
            "hubs": hub_results,
            "summary": {"total": hub_results.len(), "pass": total_pass, "warn": total_warn, "fail": total_fail},
            "fleet_versions": fleet_versions,
            "action_items": action_items,
            "_snapshot_ts_ms": snapshot_ts_ms,
            "queue_status": queue_status_obj,
        });
        if let Some(ls) = legacy_summary_obj.clone()
            && let Some(obj) = top.as_object_mut()
        {
            obj.insert("legacy_summary".to_string(), ls);
        }
        if let Some(bs) = bus_state_summary_obj.clone()
            && let Some(obj) = top.as_object_mut()
        {
            obj.insert("bus_state_summary".to_string(), bs);
        }
        // T-1666: attach pin_check_summary when --include-pin-check ran. Verdict
        // mirrors `fleet verify` semantics so the two commands agree on rotation
        // state. Field is absent when the flag is off — preserves existing JSON
        // shape for default-mode callers.
        if let Some(pcs) = pin_check_summary.clone()
            && let Some(obj) = top.as_object_mut()
        {
            obj.insert("pin_check_summary".to_string(), pcs);
        }
        if let Some(d) = legacy_diff_obj.as_ref()
            && let Some(obj) = top.as_object_mut()
        {
            obj.insert("legacy_diff".to_string(), legacy_diff_to_json(d));
        }
        // T-1468: attach trend block under legacy_summary (when present) so
        // dashboards consuming the same legacy_summary key get everything in
        // one place. Falls back to top-level when legacy_summary is absent
        // (defensive — should not happen given trend_snapshots requires
        // legacy_usage).
        if let Some((points, trajectory, eta)) = legacy_trend_obj.as_ref() {
            let trend_json = legacy_trend_to_json(points, trajectory, eta.as_ref());
            if let Some(obj) = top.as_object_mut() {
                let attached = obj
                    .get_mut("legacy_summary")
                    .and_then(|ls| ls.as_object_mut())
                    .map(|ls_obj| {
                        ls_obj.insert("trend".to_string(), trend_json.clone());
                    })
                    .is_some();
                if !attached {
                    obj.insert("legacy_trend".to_string(), trend_json);
                }
            }
        }
        Some(top)
    } else {
        None
    };

    // T-1463: persist snapshot to disk. Atomic via .tmp + rename so a future
    // --diff never reads a half-written file.
    if let (Some(path), Some(doc)) = (save_snapshot.as_deref(), json_doc.as_ref()) {
        let body = serde_json::to_string_pretty(doc)?;
        let tmp = path.with_extension({
            let cur = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if cur.is_empty() { "tmp".to_string() } else { format!("{cur}.tmp") }
        });
        std::fs::write(&tmp, &body)
            .with_context(|| format!("--save-snapshot: failed to write {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("--save-snapshot: failed to rename {} -> {}", tmp.display(), path.display()))?;
        if !json {
            eprintln!();
            eprintln!("snapshot saved: {}", path.display());
        }
    }

    if json {
        let doc = json_doc.expect("built when need_json_doc");
        println!("{}", serde_json::to_string_pretty(&doc)?);
    } else {
        eprintln!("Fleet summary: {} hub(s), {} ok, {} warn, {} fail",
            hub_results.len(), total_pass, total_warn, total_fail);

        // T-1132: Versions in fleet summary — flag skew with a hint.
        if !fleet_versions.is_empty() {
            let parts: Vec<String> = fleet_versions
                .iter()
                .map(|(v, count)| {
                    let plural = if *count == 1 { "hub" } else { "hubs" };
                    format!("{} ({} {})", v, count, plural)
                })
                .collect();
            eprintln!("Versions in fleet: {}", parts.join(", "));
            // Count distinct *known* versions to decide whether to flag skew.
            let distinct_known = fleet_versions
                .keys()
                .filter(|v| v.as_str() != "unknown")
                .count();
            if distinct_known > 1 {
                eprintln!(
                    "  hint: fleet version skew detected — Tier-B RPCs may fail across the diversity; see docs on Tier-A vs Tier-B methods"
                );
            }
        }

        // T-1617 / T-1618: action-items rollup. Per-class signals aggregated
        // into a single block so the operator gets at-a-glance "what to do"
        // without parsing N identical per-hub WARN lines. Per-hub detail
        // above is preserved; this is a roll-up summary. The Vec is built
        // before json_doc construction so --json sees the same content.
        if !action_items.is_empty() {
            eprintln!();
            eprintln!("Action items:");
            for item in &action_items {
                eprintln!("  - {}", item);
            }
        }
    }

    // T-1465: verdict-mapped exit code, applied AFTER all output is produced
    // so operators see the human/JSON summary before the shell decides.
    // Connectivity failures keep precedence: a hub that did not connect
    // means the verdict is built from incomplete data, so the existing
    // non-zero `total_fail > 0` semantic stays authoritative — we only
    // override exit when total_fail == 0 (clean fleet sweep).
    if exit_code_on_verdict
        && total_fail == 0
        && let Some(ls) = legacy_summary_obj.as_ref()
        && let Some(verdict) = ls.get("verdict").and_then(|v| v.as_str())
    {
        let code = verdict_to_exit_code(verdict);
        if code != 0 {
            std::process::exit(code);
        }
    }

    Ok(())
}

/// T-1614: Build an actionable hint for an unreachable hub based on the
/// address kind. Used by both the "Cannot connect / no route" branch and
/// the timeout branch in `cmd_fleet_status` — both indicate the operator
/// can't reach the hub, but the next probe depends on whether the address
/// is loopback, RFC5737 documentation range, RFC1918 private, or public.
///
/// Connection-refused (RST from a listening kernel without a bound port)
/// stays specialized in the caller — that's "process not running", a
/// different operator response.
fn classify_unreachable_hint(name: &str, address: &str) -> String {
    let host = address.split(':').next().unwrap_or(address);
    let port = address.split(':').nth(1).unwrap_or("?");
    if host == "localhost" || host.starts_with("127.") {
        format!(
            "{}: Localhost unreachable — hub not running on this host. Start with: termlink hub start (verify with: termlink hub status)",
            name
        )
    } else if host.starts_with("192.0.2.")
        || host.starts_with("198.51.100.")
        || host.starts_with("203.0.113.")
    {
        format!(
            "{}: Profile points at {} (RFC5737 documentation/test range — never routable). Stale config; remove with: termlink remote profile remove {}",
            name, host, name
        )
    } else if is_rfc1918(host) {
        format!(
            "{}: Private-network hub at {} unreachable — verify route + remote process. Probe: nc -zv {} {} ; ssh root@{} systemctl status termlink-hub",
            name, host, host, port, host
        )
    } else {
        format!(
            "{}: Network unreachable to {} — likely firewall/route. Probe: nc -zv {} {} ; ping -c2 {}",
            name, address, host, port, host
        )
    }
}

/// T-1614: Match RFC1918 private network ranges (10/8, 172.16/12, 192.168/16).
/// Used by the fleet-status TIMEOUT hint to distinguish "remote private hub
/// unreachable" from generic "public/firewall block" — different probes apply.
fn is_rfc1918(host: &str) -> bool {
    if host.starts_with("10.") || host.starts_with("192.168.") {
        return true;
    }
    if let Some(rest) = host.strip_prefix("172.") {
        if let Some(second_octet) = rest.split('.').next() {
            if let Ok(n) = second_octet.parse::<u8>() {
                return (16..=31).contains(&n);
            }
        }
    }
    false
}

/// T-1034: Classify fleet doctor errors into actionable diagnostic hints.
fn classify_fleet_error(msg: &str, address: &str) -> String {
    if msg.contains("invalid signature") || msg.contains("Token validation failed") {
        "Secret mismatch — hub was likely restarted with a new secret. \
         Fetch the current secret from the remote hub's hub.secret file".to_string()
    } else if msg.contains("TOFU VIOLATION") || msg.contains("fingerprint changed") {
        format!("Hub certificate changed. If expected (hub restart), clear with: \
         termlink tofu clear {address}")
    } else if msg.contains("Connection refused") {
        "Hub is not listening on this port. Check if the hub process is running \
         on the remote host (systemctl status termlink-hub)".to_string()
    } else if msg.contains("Secret file not found") {
        "The configured secret_file path does not exist. \
         Check hubs.toml and verify the file is present".to_string()
    } else if msg.contains("InvalidContentType") || msg.contains("tls") || msg.contains("TLS") {
        "TLS handshake failed — the hub may not be running TLS on this port, \
         or there is a protocol version mismatch".to_string()
    } else {
        "Unexpected error — check hub logs on the remote host for details".to_string()
    }
}

/// T-1106: Run a layered connectivity probe per hub.
///
/// Probes in order: TCP connect → TLS handshake → HMAC auth → RPC ping.
/// Each layer's result (pass/fail + latency) is reported independently so
/// the operator can see exactly where a connection breaks. Stops at the
/// first failing layer — subsequent layers require the prior to succeed.
pub(crate) async fn cmd_net_test(
    profile_filter: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    use serde_json::json;
    use std::time::{Duration, Instant};

    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({
                "ok": true, "hubs": [],
                "summary": {"total": 0, "healthy": 0, "degraded": 0, "unreachable": 0},
            }))?);
        } else {
            eprintln!("No hubs configured. Add hubs with: termlink remote profile add <name> <host:port> --secret-file <path>");
        }
        return Ok(());
    }

    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();
    if let Some(filter) = profile_filter {
        hub_names.retain(|n| n.as_str() == filter);
        if hub_names.is_empty() {
            // T-1917: honor --json on the profile-not-found error path.
            if json {
                super::json_error_exit(json!({
                    "ok": false,
                    "error": format!("Hub profile '{}' not found. Run: termlink remote profile list", filter),
                }));
            }
            anyhow::bail!("Hub profile '{}' not found. Run: termlink remote profile list", filter);
        }
    }

    let timeout_dur = Duration::from_secs(timeout_secs);
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut healthy = 0u32;
    let mut degraded = 0u32;
    let mut unreachable = 0u32;

    for name in &hub_names {
        let entry = &config.hubs[*name];

        let (host, port) = match parse_host_port(&entry.address) {
            Ok(hp) => hp,
            Err(e) => {
                unreachable += 1;
                results.push(json!({
                    "hub": name, "address": entry.address,
                    "healthy": false, "diagnosis": format!("invalid address: {}", e),
                    "layers": {},
                }));
                continue;
            }
        };

        let mut layers = serde_json::Map::new();
        let mut hub_healthy = true;
        // T-1649: widened from `Option<&'static str>` to `Option<String>` so the
        // HMAC-mismatch diagnosis can carry per-hub formatted heal incantation
        // (profile name + declared-channel-aware `--bootstrap-from` argument).
        let mut diagnosis: Option<String> = None;

        // --- L1: TCP ---
        let tcp_start = Instant::now();
        let tcp_result = tokio::time::timeout(
            timeout_dur,
            tokio::net::TcpStream::connect((host.as_str(), port)),
        ).await;
        let tcp_latency = tcp_start.elapsed().as_millis() as u64;
        let tcp_ok = matches!(tcp_result, Ok(Ok(_)));
        layers.insert("tcp".to_string(), match &tcp_result {
            Ok(Ok(_)) => json!({"status": "pass", "latency_ms": tcp_latency}),
            Ok(Err(e)) => json!({"status": "fail", "latency_ms": tcp_latency, "error": e.to_string()}),
            Err(_) => json!({"status": "timeout", "latency_ms": timeout_secs * 1000}),
        });
        if !tcp_ok {
            hub_healthy = false;
            diagnosis = Some("Network-level failure — check firewall/VPN/routing and hub process is listening on the configured port".to_string());
        }

        // --- L2: TLS ---
        if tcp_ok {
            let addr = termlink_protocol::TransportAddr::Tcp {
                host: host.clone(),
                port,
            };
            let tls_start = Instant::now();
            let tls_result = tokio::time::timeout(
                timeout_dur,
                client::Client::connect_addr(&addr),
            ).await;
            let tls_latency = tls_start.elapsed().as_millis() as u64;

            match tls_result {
                Ok(Ok(mut rpc_client)) => {
                    layers.insert("tls".to_string(),
                        json!({"status": "pass", "latency_ms": tls_latency}));

                    // --- L3: AUTH ---
                    let auth_outcome = net_probe_auth(&mut rpc_client, entry, timeout_dur).await;
                    match auth_outcome {
                        Ok(auth_latency) => {
                            layers.insert("auth".to_string(),
                                json!({"status": "pass", "latency_ms": auth_latency}));

                            // --- L4: PING (session.discover) ---
                            let ping_start = Instant::now();
                            let ping_result = tokio::time::timeout(
                                timeout_dur,
                                rpc_client.call("session.discover", json!("net-ping"), json!({})),
                            ).await;
                            let ping_latency = ping_start.elapsed().as_millis() as u64;
                            match ping_result {
                                Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_))) => {
                                    layers.insert("ping".to_string(),
                                        json!({"status": "pass", "latency_ms": ping_latency}));
                                }
                                Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e))) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC call rejected — hub is authenticated but refusing session.discover".to_string());
                                    layers.insert("ping".to_string(), json!({
                                        "status": "fail", "latency_ms": ping_latency,
                                        "error": format!("{} {}", e.error.code, e.error.message),
                                    }));
                                }
                                Ok(Err(e)) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC transport error after auth — hub may have disconnected".to_string());
                                    layers.insert("ping".to_string(), json!({
                                        "status": "fail", "latency_ms": ping_latency,
                                        "error": e.to_string(),
                                    }));
                                }
                                Err(_) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC timeout after auth — hub is slow or overloaded".to_string());
                                    layers.insert("ping".to_string(), json!({
                                        "status": "timeout", "latency_ms": timeout_secs * 1000,
                                    }));
                                }
                            }
                        }
                        Err((auth_latency, msg)) => {
                            hub_healthy = false;
                            // T-1649: format per-hub heal incantation (profile name +
                            // declared-channel-aware --bootstrap-from arg) so operators
                            // get a copy-pasteable command instead of <profile>+ssh:<host>.
                            diagnosis = Some(format_hmac_mismatch_diagnosis(name, entry));
                            layers.insert("auth".to_string(), json!({
                                "status": "fail", "latency_ms": auth_latency,
                                "error": msg,
                            }));
                        }
                    }
                }
                Ok(Err(e)) => {
                    hub_healthy = false;
                    let msg = e.to_string();
                    diagnosis = Some(if msg.contains("TOFU") || msg.contains("fingerprint") {
                        "TLS cert changed — run: termlink tofu clear <host:port> and retry".to_string()
                    } else {
                        "TLS handshake failed — hub may not be speaking TLS, or cert is invalid".to_string()
                    });
                    layers.insert("tls".to_string(), json!({
                        "status": "fail", "latency_ms": tls_latency,
                        "error": msg,
                    }));
                }
                Err(_) => {
                    hub_healthy = false;
                    diagnosis = Some("TLS handshake timed out — hub is slow or silently dropping TLS".to_string());
                    layers.insert("tls".to_string(), json!({
                        "status": "timeout", "latency_ms": timeout_secs * 1000,
                    }));
                }
            }
        }

        if hub_healthy {
            healthy += 1;
        } else if layers.get("tcp").and_then(|l| l.get("status")).and_then(|s| s.as_str()) == Some("pass") {
            degraded += 1;
        } else {
            unreachable += 1;
        }

        let mut hub_result = json!({
            "hub": name,
            "address": entry.address,
            "healthy": hub_healthy,
            "layers": layers,
        });
        if let Some(d) = diagnosis {
            hub_result["diagnosis"] = json!(d);
        }
        results.push(hub_result);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": unreachable == 0 && degraded == 0,
            "hubs": results,
            "summary": {
                "total": hub_names.len(),
                "healthy": healthy,
                "degraded": degraded,
                "unreachable": unreachable,
            },
        }))?);
    } else {
        render_net_test_text(&results, healthy, degraded, unreachable);
    }

    Ok(())
}

/// Parse "host:port" into (host, port) — shared logic with connect_remote_hub.
fn parse_host_port(addr: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("expected host:port, got '{}'", addr);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("invalid port in '{}'", addr))?;
    Ok((host, port))
}

/// Run the AUTH layer of the net test: build a token from the hub's secret
/// and call `hub.auth`. Returns Ok(latency_ms) on success or Err((latency_ms, message)).
async fn net_probe_auth(
    rpc_client: &mut client::Client,
    entry: &HubEntry,
    timeout_dur: std::time::Duration,
) -> std::result::Result<u64, (u64, String)> {
    use std::time::Instant;
    use termlink_session::auth::{self, PermissionScope};

    let start = Instant::now();

    // Read secret (file or inline)
    let hex = match (entry.secret_file.as_deref(), entry.secret.as_deref()) {
        (Some(path), _) => match std::fs::read_to_string(path) {
            Ok(s) => s.trim().to_string(),
            Err(e) => return Err((start.elapsed().as_millis() as u64,
                format!("cannot read secret file {}: {}", path, e))),
        },
        (None, Some(h)) => h.to_string(),
        (None, None) => return Err((start.elapsed().as_millis() as u64,
            "no secret configured (neither secret_file nor inline secret)".to_string())),
    };
    if hex.len() != 64 {
        return Err((start.elapsed().as_millis() as u64,
            format!("secret must be 64 hex chars, got {}", hex.len())));
    }
    let secret_bytes: Vec<u8> = match (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<std::result::Result<Vec<u8>, _>>()
    {
        Ok(b) => b,
        Err(e) => return Err((start.elapsed().as_millis() as u64,
            format!("invalid hex in secret: {}", e))),
    };
    let secret: auth::TokenSecret = match secret_bytes.try_into() {
        Ok(s) => s,
        Err(_) => return Err((start.elapsed().as_millis() as u64,
            "secret must decode to exactly 32 bytes".to_string())),
    };

    let scope_str = entry.scope.as_deref().unwrap_or("execute");
    let perm_scope = match scope_str {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => return Err((start.elapsed().as_millis() as u64,
            format!("invalid scope '{}'", scope_str))),
    };
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    let auth_result = tokio::time::timeout(
        timeout_dur,
        rpc_client.call("hub.auth", serde_json::json!("net-auth"),
            serde_json::json!({"token": token.raw})),
    ).await;

    let latency = start.elapsed().as_millis() as u64;
    match auth_result {
        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_))) => Ok(latency),
        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e))) => {
            Err((latency, format!("{} {}", e.error.code, e.error.message)))
        }
        Ok(Err(e)) => Err((latency, format!("RPC error: {}", e))),
        Err(_) => Err((latency, format!("auth timeout after {}s", timeout_dur.as_secs()))),
    }
}

/// Render the text output for `termlink net test`.
fn render_net_test_text(
    results: &[serde_json::Value],
    healthy: u32,
    degraded: u32,
    unreachable: u32,
) {
    eprintln!();
    for hub in results {
        let name = hub["hub"].as_str().unwrap_or("?");
        let addr = hub["address"].as_str().unwrap_or("?");
        let hub_healthy = hub["healthy"].as_bool().unwrap_or(false);

        let (badge, colour) = if hub_healthy {
            ("HEALTHY", "\x1b[32m")
        } else {
            ("FAIL", "\x1b[31m")
        };
        eprintln!("  {colour}{badge}\x1b[0m  {name}  ({addr})");

        for layer in ["tcp", "tls", "auth", "ping"] {
            let Some(entry) = hub["layers"].get(layer) else { continue };
            let status = entry["status"].as_str().unwrap_or("?");
            let latency = entry["latency_ms"].as_u64().unwrap_or(0);
            let marker = match status {
                "pass" => "\x1b[32mPASS\x1b[0m",
                "fail" => "\x1b[31mFAIL\x1b[0m",
                "timeout" => "\x1b[31mTIME\x1b[0m",
                _ => "----",
            };
            let layer_upper = layer.to_uppercase();
            eprintln!("    {marker}  {layer_upper:<4}  {latency:>4}ms");
            if status != "pass"
                && let Some(err) = entry["error"].as_str()
            {
                eprintln!("          \x1b[2m└─ {}\x1b[0m", err);
            }
        }

        if let Some(diag) = hub["diagnosis"].as_str() {
            eprintln!("    \x1b[33m→\x1b[0m {}", diag);
        }
        eprintln!();
    }

    let total = results.len();
    if degraded == 0 && unreachable == 0 {
        eprintln!("  NET: \x1b[32mall {} hub(s) fully reachable\x1b[0m", total);
    } else {
        eprintln!("  NET: {} hub(s), \x1b[32m{} healthy\x1b[0m, \x1b[33m{} degraded\x1b[0m, \x1b[31m{} unreachable\x1b[0m",
            total, healthy, degraded, unreachable);
    }
    eprintln!();
}

/// T-1052: classify an error message into the auth/cert drift classes we care about.
/// Returns `None` for unrelated errors (connection refused, etc.) so we stay quiet.
fn auth_mismatch_class(msg: &str) -> Option<&'static str> {
    if msg.contains("invalid signature") || msg.contains("Token validation failed") {
        Some("auth-mismatch")
    } else if msg.contains("TOFU VIOLATION") || msg.contains("fingerprint changed") {
        Some("tofu-violation")
    } else {
        None
    }
}

/// T-1682: derive the watch loop's effective conn-state class for one hub_obj.
///
/// `cmd_fleet_doctor` writes `status` as one of "ok" / "error" / "timeout"
/// in the JSON output. The watch loop in T-1681 needs to distinguish a
/// secret-only-rotation auth-mismatch from a generic connect failure so
/// `--auto-heal` can gate on it. We compute the finer class here by
/// classifying the `error` message via `auth_mismatch_class` whenever
/// `status == "error"`; otherwise we pass `status` through unchanged.
///
/// JSON output is NOT modified — this remapping lives only in the watch
/// parser's in-memory state.
fn derive_watch_conn(hub: &serde_json::Value) -> String {
    let raw = hub
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");
    if raw == "error" {
        let err_msg = hub.get("error").and_then(|s| s.as_str()).unwrap_or("");
        match auth_mismatch_class(err_msg) {
            Some(class) => class.to_string(),
            None => raw.to_string(),
        }
    } else {
        raw.to_string()
    }
}

/// T-1052: compute UTC ISO-8601 timestamp. Same algorithm as `termlink_session::tofu::now_utc`
/// but inlined here to avoid exporting a new public symbol purely for this helper.
fn utc_iso8601_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let ydays = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < ydays { break; }
        remaining -= ydays;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0usize;
    for (i, &md) in mdays.iter().enumerate() {
        if remaining < md as i64 { mo = i; break; }
        remaining -= md as i64;
    }
    format!("{y:04}-{:02}-{:02}T{h:02}:{m:02}:{s:02}Z", mo + 1, remaining + 1)
}

/// T-1052: return `true` if a dedupe marker exists, is younger than 24h, and its
/// recorded fingerprint matches the current one (→ skip recording a duplicate).
fn marker_deduped(marker: &std::path::Path, fingerprint: &str) -> bool {
    let Ok(meta) = std::fs::metadata(marker) else { return false; };
    let Ok(modified) = meta.modified() else { return false; };
    let Ok(age) = modified.elapsed() else { return false; };
    if age.as_secs() >= 86400 { return false; }
    let Ok(content) = std::fs::read_to_string(marker) else { return false; };
    content.trim() == fingerprint
}

/// T-1052 / R1 compliance: when fleet-doctor sees an auth-mismatch or TOFU violation,
/// append a learning to `.context/project/learnings.yaml` carrying the hub address,
/// the current pinned fingerprint (or "unknown"), and an ISO-8601 UTC timestamp.
///
/// Future agents can compare the recorded `hub_fingerprint=` against the currently
/// pinned fingerprint to detect memory drift (the learning was written under a
/// previous rotation of the cert).
///
/// Deduped via `.context/working/.fleet-learning-<hub>`: skip if a marker younger
/// than 24h exists AND the fingerprint hasn't changed since.
///
/// Best-effort only: silently no-ops when run outside a framework-managed project
/// (no `.context/project/` dir present). Never fails the caller.
pub(crate) fn maybe_record_auth_mismatch_learning(
    hub_name: &str,
    address: &str,
    error_msg: &str,
) -> Result<()> {
    let class = match auth_mismatch_class(error_msg) {
        Some(c) => c,
        None => return Ok(()),
    };

    // Locate framework project root from CWD. No-op outside framework projects.
    let cwd = std::env::current_dir()?;
    let learnings_path = cwd.join(".context/project/learnings.yaml");
    if !learnings_path.exists() {
        return Ok(());
    }
    let working_dir = cwd.join(".context/working");
    if let Err(e) = std::fs::create_dir_all(&working_dir) {
        return Err(anyhow::anyhow!("failed to create .context/working: {e}"));
    }

    // Look up the currently pinned fingerprint (may be absent if TOFU entry not yet recorded).
    let fingerprint = termlink_session::tofu::KnownHubStore::default_store()
        .get(address)
        .unwrap_or_else(|| "unknown".to_string());

    // Dedupe: marker file content = fingerprint at last recording. Same fingerprint
    // within 24h → skip. Changed fingerprint or older marker → record again.
    let safe_name: String = hub_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let marker = working_dir.join(format!(".fleet-learning-{safe_name}"));
    if marker_deduped(&marker, &fingerprint) {
        return Ok(());
    }

    // Build the learning entry. Fingerprint + timestamp are embedded in the learning
    // text as `key=value` pairs so future drift-detection can parse them without
    // extending the framework's `add-learning` schema.
    let now_iso = utc_iso8601_now();
    let date_only = now_iso.split('T').next().unwrap_or("");
    let learning_text = format!(
        "Fleet doctor observed {class} on hub '{hub_name}' ({address}). \
         date_observed={now_iso} hub_fingerprint={fingerprint}. \
         T-1051 Option D auto-registration — if a later agent sees a different pinned \
         fingerprint for this hub, this learning is stale and should not be acted on.",
    );

    // Determine next PL-XXX id by scanning existing entries.
    let existing = std::fs::read_to_string(&learnings_path).unwrap_or_default();
    let max_id = existing
        .lines()
        .filter_map(|l| l.trim().strip_prefix("- id: PL-"))
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let next_id = max_id + 1;
    let id = format!("PL-{:03}", next_id);

    let entry = format!(
        "- id: {id}\n  learning: \"{text}\"\n  source: T-1052\n  task: T-1051\n  date: {date}\n  context: fleet-doctor auto-registered\n  application: \"Drift-detection: compare hub_fingerprint in this learning against current KnownHubStore.get(address)\"\n",
        text = learning_text,
        date = date_only,
    );

    let mut new_content = existing;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&entry);
    std::fs::write(&learnings_path, new_content)
        .with_context(|| format!("failed to write {}", learnings_path.display()))?;

    // Refresh the dedupe marker.
    let _ = std::fs::write(&marker, &fingerprint);

    Ok(())
}

// =========================================================================
// T-1053: fleet-doctor concern auto-registration (G-019 compliance)
//
// One-off observations are recorded as learnings by T-1052. A sustained
// pattern — ≥3 consecutive fleet-doctor failures for the same hub, spanning
// >24h — is promoted to a concern in .context/project/concerns.yaml so it
// surfaces in Watchtower and audit passes.
//
// Per-hub state is persisted in .context/working/.fleet-failure-state.json
// as { "hubs": { "<hub>": { "consecutive_failures": N, "first_failure_at":
// "...", "last_failure_at": "...", "last_class": "...", "concern_registered":
// bool } } }. Passing runs reset the counter and re-arm concern_registered.
// =========================================================================

const FLEET_CONCERN_THRESHOLD: u32 = 3;
const FLEET_CONCERN_AGE_SECS: u64 = 86_400;

/// State file path. Returns None when outside a framework project.
fn fleet_state_path() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let working = cwd.join(".context/working");
    if !working.exists() {
        return None;
    }
    Some(working.join(".fleet-failure-state.json"))
}

fn fleet_concerns_path() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let concerns = cwd.join(".context/project/concerns.yaml");
    if !concerns.exists() {
        return None;
    }
    Some(concerns)
}

/// Parse the `YYYY-MM-DDTHH:MM:SSZ` format produced by `utc_iso8601_now`
/// into seconds since the epoch. Returns `None` on parse failure.
///
/// Deliberately permissive — the input comes from our own writer, so we
/// accept whatever we emit and silently fail on anything else rather than
/// panicking inside a best-effort pathway.
fn parse_iso8601_utc(s: &str) -> Option<u64> {
    // Format: YYYY-MM-DDTHH:MM:SSZ  (20 chars)
    if s.len() < 19 || !s.ends_with('Z') {
        return None;
    }
    let y: i64 = s.get(0..4)?.parse().ok()?;
    let mo: u32 = s.get(5..7)?.parse().ok()?;
    let d: u32 = s.get(8..10)?.parse().ok()?;
    let h: u32 = s.get(11..13)?.parse().ok()?;
    let mi: u32 = s.get(14..16)?.parse().ok()?;
    let se: u32 = s.get(17..19)?.parse().ok()?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60 {
        return None;
    }

    // Days from epoch (1970-01-01) to Y-M-D.
    let mut days: i64 = 0;
    for yr in 1970..y {
        days += if yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0) { 366 } else { 365 };
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &md) in mdays.iter().enumerate() {
        if (i as u32) + 1 == mo {
            break;
        }
        days += md as i64;
    }
    days += (d as i64) - 1;

    let secs = (days * 86_400) + (h as i64) * 3600 + (mi as i64) * 60 + (se as i64);
    if secs < 0 { None } else { Some(secs as u64) }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Load state file, returning an empty map if the file is absent or malformed.
/// Best-effort: we never fail the caller just because state is unreadable.
fn load_fleet_state(path: &std::path::Path) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({"hubs": {}}))
}

fn save_fleet_state(path: &std::path::Path, state: &serde_json::Value) {
    if let Ok(s) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, s);
    }
}

/// Update per-hub failure tracking and, on threshold breach, append a concern.
///
/// Classification rules:
/// - `class = None` (no auth-class error) → treated as a reset signal for the
///   hub's auth-failure streak. The hub may still be failing for other reasons
///   (connection refused, TLS handshake, etc.), but those aren't the
///   auth-rotation pattern T-1051 targets.
/// - `class = Some("auth-mismatch" | "tofu-violation")` → increment the streak.
///
/// Best-effort: silently no-ops outside framework projects (no `.context/`).
/// Never fails the caller.
pub(crate) fn maybe_track_fleet_failure(
    hub_name: &str,
    address: &str,
    class: Option<&str>,
) -> Result<()> {
    let state_path = match fleet_state_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    let mut state = load_fleet_state(&state_path);
    let now_iso = utc_iso8601_now();
    let now_secs = now_unix_secs();

    // Ensure hubs object exists.
    if !state.get("hubs").map(|v| v.is_object()).unwrap_or(false) {
        state["hubs"] = serde_json::json!({});
    }

    // Take (or init) the per-hub entry. We mutate via take-replace to avoid
    // borrow-checker fights with serde_json's indexing API.
    let mut hub_state = state["hubs"]
        .get(hub_name)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({
            "consecutive_failures": 0,
            "first_failure_at": serde_json::Value::Null,
            "last_failure_at": serde_json::Value::Null,
            "last_class": serde_json::Value::Null,
            "concern_registered": false,
        }));

    match class {
        None => {
            // Reset — pass or non-auth failure.
            hub_state["consecutive_failures"] = serde_json::json!(0);
            hub_state["first_failure_at"] = serde_json::Value::Null;
            hub_state["last_failure_at"] = serde_json::Value::Null;
            hub_state["last_class"] = serde_json::Value::Null;
            hub_state["concern_registered"] = serde_json::json!(false);
        }
        Some(c) => {
            let prior = hub_state["consecutive_failures"].as_u64().unwrap_or(0);
            let new_count = prior + 1;
            hub_state["consecutive_failures"] = serde_json::json!(new_count);
            if prior == 0 {
                hub_state["first_failure_at"] = serde_json::json!(now_iso.clone());
            }
            hub_state["last_failure_at"] = serde_json::json!(now_iso.clone());
            hub_state["last_class"] = serde_json::json!(c);

            // Threshold check.
            let already_registered = hub_state["concern_registered"].as_bool().unwrap_or(false);
            let first_at = hub_state["first_failure_at"].as_str().unwrap_or("");
            let first_secs = parse_iso8601_utc(first_at).unwrap_or(now_secs);
            let age = now_secs.saturating_sub(first_secs);

            if !already_registered
                && (new_count as u32) >= FLEET_CONCERN_THRESHOLD
                && age > FLEET_CONCERN_AGE_SECS
                && append_fleet_concern(hub_name, address, c, new_count as u32, first_at, &now_iso).is_ok()
            {
                hub_state["concern_registered"] = serde_json::json!(true);
            }
        }
    }

    state["hubs"][hub_name] = hub_state;
    save_fleet_state(&state_path, &state);
    Ok(())
}

/// Append a gap-type concern to `.context/project/concerns.yaml`.
fn append_fleet_concern(
    hub_name: &str,
    address: &str,
    class: &str,
    consecutive: u32,
    first_at: &str,
    now_iso: &str,
) -> Result<()> {
    let path = match fleet_concerns_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let max_id = existing
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("- id: G-").or_else(|| t.strip_prefix("id: G-"))
        })
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let id = format!("G-{:03}", max_id + 1);
    let date_only = now_iso.split('T').next().unwrap_or("");

    let title = format!(
        "TermLink hub '{hub_name}' ({address}) has been failing fleet-doctor with {class} for {consecutive}+ consecutive runs over >24h"
    );
    let description = format!(
        "Auto-registered by T-1053 under G-019 (framework-blind-to-persistent-failure guard). Hub has failed fleet-doctor with error class '{class}' for {consecutive} consecutive runs since {first_at} (first observed). This indicates either a genuinely broken hub auth state OR stale client credentials that were not refreshed after a hub rotation. Per T-1051 Option D, the heal path is: 1) confirm the hub is intended to be up (termlink remote ping {address} from a trusted anchor), 2) if yes, refresh the client secret (termlink fleet reauth {hub_name} when T-1054 lands, or manually copy /var/lib/termlink/hub.secret via an out-of-band channel for now)."
    );

    let entry = format!(
        "\n- id: {id}\n  type: gap\n  title: \"{title}\"\n  description: \"{description}\"\n  spec_reference: \"T-1051 inception, T-1053 implementation, .context/working/.fleet-failure-state.json\"\n  severity: high\n  trigger_fired: true\n  trigger_event: \"{now_iso}: {consecutive} consecutive fleet-doctor failures on {hub_name} ({address}) with class {class}, first observed {first_at}\"\n  detection_lag_days: \"1\"\n  what_works_now: \"Fleet doctor correctly classifies the error and emits a hint. T-1052 has already recorded an isolated learning. This concern escalates the sustained pattern to Watchtower visibility.\"\n  what_remains: \"Operator must refresh the client's cached secret for this hub. Long-term fix: T-1054 (termlink fleet reauth) lands a one-command heal.\"\n  mitigation_candidate: \"Ship T-1054 (fleet reauth Tier-1) and T-1055 (fleet reauth --bootstrap-from, Tier-2).\"\n  status: watching\n  created: {date_only}\n  last_reviewed: {date_only}\n  related_tasks: [T-1051, T-1052, T-1053, T-1054, T-1055]\n",
        id = id,
        title = title,
        description = description,
        now_iso = now_iso,
        consecutive = consecutive,
        hub_name = hub_name,
        address = address,
        class = class,
        first_at = first_at,
        date_only = date_only,
    );

    let mut new_content = existing;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&entry);
    std::fs::write(&path, new_content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// =========================================================================
// T-1054: fleet reauth — print the copy-pasteable heal incantation for a hub
// profile. Tier-1 only (no automation / no SSH). `--bootstrap-from` lands in
// T-1055. The trust anchor MUST be out-of-band — we explicitly label it.
// =========================================================================

/// Render the heal plan for a single hub profile. Pure — no IO, no stdout —
/// so it can be unit-tested without a filesystem or a shell.
fn render_fleet_reauth_plan(profile: &str, entry: &crate::config::HubEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TermLink fleet reauth — {profile}\n"));
    out.push_str(&format!("Hub profile:      {profile}\n"));
    out.push_str(&format!("Hub address:      {}\n", entry.address));
    match (&entry.secret_file, &entry.secret) {
        (Some(path), _) => {
            out.push_str(&format!("Secret source:    file → {path}\n"));
        }
        (None, Some(_)) => {
            out.push_str("Secret source:    inline in hubs.toml (WARNING — hard to rotate; consider switching to secret_file)\n");
        }
        (None, None) => {
            out.push_str("Secret source:    NONE configured (hubs.toml entry is missing both secret_file and secret)\n");
        }
    }

    out.push('\n');
    out.push_str("Trust anchor:     OUT-OF-BAND required (T-1054 is Tier-1 — no automation).\n");
    out.push_str("                  The channel that delivers the new secret MUST NOT itself depend\n");
    out.push_str("                  on termlink auth (chicken-and-egg). Use SSH, git-pull with signed\n");
    out.push_str("                  commits, physical USB, or another channel whose trust survives a\n");
    out.push_str("                  termlink cert/secret rotation. T-1055 will add `--bootstrap-from`\n");
    out.push_str("                  so the anchor becomes an explicit command argument.\n\n");

    out.push_str("# Heal steps (copy-paste, adjust host as needed):\n\n");
    let host = entry.address.split(':').next().unwrap_or(&entry.address);
    out.push_str(&format!("  # 1. Read the current secret on the hub host ({host}) via an out-of-band channel.\n"));
    out.push_str("  #    Example (requires working SSH to the hub host):\n");
    out.push_str(&format!("  ssh {host} -- sudo cat /var/lib/termlink/hub.secret\n\n"));

    match &entry.secret_file {
        Some(path) => {
            out.push_str("  # 2. Write the hex value from step 1 to the local secret file.\n");
            out.push_str(&format!("  echo \"<paste-the-hex-from-step-1>\" > {path}\n"));
            out.push_str(&format!("  chmod 600 {path}\n\n"));
        }
        None => {
            out.push_str("  # 2. Update the inline secret in ~/.termlink/hubs.toml:\n");
            out.push_str(&format!("  #    [hubs.{profile}]\n"));
            out.push_str("  #    secret = \"<paste-the-hex-from-step-1>\"\n");
            out.push_str("  #    (Consider switching to secret_file = \"/root/.termlink/secrets/<host>.hex\" for cleaner rotation.)\n\n");
        }
    }

    out.push_str("# 3. Verify the heal:\n");
    out.push_str("  termlink fleet doctor\n\n");
    out.push_str(&format!("# Expected: the [PASS] line for {profile} ({}) appears and fleet-doctor reports 0 fail.\n",
        entry.address));
    out.push_str("# If still failing: confirm the hub's hub.secret file is actually the one the hub is\n");
    out.push_str("# serving (hub may be using a different runtime_dir — T-1031 handoff, see `termlink doctor`).\n");

    out
}

/// `termlink fleet reauth <profile> [--bootstrap-from SOURCE]`.
///
/// When `bootstrap_from` is `None`: prints the Tier-1 heal incantation
/// (T-1054 behavior preserved).
///
/// When `bootstrap_from` is `Some("file:PATH" | "ssh:HOST")`: fetches the
/// new secret via the named out-of-band channel, validates it, backs up the
/// existing secret file, and writes the new one at chmod 600 (T-1055, R2).
pub(crate) fn cmd_fleet_reauth(
    profile: &str,
    bootstrap_from: Option<&str>,
    json: bool,
) -> Result<()> {
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        anyhow::bail!(
            "No hubs configured in {}. Add one with: termlink profile add {} <host:port> --secret-file <path>",
            crate::config::hubs_config_path().display(),
            profile,
        );
    }
    let entry = match config.hubs.get(profile) {
        Some(e) => e,
        None => {
            let mut known: Vec<&String> = config.hubs.keys().collect();
            known.sort();
            let known_list: Vec<String> = known.iter().map(|s| (*s).clone()).collect();
            anyhow::bail!(
                "Unknown hub profile '{profile}'. Configured profiles: {}. \
                 Add one with: termlink profile add {profile} <host:port> --secret-file <path>",
                if known_list.is_empty() { "<none>".to_string() } else { known_list.join(", ") },
            );
        }
    };

    // T-1291: resolve `--bootstrap-from auto` to the declared trust anchor on
    // the profile. Operator types one flag instead of remembering the exact
    // OOB incantation per hub. Unknown schemes still hard-error inside
    // `fetch_bootstrap_secret` (back-compat: pre-T-1291 callers explicitly
    // passing `file:` / `ssh:` continue to work unchanged).
    let resolved: Option<String> = match bootstrap_from {
        Some("auto") => match entry.bootstrap_from.as_deref() {
            Some(declared) => Some(declared.to_string()),
            None => anyhow::bail!(
                "Profile '{profile}' has no `bootstrap_from` declared in hubs.toml — \
                 cannot resolve --bootstrap-from auto. Either:\n\
                 \n  \
                 (a) declare it: in ~/.termlink/hubs.toml under [hubs.{profile}] add \
                 `bootstrap_from = \"ssh:<host>\"` or `bootstrap_from = \"file:<path>\"`, then retry; \
                 \n  \
                 (b) pass an explicit source: termlink fleet reauth {profile} --bootstrap-from ssh:<host>"
            ),
        },
        Some(other) => Some(other.to_string()),
        None => None,
    };

    match resolved.as_deref() {
        None => {
            // Tier-1 behavior — print the heal incantation (or its JSON form).
            let plan_text = render_fleet_reauth_plan(profile, entry);
            if json {
                let out = serde_json::json!({
                    "ok": true,
                    "profile": profile,
                    "mode": "plan-only",
                    "source": serde_json::Value::Null,
                    "secret_file": entry.secret_file.clone(),
                    "fingerprint_preview": serde_json::Value::Null,
                    "plan_text": plan_text,
                    "error": serde_json::Value::Null,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{plan_text}");
            }
            Ok(())
        }
        Some(source) => {
            let outcome = cmd_fleet_reauth_bootstrap(profile, entry, source)?;
            if json {
                let out = serde_json::json!({
                    "ok": true,
                    "profile": outcome.profile,
                    "mode": "healed",
                    "source": outcome.source,
                    "secret_file": outcome.secret_file,
                    "fingerprint_preview": outcome.fingerprint_preview,
                    "plan_text": serde_json::Value::Null,
                    "error": serde_json::Value::Null,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print_reauth_outcome_human(&outcome);
            }
            Ok(())
        }
    }
}

/// T-1679: bulk-heal every drifted profile that has declared bootstrap_from.
///
/// Probes every profile in `~/.termlink/hubs.toml` (parallel, 10s per-probe
/// timeout via `probe_cert_with_timeout`), classifies each into
/// match/drift/no-pin/probe-fail (same logic as `cmd_fleet_verify`), then for
/// every `drift` profile with a declared `bootstrap_from` it invokes the
/// Tier-2 bootstrap heal. Per-profile failures do NOT abort the loop —
/// operators want the rest of the fleet healed even if one heal fails.
///
/// Profiles drifted without declared bootstrap_from are skipped with a hint
/// pointing at Tier-1 (`fleet reauth <profile>` to print the incantation, or
/// declare bootstrap_from per profile). R2 (out-of-band trust anchor) is
/// preserved — every heal goes through the existing fetch_bootstrap_secret
/// path with its scheme allow-list.
///
/// Exit code semantics (via std::process::exit):
///   0 — no drift, OR every drifted profile healed cleanly
///   1 — any drifted profile was skipped (no bootstrap_from) or failed to heal
pub(crate) async fn cmd_fleet_reauth_all() -> Result<()> {
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        println!("No hubs configured in {}", crate::config::hubs_config_path().display());
        return Ok(());
    }

    let mut profiles: Vec<(String, String, crate::config::HubEntry)> = config
        .hubs
        .iter()
        .map(|(name, e)| (name.clone(), e.address.clone(), e.clone()))
        .collect();
    profiles.sort_by(|a, b| a.0.cmp(&b.0));

    let store = termlink_session::tofu::KnownHubStore::default_store();
    let probe_timeout = std::time::Duration::from_secs(10);

    let probes: Vec<_> = profiles
        .iter()
        .map(|(name, addr, _)| {
            let name = name.clone();
            let addr = addr.clone();
            tokio::spawn(async move {
                let result = termlink_session::tofu::probe_cert_with_timeout(
                    &addr, probe_timeout,
                ).await;
                (name, addr, result)
            })
        })
        .collect();

    // Collect (profile, status, healed_outcome, note) for the summary table.
    let mut rows: Vec<(String, &'static str, &'static str, String)> = Vec::new();
    let mut any_failure = false;

    for (handle, (name, _addr, entry)) in probes.into_iter().zip(profiles.iter()) {
        let (probe_name, address, probe_result) = match handle.await {
            Ok(t) => t,
            Err(e) => {
                rows.push((name.clone(), "probe-fail", "skip", format!("task panic: {e}")));
                any_failure = true;
                continue;
            }
        };
        debug_assert_eq!(&probe_name, name);

        let pinned = store.get(&address);
        let status = match &probe_result {
            Ok((_, wire)) => match &pinned {
                Some(pin) if pin == wire => "match",
                Some(_) => "drift",
                None => "no-pin",
            },
            Err(_) => "probe-fail",
        };

        match status {
            "match" => {
                rows.push((name.clone(), status, "n/a", "pin matches wire".into()));
            }
            "no-pin" => {
                rows.push((name.clone(), status, "n/a", "no entry in known_hubs".into()));
            }
            "probe-fail" => {
                let err = match probe_result {
                    Err(e) => e,
                    _ => "unreachable".into(),
                };
                rows.push((name.clone(), status, "skip", err));
            }
            "drift" => {
                // The interesting branch — heal if we have a declared anchor.
                match entry.bootstrap_from.as_deref() {
                    Some(source) => {
                        match cmd_fleet_reauth_bootstrap(name, entry, source) {
                            Ok(outcome) => {
                                print_reauth_outcome_human(&outcome);
                                rows.push((
                                    name.clone(),
                                    status,
                                    "healed",
                                    format!("via {source}"),
                                ));
                            }
                            Err(e) => {
                                any_failure = true;
                                rows.push((
                                    name.clone(),
                                    status,
                                    "failed",
                                    format!("{e}"),
                                ));
                            }
                        }
                    }
                    None => {
                        any_failure = true;
                        rows.push((
                            name.clone(),
                            status,
                            "skip",
                            "no bootstrap_from declared — run `termlink fleet reauth <profile>` for Tier-1 incantation".into(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    // Render summary table.
    println!();
    println!("{:<24} {:<11} {:<8} NOTE", "PROFILE", "STATUS", "HEALED?");
    println!("{}", "-".repeat(80));
    for (name, status, healed, note) in &rows {
        println!("{:<24} {:<11} {:<8} {}", name, status, healed, note);
    }
    let drifted = rows.iter().filter(|r| r.1 == "drift").count();
    let healed = rows.iter().filter(|r| r.2 == "healed").count();
    println!();
    println!(
        "Summary: {drifted} drifted, {healed} healed{}",
        if any_failure {
            " (some skipped or failed — exit 1)"
        } else {
            ""
        }
    );

    if any_failure {
        std::process::exit(1);
    }
    Ok(())
}

/// T-1660: fleet-wide TLS pin verification.
///
/// For every profile in `~/.termlink/hubs.toml`, probe the hub's wire
/// fingerprint and compare against the entry in `KnownHubStore`. Pure
/// read-only — no auth, no mutation. Cron-friendly with a fleet-rollup
/// exit code.
///
/// Verdict precedence (drift dominates):
///   match     — every reachable hub matches its pin
///   drift     — at least one hub rotated (heal required)
///   probe-fail — at least one hub unreachable / TLS error (no drift)
///   no-pin    — at least one hub not in KnownHubStore (no drift/probe-fail)
///
/// Exit code mapping: match=0, drift=1, no-pin=2, probe-fail=3.
/// `--exit-on-drift-only` collapses 2/3 to 0 (only page on actual rotation).
pub(crate) async fn cmd_fleet_verify(json: bool, exit_on_drift_only: bool) -> Result<()> {
    let config = crate::config::load_hubs_config();

    // Stable ordering — operators rely on `fleet verify` output being
    // diff-friendly across runs.
    let mut profiles: Vec<(String, String)> = config
        .hubs
        .iter()
        .map(|(name, e)| (name.clone(), e.address.clone()))
        .collect();
    profiles.sort_by(|a, b| a.0.cmp(&b.0));

    if profiles.is_empty() {
        if json {
            println!("{}", serde_json::json!({
                "verdict": "match",
                "profiles": [],
                "note": "no hubs configured",
            }));
        } else {
            println!("No hubs configured in {}", crate::config::hubs_config_path().display());
            println!("  Add one with: termlink profile add <name> <host:port> --secret-file <path>");
        }
        return Ok(());
    }

    let store = termlink_session::tofu::KnownHubStore::default_store();

    // Probe in parallel, bounded per-probe to 10s via T-1675's
    // probe_cert_with_timeout primitive. Without the bound a single
    // unreachable hub stretches the slowest-probe to the OS TCP retry
    // budget (30-60+s) and gates the entire fleet sweep on it. For 3-5
    // reachable hubs this is ~one round-trip total. The fixed 10s is the
    // same default used by `fleet doctor --timeout` — `fleet verify` has
    // no CLI flag to tune it, so we wire the same default here.
    let probe_timeout = std::time::Duration::from_secs(10);
    let probes: Vec<_> = profiles
        .iter()
        .map(|(name, addr)| {
            let name = name.clone();
            let addr = addr.clone();
            tokio::spawn(async move {
                let result = termlink_session::tofu::probe_cert_with_timeout(
                    &addr, probe_timeout,
                ).await;
                (name, addr, result)
            })
        })
        .collect();

    #[derive(serde::Serialize)]
    struct ProfileResult {
        name: String,
        address: String,
        status: &'static str,
        wire: Option<String>,
        pinned: Option<String>,
        error: Option<String>,
    }

    let mut results: Vec<ProfileResult> = Vec::with_capacity(profiles.len());
    for handle in probes {
        let (name, address, probe_result) = match handle.await {
            Ok(t) => t,
            Err(e) => {
                results.push(ProfileResult {
                    name: "<join-error>".to_string(),
                    address: "<unknown>".to_string(),
                    status: "probe-fail",
                    wire: None,
                    pinned: None,
                    error: Some(format!("task panic: {e}")),
                });
                continue;
            }
        };
        let pinned = store.get(&address);
        let (status, wire, error) = match probe_result {
            Ok((_, wire)) => match &pinned {
                Some(pin) if pin == &wire => ("match", Some(wire), None),
                Some(_) => ("drift", Some(wire), None),
                None => ("no-pin", Some(wire), None),
            },
            Err(e) => ("probe-fail", None, Some(e)),
        };
        results.push(ProfileResult { name, address, status, wire, pinned, error });
    }

    // Fleet rollup — drift > probe-fail > no-pin > match.
    let any_drift = results.iter().any(|r| r.status == "drift");
    let any_probe_fail = results.iter().any(|r| r.status == "probe-fail");
    let any_no_pin = results.iter().any(|r| r.status == "no-pin");
    let verdict = if any_drift {
        "drift"
    } else if any_probe_fail {
        "probe-fail"
    } else if any_no_pin {
        "no-pin"
    } else {
        "match"
    };

    if json {
        println!("{}", serde_json::to_string(&serde_json::json!({
            "verdict": verdict,
            "profiles": results,
        }))?);
    } else {
        println!("{:<24} {:<28} STATUS", "PROFILE", "ADDRESS");
        println!("{}", "-".repeat(72));
        for r in &results {
            let note = match r.status {
                "match" => "pin matches wire".to_string(),
                "drift" => "ROTATED — heal required".to_string(),
                "no-pin" => "no entry in known_hubs".to_string(),
                "probe-fail" => r.error.clone().unwrap_or_else(|| "unreachable".to_string()),
                _ => String::new(),
            };
            println!("{:<24} {:<28} {:<11} {}", r.name, r.address, r.status, note);
        }
        println!();
        println!("Fleet verdict: {}", verdict);
        if verdict == "drift" {
            println!();
            println!("  Heal drifted hubs: termlink fleet reauth <profile> --bootstrap-from auto");
            println!("  Then re-pin:       termlink tofu clear <address>");
        }
    }

    let exit = match verdict {
        "match" => 0,
        "drift" => 1,
        "no-pin" => if exit_on_drift_only { 0 } else { 2 },
        "probe-fail" => if exit_on_drift_only { 0 } else { 3 },
        _ => 0,
    };
    if exit != 0 {
        std::process::exit(exit);
    }
    Ok(())
}

/// T-1688: per-profile preflight classification. Pure function, exported for tests.
/// Reuses the live heal path's fetch + validate helpers so the check exercises the
/// exact same code as `--auto-heal` would.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BootstrapCheckStatus {
    Ok,
    NoAnchor,
    FetchFail(String),
    InvalidFormat(String),
}

impl BootstrapCheckStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NoAnchor => "no-anchor",
            Self::FetchFail(_) => "fetch-fail",
            Self::InvalidFormat(_) => "invalid-format",
        }
    }
    fn error(&self) -> Option<&str> {
        match self {
            Self::FetchFail(msg) | Self::InvalidFormat(msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

fn classify_bootstrap_check(bootstrap_from: Option<&str>) -> BootstrapCheckStatus {
    let Some(source) = bootstrap_from else {
        return BootstrapCheckStatus::NoAnchor;
    };
    let raw = match fetch_bootstrap_secret(source) {
        Ok(r) => r,
        Err(e) => return BootstrapCheckStatus::FetchFail(format!("{e:#}")),
    };
    match normalize_and_validate_secret_hex(&raw) {
        Ok(_) => BootstrapCheckStatus::Ok,
        Err(e) => BootstrapCheckStatus::InvalidFormat(format!("{e:#}")),
    }
}

/// T-1688: roll up per-profile statuses into an exit code.
/// - 0 = no fetch-fail and no invalid-format
/// - 1 = any fetch-fail or invalid-format
/// - 2 = `--all` and no profile declares `bootstrap_from` at all
fn bootstrap_check_exit_code(statuses: &[BootstrapCheckStatus], all_mode: bool) -> i32 {
    let any_fail = statuses
        .iter()
        .any(|s| matches!(s, BootstrapCheckStatus::FetchFail(_) | BootstrapCheckStatus::InvalidFormat(_)));
    if any_fail {
        return 1;
    }
    if all_mode {
        let any_declared = statuses
            .iter()
            .any(|s| !matches!(s, BootstrapCheckStatus::NoAnchor));
        if !any_declared {
            return 2;
        }
    }
    0
}

/// T-1688: roll up statuses into a single verdict word for JSON consumers.
fn bootstrap_check_verdict(statuses: &[BootstrapCheckStatus], all_mode: bool) -> &'static str {
    if statuses
        .iter()
        .any(|s| matches!(s, BootstrapCheckStatus::FetchFail(_) | BootstrapCheckStatus::InvalidFormat(_)))
    {
        return "fail";
    }
    let any_declared = statuses
        .iter()
        .any(|s| !matches!(s, BootstrapCheckStatus::NoAnchor));
    if !any_declared {
        return if all_mode { "no-anchor" } else { "ok" };
    }
    let any_missing = statuses
        .iter()
        .any(|s| matches!(s, BootstrapCheckStatus::NoAnchor));
    if any_missing { "mixed" } else { "ok" }
}

/// T-1688: preflight-validate declared `bootstrap_from` anchors WITHOUT
/// performing a heal. See CLI doc-comment on FleetSub::BootstrapCheck.
///
/// Either `profile` or `all=true` MUST be provided (validated here, not by clap,
/// because both fields are individually optional in the variant).
pub(crate) fn cmd_fleet_bootstrap_check(
    profile: Option<&str>,
    all: bool,
    json: bool,
) -> Result<()> {
    if profile.is_none() && !all {
        anyhow::bail!(
            "fleet bootstrap-check: either <profile> or --all must be given\n  e.g. termlink fleet bootstrap-check ring20-management\n       termlink fleet bootstrap-check --all"
        );
    }

    let config = crate::config::load_hubs_config();

    // Pick the profile set. Mirrors `fleet verify` ordering convention.
    let mut entries: Vec<(String, crate::config::HubEntry)> = if let Some(name) = profile {
        let entry = config
            .hubs
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no such hub profile: '{name}'"))?;
        vec![(name.to_string(), entry)]
    } else {
        // --all
        config.hubs.iter().map(|(n, e)| (n.clone(), e.clone())).collect()
    };
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    if entries.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "verdict": if all { "no-anchor" } else { "ok" },
                    "profiles": [],
                    "note": "no hubs configured",
                })
            );
        } else {
            println!(
                "No hubs configured in {}",
                crate::config::hubs_config_path().display()
            );
        }
        return Ok(());
    }

    #[derive(serde::Serialize)]
    struct ProfileResult {
        name: String,
        address: String,
        bootstrap_from: Option<String>,
        status: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    }

    let mut statuses: Vec<BootstrapCheckStatus> = Vec::with_capacity(entries.len());
    let mut results: Vec<ProfileResult> = Vec::with_capacity(entries.len());

    for (name, entry) in &entries {
        let status = classify_bootstrap_check(entry.bootstrap_from.as_deref());
        results.push(ProfileResult {
            name: name.clone(),
            address: entry.address.clone(),
            bootstrap_from: entry.bootstrap_from.clone(),
            status: status.as_str(),
            error: status.error().map(|s| s.to_string()),
        });
        statuses.push(status);
    }

    let verdict = bootstrap_check_verdict(&statuses, all);

    if json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "verdict": verdict,
                "profiles": results,
            }))?
        );
    } else {
        println!("{:<24} {:<28} {:<14} STATUS", "PROFILE", "ADDRESS", "ANCHOR");
        println!("{}", "-".repeat(86));
        for r in &results {
            let anchor = r.bootstrap_from.as_deref().unwrap_or("—");
            // Truncate anchor for column-friendly display; full string in JSON.
            let anchor_display: String = if anchor.chars().count() > 14 {
                let mut s: String = anchor.chars().take(11).collect();
                s.push_str("...");
                s
            } else {
                anchor.to_string()
            };
            let note = match r.status {
                "ok" => "fetched + valid hex".to_string(),
                "no-anchor" => "no bootstrap_from declared".to_string(),
                "fetch-fail" => r.error.clone().unwrap_or_else(|| "unknown error".to_string()),
                "invalid-format" => r.error.clone().unwrap_or_else(|| "bad hex".to_string()),
                _ => String::new(),
            };
            println!(
                "{:<24} {:<28} {:<14} {:<14} {}",
                r.name, r.address, anchor_display, r.status, note
            );
        }
        println!();
        println!("Fleet verdict: {}", verdict);
        if verdict == "fail" {
            println!();
            println!(
                "  Fix declarations in {} so each `bootstrap_from` channel returns 64-char hex.",
                crate::config::hubs_config_path().display()
            );
            println!("  Re-test:           termlink fleet bootstrap-check --all");
        } else if verdict == "no-anchor" {
            println!();
            println!("  No profile declares `bootstrap_from`. Add e.g.:");
            println!("    [hubs.<name>]");
            println!("    bootstrap_from = \"ssh:<host>\"   # or \"file:<path>\"");
        }
    }

    let exit = bootstrap_check_exit_code(&statuses, all);
    if exit != 0 {
        std::process::exit(exit);
    }
    Ok(())
}

/// T-1820: classifier for a single `~/.termlink/secrets/*.hex` file. Pure;
/// takes the inputs an OS stat would produce plus the trimmed content. Returns
/// the human-readable status plus zero-or-more reason strings.
///
/// Status taxonomy:
/// - `ok`            — perms 0o600-equivalent (group/other have no bits set),
///                     content is exactly 64 hex chars
/// - `warn-perms`    — group or other have any bit set (rwx). G-011 incident class.
/// - `warn-format`   — content is not a 64-char lowercase/uppercase hex string.
///                     Likely truncated, corrupt, or wrong file misplaced here.
/// - `warn-drift`    — T-1822: content differs from the authoritative
///                     `<runtime_dir>/hub.secret` passed via `--check-drift`.
///                     Closes G-011 item 1 — the 2026-04-20 PL-041 case where
///                     the giving-end cache had silently rotted after a restart.
/// - `info-orphan`   — perms+format both ok, but no `hubs.toml` profile's
///                     `secret_file` (after `~/`-expansion) matches this file.
///                     Likely leftover from IP renumbering or removed profile.
/// - `ok-mirror`     — T-1822: perms+format ok AND content matches authoritative
///                     (`--check-drift` set). Positive confirmation, distinct
///                     from plain `ok` (where no drift check was performed).
///
/// A single file can carry multiple reasons (e.g. warn-perms AND warn-format).
/// Operator-actionable verdict is the highest-severity reason. Order:
/// warn-perms > warn-format > warn-drift > info-orphan > ok-mirror > ok.
pub(crate) fn classify_secret_file(
    mode: u32,
    content_trimmed: &str,
    is_orphan: bool,
    authoritative_hex: Option<&str>,
) -> (String, Vec<String>) {
    let mut reasons: Vec<String> = Vec::new();

    // Perms check (G-011 item 4 — the world-readable 0o644 case).
    if mode & 0o077 != 0 {
        reasons.push(format!("perms 0o{:03o} expose secret to group/world", mode));
    }

    // Format check — must parse as 64 hex chars.
    let hex_ok = content_trimmed.len() == 64
        && content_trimmed.chars().all(|c| c.is_ascii_hexdigit());
    if !hex_ok {
        reasons.push(format!(
            "content not 64-hex-char (got {} chars)",
            content_trimmed.len()
        ));
    }

    let perms_warn = mode & 0o077 != 0;
    let format_warn = !hex_ok;

    // T-1822: drift check. Only meaningful when BOTH files are valid 64-hex —
    // a malformed cache file is already flagged by warn-format; we don't want
    // to double-flag with a drift complaint that's really a format complaint.
    // The authoritative_hex passed in is assumed pre-normalized (trimmed,
    // lowercased) so a byte comparison is sufficient.
    let drift_status = match authoritative_hex {
        Some(auth) if hex_ok && auth.len() == 64 => {
            let own = content_trimmed.to_ascii_lowercase();
            let auth_lc = auth.to_ascii_lowercase();
            if own == auth_lc {
                Some("ok-mirror")
            } else {
                reasons.push(
                    "content differs from authoritative hub.secret (--check-drift)".to_string(),
                );
                Some("warn-drift")
            }
        }
        _ => None,
    };

    // Orphan status (informational only — the cache may be genuinely
    // unused yet, or it may be obsolete; the operator decides).
    if is_orphan && !perms_warn && !format_warn && drift_status != Some("warn-drift") {
        reasons.push("not referenced by any hubs.toml profile".to_string());
    }

    let status = if perms_warn {
        "warn-perms"
    } else if format_warn {
        "warn-format"
    } else if drift_status == Some("warn-drift") {
        "warn-drift"
    } else if is_orphan {
        "info-orphan"
    } else if drift_status == Some("ok-mirror") {
        "ok-mirror"
    } else {
        "ok"
    };
    (status.to_string(), reasons)
}

/// T-1820: scan a directory for `*.hex` files. Pure: takes the dir path and a
/// closure that yields the cross-reference set of currently-referenced files
/// (so unit tests can supply a fixed set). Returns the rows sorted by path.
///
/// T-1822 extension: when `authoritative_hex` is `Some(&str)`, each row's
/// classification additionally compares its content against that authoritative
/// value (mirror/drift verdict). The caller is responsible for pre-trimming
/// + validating the authoritative hex.
///
/// T-1824 extension: when `target_cache` is `Some(&Path)`, drift comparison
/// applies ONLY to the row whose path canonicalizes equal to the target.
/// Every other row receives `None` for authoritative_hex (preserving plain
/// perms/format/orphan verdict). When `target_cache` is `None`, behavior
/// matches the broad-mode T-1822 path (drift-check every row).
fn scan_secrets_dir<F>(
    dir: &std::path::Path,
    is_referenced: F,
    authoritative_hex: Option<&str>,
    target_cache: Option<&std::path::Path>,
) -> Vec<(std::path::PathBuf, u32, u64, String, Vec<String>)>
where
    F: Fn(&std::path::Path) -> bool,
{
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let mut rows: Vec<(std::path::PathBuf, u32, u64, String, Vec<String>)> = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return rows,
    };
    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("hex"))
        .collect();
    paths.sort();

    // T-1824: pre-resolve the target cache path for comparison. Use
    // canonicalize() to handle symlinks, ~/expansion, relative-vs-absolute.
    // Falls back to the raw path on canonicalize failure (e.g. doesn't exist
    // yet — caller still gets the "no target match" signal).
    let target_canonical: Option<std::path::PathBuf> = target_cache.map(|p| {
        std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
    });

    for path in paths {
        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        #[cfg(unix)]
        let mode = metadata.permissions().mode() & 0o777;
        #[cfg(not(unix))]
        let mode: u32 = 0o600; // placeholder — non-unix path skips perms check

        let size = metadata.len();
        let content_trimmed = std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let orphan = !is_referenced(&path);

        // T-1824: only pass authoritative_hex through when this row IS the
        // target (or no target set → broad-mode). Other rows get None →
        // plain T-1820 classifier behavior.
        let pass_auth: Option<&str> = match &target_canonical {
            None => authoritative_hex, // broad mode
            Some(tgt) => {
                let path_canonical =
                    std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                if &path_canonical == tgt {
                    authoritative_hex
                } else {
                    None
                }
            }
        };

        let (status, reasons) = classify_secret_file(mode, &content_trimmed, orphan, pass_auth);
        rows.push((path, mode, size, status, reasons));
    }
    rows
}

/// T-1820: audit `~/.termlink/secrets/*.hex` for security hygiene. Read-only;
/// never authenticates; never contacts a hub. Closes G-011 item 4.
pub(crate) fn cmd_fleet_secrets_audit(
    dir_override: Option<&str>,
    check_drift: Option<&str>,
    target_cache: Option<&str>,
    json: bool,
) -> Result<()> {
    use serde_json::json;

    // T-1824 R1: --target-cache requires --check-drift. Standalone makes no
    // sense (there's nothing to compare against). Exit 2 = usage error.
    if target_cache.is_some() && check_drift.is_none() {
        eprintln!(
            "error: --target-cache requires --check-drift <PATH> (the target needs an authoritative to compare against)"
        );
        std::process::exit(2);
    }

    // Resolve scan directory.
    let dir_path: std::path::PathBuf = if let Some(d) = dir_override {
        expand_secret_file_path(d)
    } else {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME not set; cannot resolve default secrets dir"))?;
        std::path::PathBuf::from(home).join(".termlink").join("secrets")
    };

    // T-1822: load authoritative hub.secret if --check-drift was passed.
    // We audit its perms+format independently, then pass its hex content
    // into the scanner so each cache row gets a mirror/drift verdict.
    // Failure to read/parse becomes a top-level error_message but does NOT
    // abort the scan (operator still sees perms/format/orphan info).
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    let authoritative_info: Option<serde_json::Value>;
    let mut authoritative_hex_owned: Option<String> = None;
    let mut authoritative_error: Option<String> = None;
    match check_drift {
        Some(path_str) => {
            let auth_path = expand_secret_file_path(path_str);
            match std::fs::metadata(&auth_path) {
                Ok(md) => {
                    #[cfg(unix)]
                    let auth_mode = md.permissions().mode() & 0o777;
                    #[cfg(not(unix))]
                    let auth_mode: u32 = 0o600;
                    let auth_size = md.len();
                    let content = std::fs::read_to_string(&auth_path)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    // Classify the authoritative file using its own hex as
                    // self-reference → status will be ok-mirror if perms/format
                    // both clean. The "is_orphan" arg is irrelevant here (the
                    // authoritative file is by construction not in the cache
                    // dir), so pass `false`.
                    let (auth_status, auth_reasons) =
                        classify_secret_file(auth_mode, &content, false, Some(&content));
                    let hex_ok = content.len() == 64
                        && content.chars().all(|c| c.is_ascii_hexdigit());
                    if hex_ok {
                        authoritative_hex_owned = Some(content.to_ascii_lowercase());
                    } else {
                        authoritative_error = Some(format!(
                            "authoritative file content is not 64-char hex (got {} chars); drift-check disabled — perms/format/orphan still run",
                            content.len()
                        ));
                    }
                    authoritative_info = Some(json!({
                        "path": auth_path.display().to_string(),
                        "mode": format!("0o{:03o}", auth_mode),
                        "size": auth_size,
                        "status": auth_status,
                        "reasons": auth_reasons,
                    }));
                }
                Err(e) => {
                    authoritative_error = Some(format!(
                        "cannot read authoritative file {}: {}; drift-check disabled — perms/format/orphan still run",
                        auth_path.display(),
                        e
                    ));
                    authoritative_info = Some(json!({
                        "path": auth_path.display().to_string(),
                        "mode": null,
                        "size": null,
                        "status": "error",
                        "reasons": [authoritative_error.clone().unwrap_or_default()],
                    }));
                }
            }
        }
        None => {
            authoritative_info = None;
        }
    }

    // Build the set of secret_file paths referenced by hubs.toml. Used to
    // classify orphans. Empty config (no profiles) means every hex file is
    // orphan — still useful info.
    let config = crate::config::load_hubs_config();
    let referenced: std::collections::HashSet<std::path::PathBuf> = config
        .hubs
        .values()
        .filter_map(|entry| entry.secret_file.as_deref())
        .map(expand_secret_file_path)
        .collect();
    let is_referenced = |p: &std::path::Path| referenced.contains(p);

    // T-1824: resolve --target-cache once. Used both by the scanner (to
    // narrow drift comparison) and below for the "named target doesn't
    // exist in scan" warning.
    let target_cache_path: Option<std::path::PathBuf> = target_cache.map(expand_secret_file_path);

    let rows = scan_secrets_dir(
        &dir_path,
        is_referenced,
        authoritative_hex_owned.as_deref(),
        target_cache_path.as_deref(),
    );

    // T-1824 R2: if --target-cache was set but the named file isn't in the
    // scan (typo or wrong dir), warn the operator — silent skipping would
    // hide a real misuse.
    let mut target_cache_missing: Option<String> = None;
    if let Some(tgt) = &target_cache_path {
        let tgt_canonical = std::fs::canonicalize(tgt).unwrap_or_else(|_| tgt.clone());
        let found = rows.iter().any(|(p, _, _, _, _)| {
            let p_canon = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            p_canon == tgt_canonical
        });
        if !found {
            target_cache_missing = Some(format!(
                "--target-cache path {} not found in scan dir {} — no drift verdict rendered. Check for typo or wrong --dir.",
                tgt.display(),
                dir_path.display()
            ));
        }
    }

    // Summary counts.
    let mut count_ok = 0u32;
    let mut count_ok_mirror = 0u32;
    let mut count_warn_perms = 0u32;
    let mut count_warn_format = 0u32;
    let mut count_warn_drift = 0u32;
    let mut count_info_orphan = 0u32;
    for (_, _, _, status, _) in &rows {
        match status.as_str() {
            "ok" => count_ok += 1,
            "ok-mirror" => count_ok_mirror += 1,
            "warn-perms" => count_warn_perms += 1,
            "warn-format" => count_warn_format += 1,
            "warn-drift" => count_warn_drift += 1,
            "info-orphan" => count_info_orphan += 1,
            _ => {}
        }
    }
    let total = rows.len() as u32;

    if json {
        let files: Vec<serde_json::Value> = rows
            .iter()
            .map(|(path, mode, size, status, reasons)| {
                json!({
                    "path": path.display().to_string(),
                    "mode": format!("0o{:03o}", mode),
                    "size": size,
                    "status": status,
                    "reasons": reasons,
                })
            })
            .collect();
        let mut envelope = json!({
            "ok": count_warn_perms == 0 && count_warn_format == 0 && count_warn_drift == 0,
            "dir": dir_path.display().to_string(),
            "files": files,
            "summary": {
                "total": total,
                "ok": count_ok,
                "ok_mirror": count_ok_mirror,
                "warn_perms": count_warn_perms,
                "warn_format": count_warn_format,
                "warn_drift": count_warn_drift,
                "info_orphan": count_info_orphan,
            }
        });
        if let Some(obj) = envelope.as_object_mut() {
            if let Some(auth) = authoritative_info {
                obj.insert("authoritative".to_string(), auth);
            }
            if let Some(err) = authoritative_error.as_deref() {
                obj.insert(
                    "authoritative_error".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
            // T-1824: always include target_cache in envelope (null = broad mode).
            obj.insert(
                "target_cache".to_string(),
                target_cache_path
                    .as_ref()
                    .map(|p| serde_json::Value::String(p.display().to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
            if let Some(err) = target_cache_missing.as_deref() {
                obj.insert(
                    "target_cache_error".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
        }
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("# fleet secrets-audit | dir={}", dir_path.display());
        if let Some(auth_v) = &authoritative_info {
            let path = auth_v.get("path").and_then(|v| v.as_str()).unwrap_or("?");
            let status = auth_v.get("status").and_then(|v| v.as_str()).unwrap_or("?");
            let mode = auth_v
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("0o???");
            println!("# authoritative: {status} {mode} {path}");
        }
        if let Some(tgt) = &target_cache_path {
            println!("# target-cache: {}", tgt.display());
        }
        if let Some(err) = authoritative_error.as_deref() {
            println!("# WARNING: {err}");
        }
        if let Some(err) = target_cache_missing.as_deref() {
            println!("# WARNING: {err}");
        }
        if rows.is_empty() {
            println!("(no .hex files found — directory missing or empty)");
        } else {
            for (path, mode, _size, status, reasons) in &rows {
                let suffix = if reasons.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", reasons.join("; "))
                };
                println!("{:<12} 0o{:03o} {}{}", status, mode, path.display(), suffix);
            }
            if check_drift.is_some() {
                println!(
                    "\n{} file(s): {} ok, {} ok-mirror, {} warn-perms, {} warn-format, {} warn-drift, {} info-orphan",
                    total,
                    count_ok,
                    count_ok_mirror,
                    count_warn_perms,
                    count_warn_format,
                    count_warn_drift,
                    count_info_orphan
                );
            } else {
                println!(
                    "\n{} file(s): {} ok, {} warn-perms, {} warn-format, {} info-orphan",
                    total, count_ok, count_warn_perms, count_warn_format, count_info_orphan
                );
            }
        }
    }

    // Exit code: 1 if any actionable warning (perms, format, drift) OR
    // target-cache typo (operator misuse). 0 otherwise (orphan-only is
    // informational; ok-mirror is positive).
    if count_warn_perms > 0
        || count_warn_format > 0
        || count_warn_drift > 0
        || target_cache_missing.is_some()
    {
        std::process::exit(1);
    }
    Ok(())
}

/// T-1055 Tier-2 heal: fetch the new secret via the named out-of-band source,
/// validate it, back up the existing file, and write the new one.
/// T-1728: structured outcome from Tier-2 heal — lets the CLI render either
/// human eprintln or JSON, and lets the MCP wrapper (`termlink_fleet_reauth`)
/// emit a deterministic JSON shape without re-doing the heal.
pub(crate) struct ReauthBootstrapOutcome {
    pub profile: String,
    pub address: String,
    pub secret_file: String,
    pub source: String,
    pub fingerprint_preview: String,
}

pub(crate) fn cmd_fleet_reauth_bootstrap(
    profile: &str,
    entry: &crate::config::HubEntry,
    source: &str,
) -> Result<ReauthBootstrapOutcome> {
    let secret_file = match &entry.secret_file {
        Some(p) => p.clone(),
        None => anyhow::bail!(
            "Profile '{profile}' uses an inline secret (no secret_file). \
             The --bootstrap-from heal path writes to secret_file only. \
             Migrate first: in ~/.termlink/hubs.toml change [hubs.{profile}] to use \
             secret_file = \"/root/.termlink/secrets/<host>.hex\" instead of secret = ..., then retry."
        ),
    };

    // Resolve the bootstrap source to the hex value of the new secret.
    let raw = fetch_bootstrap_secret(source)
        .with_context(|| format!("failed to fetch new secret via {source}"))?;
    let hex = normalize_and_validate_secret_hex(&raw)
        .with_context(|| format!("new secret from {source} is not valid 64-char hex"))?;

    // Persist: back up existing, then atomically write the new file.
    let target = std::path::PathBuf::from(&secret_file);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create parent dir for secret file: {}", parent.display())
        })?;
    }
    if target.exists() {
        let backup = target.with_extension("hex.bak");
        std::fs::copy(&target, &backup).with_context(|| {
            format!("failed to back up existing secret to {}", backup.display())
        })?;
    }
    write_secret_file(&target, &hex)?;

    // 12-char preview keeps full secret out of terminal history.
    let preview: String = hex.chars().take(12).collect();
    Ok(ReauthBootstrapOutcome {
        profile: profile.to_string(),
        address: entry.address.clone(),
        secret_file,
        source: source.to_string(),
        fingerprint_preview: preview,
    })
}

/// Render the human-readable success summary for a Tier-2 heal. CLI calls
/// this; MCP does not (it returns JSON via cmd_fleet_reauth's `--json` path).
fn print_reauth_outcome_human(outcome: &ReauthBootstrapOutcome) {
    eprintln!("[OK] heal complete");
    eprintln!("     profile:      {}", outcome.profile);
    eprintln!("     address:      {}", outcome.address);
    eprintln!("     secret file:  {}", outcome.secret_file);
    eprintln!("     bootstrap:    {}", outcome.source);
    eprintln!(
        "     new secret:   {}… (first 12 of 64 hex chars)",
        outcome.fingerprint_preview
    );
    eprintln!();
    eprintln!("Verify with: termlink fleet doctor");
}

/// Read the hex secret from the named bootstrap source.
/// Scheme → behavior:
///   file:<path>    → read file contents (UTF-8)
///   ssh:<host>     → spawn `ssh <host> -- sudo cat /var/lib/termlink/hub.secret`
/// Any other prefix → error listing the accepted forms.
fn fetch_bootstrap_secret(source: &str) -> Result<String> {
    if let Some(path) = source.strip_prefix("file:") {
        if path.is_empty() {
            anyhow::bail!("file: source requires a path (e.g. file:/tmp/new-secret.hex)");
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read bootstrap file: {path}"))?;
        return Ok(content);
    }
    if let Some(host) = source.strip_prefix("ssh:") {
        if host.is_empty() {
            anyhow::bail!("ssh: source requires a host (e.g. ssh:hub.example.com)");
        }
        // Deliberately fixed remote path — matches the hub's default
        // runtime_dir for systemd-run deployments. Bespoke paths are an
        // explicit non-goal (see task scope).
        let output = std::process::Command::new("ssh")
            .args([host, "--", "sudo", "cat", "/var/lib/termlink/hub.secret"])
            .output()
            .with_context(|| format!("failed to invoke ssh for host '{host}'"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "ssh {host} -- sudo cat /var/lib/termlink/hub.secret exited with status {:?}: {}",
                output.status.code(),
                stderr.trim(),
            );
        }
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    anyhow::bail!(
        "Unknown bootstrap source '{source}'. Accepted forms:\n  file:<path>\n  ssh:<host>\n\
         Unsupported by design: command:<cmd> (arbitrary shell — reserved for a later task)."
    )
}

/// Trim and validate that `raw` is 64 hex chars (a 32-byte HMAC secret).
fn normalize_and_validate_secret_hex(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.len() != 64 {
        anyhow::bail!(
            "expected 64 hex characters, got {} characters (trimmed)",
            trimmed.len()
        );
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("bootstrap value contains non-hex characters");
    }
    Ok(trimmed.to_string())
}

/// Write a secret file at chmod 600. Creates the file if missing; overwrites
/// existing content atomically via a `<path>.tmp` → rename dance.
fn write_secret_file(path: &std::path::Path, hex: &str) -> Result<()> {
    let tmp = path.with_extension("hex.tmp");
    // Write content.
    std::fs::write(&tmp, hex)
        .with_context(|| format!("failed to write temp secret file: {}", tmp.display()))?;
    // Tighten perms on the temp file BEFORE the rename so there is no
    // window in which the final path exists with loose perms.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&tmp, perm).with_context(|| {
            format!("failed to chmod 600 on temp secret file: {}", tmp.display())
        })?;
    }
    std::fs::rename(&tmp, path).with_context(|| {
        format!("failed to promote {} → {}", tmp.display(), path.display())
    })?;
    Ok(())
}

pub(crate) async fn cmd_remote_doctor(
    conn: &RemoteConn<'_>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_doctor_inner(conn, json)).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!("Timeout after {}s waiting for remote doctor RPC", timeout_secs),
    }
}

async fn cmd_remote_doctor_inner(
    conn: &RemoteConn<'_>,
    json: bool,
) -> Result<()> {
    use serde_json::json;

    let mut checks: Vec<serde_json::Value> = Vec::new();
    let mut pass_count: u32 = 0;
    let mut warn_count: u32 = 0;
    let mut fail_count: u32 = 0;

    macro_rules! check {
        ($name:expr, pass, $msg:expr) => {{
            pass_count += 1;
            checks.push(json!({"check": $name, "status": "pass", "message": $msg}));
            if !json { eprintln!("  [PASS] {}: {}", $name, $msg); }
        }};
        ($name:expr, warn, $msg:expr) => {{
            warn_count += 1;
            checks.push(json!({"check": $name, "status": "warn", "message": $msg}));
            if !json { eprintln!("  [WARN] {}: {}", $name, $msg); }
        }};
        ($name:expr, fail, $msg:expr) => {{
            fail_count += 1;
            checks.push(json!({"check": $name, "status": "fail", "message": $msg}));
            if !json { eprintln!("  [FAIL] {}: {}", $name, $msg); }
        }};
    }

    if !json {
        eprintln!("Remote doctor: {}", conn.hub);
    }

    // 1. Connectivity — connect + auth
    let connect_start = std::time::Instant::now();
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => {
            let latency = connect_start.elapsed().as_millis();
            check!("connectivity", pass, format!("connected in {}ms", latency));
            c
        }
        Err(e) => {
            check!("connectivity", fail, format!("cannot connect: {}", e));
            if json {
                println!("{}", json!({
                    "ok": false,
                    "hub": conn.hub,
                    "checks": checks,
                    "summary": {"pass": pass_count, "warn": warn_count, "fail": fail_count}
                }));
            }
            return Ok(());
        }
    };

    // 2. Session count via discover (session.list is a per-session method, not hub-level)
    match rpc_client.call("session.discover", json!("doc-sd"), json!({})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            if let Some(sessions) = r.result["sessions"].as_array() {
                let count = sessions.len();
                let names: Vec<&str> = sessions.iter()
                    .filter_map(|s| s["display_name"].as_str())
                    .collect();
                if count == 0 {
                    check!("sessions", warn, "no sessions registered");
                } else {
                    check!("sessions", pass, format!("{} session(s): {}", count, names.join(", ")));
                }
            } else {
                check!("sessions", warn, "unexpected response format");
            }
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            check!("sessions", warn, format!("session.discover error: {}", e.error.message));
        }
        Err(e) => {
            check!("sessions", warn, format!("session.discover RPC failed: {}", e));
        }
    }

    // 3. Inbox status (T-1229g: channel-aware, surfaces offline targets)
    {
        let cache = termlink_session::hub_capabilities::shared_cache();
        let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
        match termlink_session::inbox_channel::status_via_channel_with_client(
            &mut rpc_client, conn.hub, cache, &mut ctx,
        ).await {
            Ok(status) => {
                if status.total_transfers == 0 {
                    check!("inbox", pass, "no pending transfers");
                } else {
                    check!("inbox", warn, format!(
                        "{} pending transfer(s) for {} target(s)",
                        status.total_transfers,
                        status.targets.len()
                    ));
                }
            }
            Err(e) => {
                check!("inbox", warn, format!("inbox.status error: {}", e));
            }
        }
    }

    // Output
    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": fail_count == 0,
            "hub": conn.hub,
            "checks": checks,
            "summary": {"pass": pass_count, "warn": warn_count, "fail": fail_count}
        }))?);
    } else {
        eprintln!("\n  Summary: {} pass, {} warn, {} fail", pass_count, warn_count, fail_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SECRET_HEX: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn connect_rejects_hub_without_colon() {
        let err = connect_remote_hub("myhost", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("host:port"),
            "expected host:port hint, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_hub_with_extra_colons() {
        let err = connect_remote_hub("a:b:c", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("host:port"),
            "expected host:port hint, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_non_numeric_port() {
        let err = connect_remote_hub("host:abc", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("Invalid port"),
            "expected Invalid port, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_missing_secret() {
        let err = connect_remote_hub("host:9100", None, None, "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("--secret-file or --secret"),
            "expected secret-required message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_short_secret() {
        let err = connect_remote_hub("host:9100", None, Some("abcd"), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("64 hex characters"),
            "expected 64-hex-char message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_non_hex_secret() {
        let bad = "z".repeat(64);
        let err = connect_remote_hub("host:9100", None, Some(&bad), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("invalid hex"),
            "expected invalid-hex message, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_unknown_scope() {
        let err = connect_remote_hub("host:9100", None, Some(VALID_SECRET_HEX), "superuser")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("Invalid scope"),
            "expected Invalid scope message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_missing_secret_file() {
        let err = connect_remote_hub(
            "host:9100",
            Some("/nonexistent/path/that/should/not/exist"),
            None,
            "control",
        )
        .await
        .err()
        .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("Secret file not found"),
            "expected secret-file-not-found message, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_accepts_all_four_permission_scopes() {
        for scope in ["observe", "interact", "control", "execute"] {
            let err = connect_remote_hub("127.0.0.1:1", None, Some(VALID_SECRET_HEX), scope)
                .await
                .err()
            .expect("expected validation error");
            let msg = format!("{err:#}");
            assert!(
                !msg.contains("Invalid scope"),
                "scope {scope} was rejected: {msg}"
            );
            assert!(
                !msg.contains("64 hex characters"),
                "scope {scope} failed at secret length: {msg}"
            );
        }
    }

    // -------------------------------------------------------------------
    // T-1614: is_rfc1918 helper for fleet-status TIMEOUT classification
    // -------------------------------------------------------------------

    #[test]
    fn is_rfc1918_matches_canonical_ranges() {
        // 10.0.0.0/8
        assert!(is_rfc1918("10.0.0.1"));
        assert!(is_rfc1918("10.255.255.255"));
        // 192.168.0.0/16
        assert!(is_rfc1918("192.168.1.1"));
        assert!(is_rfc1918("192.168.255.255"));
        // 172.16.0.0/12 — boundaries
        assert!(is_rfc1918("172.16.0.1"));
        assert!(is_rfc1918("172.31.255.255"));
        assert!(is_rfc1918("172.20.10.5"));
    }

    #[test]
    fn is_rfc1918_rejects_outside_ranges() {
        // Public IPs
        assert!(!is_rfc1918("8.8.8.8"));
        assert!(!is_rfc1918("1.1.1.1"));
        // 172.x boundaries (just outside 16-31)
        assert!(!is_rfc1918("172.15.0.1"));
        assert!(!is_rfc1918("172.32.0.1"));
        // RFC5737 documentation ranges (must NOT match RFC1918)
        assert!(!is_rfc1918("192.0.2.1"));
        assert!(!is_rfc1918("198.51.100.1"));
        assert!(!is_rfc1918("203.0.113.1"));
        // Loopback (handled separately in TIMEOUT classifier)
        assert!(!is_rfc1918("127.0.0.1"));
        // Garbage / hostnames
        assert!(!is_rfc1918("localhost"));
        assert!(!is_rfc1918("not.an.ip"));
        assert!(!is_rfc1918(""));
    }

    // -------------------------------------------------------------------
    // T-1052: fleet-doctor auto-register learning on auth-mismatch
    // -------------------------------------------------------------------

    #[test]
    fn fleet_learning_classifies_auth_errors() {
        // Known auth-mismatch patterns
        assert_eq!(auth_mismatch_class("Token validation failed: invalid signature"), Some("auth-mismatch"));
        assert_eq!(auth_mismatch_class("rpc error: invalid signature"), Some("auth-mismatch"));
        // Known TOFU patterns
        assert_eq!(auth_mismatch_class("TOFU VIOLATION: fingerprint changed"), Some("tofu-violation"));
        assert_eq!(auth_mismatch_class("fingerprint changed unexpectedly"), Some("tofu-violation"));
        // Unrelated errors must be None (don't spam learnings)
        assert_eq!(auth_mismatch_class("Connection refused"), None);
        assert_eq!(auth_mismatch_class("Secret file not found"), None);
        assert_eq!(auth_mismatch_class("TLS handshake failed"), None);
    }

    // -------------------------------------------------------------------
    // T-1682: pin the watch parser's conn-state remapping. Without this,
    // a future refactor of cmd_fleet_doctor's status vocabulary can
    // silently regress T-1681's auto-heal gate on conn=auth-mismatch
    // (which is itself the PL-162 secret-only-rotation closure).
    // -------------------------------------------------------------------
    #[test]
    fn derive_watch_conn_classifies_auth_mismatch() {
        // status=error + auth-class error message → "auth-mismatch"
        let hub = serde_json::json!({
            "hub": "ring20-management",
            "status": "error",
            "error": "Token validation failed: invalid signature",
        });
        assert_eq!(derive_watch_conn(&hub), "auth-mismatch");
    }

    #[test]
    fn derive_watch_conn_classifies_tofu_violation() {
        // status=error + tofu-class error message → "tofu-violation"
        let hub = serde_json::json!({
            "hub": "ring20-dashboard",
            "status": "error",
            "error": "TOFU VIOLATION: Hub fingerprint changed",
        });
        assert_eq!(derive_watch_conn(&hub), "tofu-violation");
    }

    #[test]
    fn derive_watch_conn_passes_through_non_error_status() {
        for raw in &["ok", "timeout", "unknown"] {
            let hub = serde_json::json!({"hub": "h", "status": raw});
            assert_eq!(derive_watch_conn(&hub), *raw);
        }
    }

    #[test]
    fn derive_watch_conn_falls_back_to_error_when_unclassified() {
        // status=error + generic error → keep "error" (no false auth claim)
        let hub = serde_json::json!({
            "hub": "h",
            "status": "error",
            "error": "Connection refused",
        });
        assert_eq!(derive_watch_conn(&hub), "error");
    }

    #[test]
    fn derive_watch_conn_handles_missing_fields() {
        // No status at all → "unknown"; no error msg under error status → "error"
        assert_eq!(derive_watch_conn(&serde_json::json!({})), "unknown");
        assert_eq!(
            derive_watch_conn(&serde_json::json!({"status": "error"})),
            "error"
        );
    }

    // -------------------------------------------------------------------
    // T-1181: fleet-doctor classify_fleet_error must see the full anyhow
    // chain (not just the top-level .context wrapper). Pins the behaviour
    // that a "Cannot connect…: TOFU VIOLATION…" composed message lands in
    // the TOFU branch, not the generic fallback.
    // -------------------------------------------------------------------
    #[test]
    fn classify_fleet_error_matches_wrapped_tofu_cause() {
        // Composed form produced by format!("{:#}", e) when the outer
        // context is "Cannot connect to … — is the hub running?" and the
        // inner cause carries "TOFU VIOLATION".
        let msg = "Cannot connect to 192.168.10.102:9100 — is the hub running?: unexpected error: TOFU VIOLATION: Hub 192.168.10.102:9100 fingerprint changed";
        let hint = classify_fleet_error(msg, "192.168.10.102:9100");
        assert!(
            hint.contains("termlink tofu clear 192.168.10.102:9100"),
            "expected actionable TOFU hint, got: {hint}"
        );
    }

    #[test]
    fn classify_fleet_error_matches_wrapped_auth_cause() {
        // Auth-mismatch wrapped under a connect-context outer.
        let msg = "Cannot connect to 10.0.0.1:9100 — is the hub running?: Token validation failed: invalid signature";
        let hint = classify_fleet_error(msg, "10.0.0.1:9100");
        assert!(
            hint.contains("Secret mismatch"),
            "expected auth-mismatch hint, got: {hint}"
        );
    }

    /// Reuse the crate-wide test env lock. Any test in this binary that
    /// mutates CWD or HOME must lock through this to avoid racing with
    /// sibling tests (e.g. `config::tests::save_and_load_hubs_config`).
    use crate::test_env_lock::ENV_LOCK as CWD_LOCK;

    /// Create an isolated tempdir that looks like a framework project, cd into it,
    /// run the closure, then restore CWD. Returns whatever the closure returned.
    ///
    /// Robust against a pre-set broken CWD: some unrelated tests in this crate
    /// (e.g. dispatch::isolate_rejects_non_git_dir) `set_current_dir` into a
    /// `tempfile::tempdir` that auto-deletes at function exit, leaving CWD
    /// pointing at a removed directory. If we called `current_dir()` after
    /// that, it would ENOENT. So we always anchor back to "/" after each run,
    /// and never try to preserve the caller's prior CWD.
    fn with_framework_cwd<R>(f: impl FnOnce(&std::path::Path) -> R) -> R {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Anchor CWD to a known-good path before doing anything that would
        // observe the current dir.
        std::env::set_current_dir("/").expect("cd to /");

        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1052-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".context/project")).expect("create .context/project");
        std::fs::create_dir_all(tmp.join(".context/working")).expect("create .context/working");
        std::fs::write(
            tmp.join(".context/project/learnings.yaml"),
            "# Project Learnings\nlearnings:\n",
        )
        .expect("seed learnings.yaml");
        std::fs::write(
            tmp.join(".context/project/concerns.yaml"),
            "# Concerns Register\nconcerns:\n",
        )
        .expect("seed concerns.yaml");

        std::env::set_current_dir(&tmp).expect("cd into tmp");

        // Also isolate HOME so KnownHubStore doesn't touch the real ~/.termlink.
        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&tmp)));

        // Restore CWD to a known-good anchor BEFORE removing tmp — otherwise
        // the remove would leave CWD dangling.
        std::env::set_current_dir("/").expect("restore cwd to /");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        match result {
            Ok(v) => v,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    #[test]
    fn fleet_learning_writes_entry_on_auth_mismatch() {
        with_framework_cwd(|tmp| {
            let err_msg = "rpc error: Token validation failed: invalid signature";
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("record learning");

            let learnings = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            assert!(learnings.contains("PL-001"), "new PL-XXX id not allocated: {learnings}");
            assert!(learnings.contains("ring20-management"), "hub name missing from learning");
            assert!(learnings.contains("10.0.0.1:9100"), "hub address missing from learning");
            assert!(learnings.contains("auth-mismatch"), "class missing from learning");
            assert!(learnings.contains("hub_fingerprint="), "fingerprint key missing from learning");
            assert!(learnings.contains("date_observed="), "date_observed key missing from learning");
            assert!(learnings.contains("source: T-1052"), "source T-1052 missing from entry");
            assert!(learnings.contains("task: T-1051"), "task T-1051 missing from entry");

            // Dedupe marker written.
            assert!(
                tmp.join(".context/working/.fleet-learning-ring20-management").exists(),
                "dedupe marker not created",
            );
        });
    }

    #[test]
    fn fleet_learning_skips_unrelated_errors() {
        with_framework_cwd(|tmp| {
            maybe_record_auth_mismatch_learning("somehub", "127.0.0.1:9100", "Connection refused")
                .expect("call succeeds");
            let learnings = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            // No PL-XXX entry should have been written.
            assert!(!learnings.contains("PL-001"), "connection-refused must not create a learning: {learnings}");
            assert!(!tmp.join(".context/working/.fleet-learning-somehub").exists(),
                "no marker should exist for unrelated errors");
        });
    }

    #[test]
    fn fleet_learning_dedupes_within_24h_same_fingerprint() {
        with_framework_cwd(|tmp| {
            let err_msg = "Token validation failed: invalid signature";
            // First record
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("first record");
            let after_first = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            let count_first = after_first.matches("- id: PL-").count();

            // Second call with same inputs — fingerprint unchanged (both "unknown"),
            // marker <24h old → should dedupe.
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("second record");
            let after_second = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            let count_second = after_second.matches("- id: PL-").count();
            assert_eq!(
                count_first, count_second,
                "duplicate call within 24h must not add a second entry: first={count_first} second={count_second}",
            );
        });
    }

    // -------------------------------------------------------------------
    // T-1053: fleet-doctor concern auto-registration
    // -------------------------------------------------------------------

    #[test]
    fn parse_iso8601_utc_roundtrips_now() {
        let s = utc_iso8601_now();
        let secs = parse_iso8601_utc(&s).expect("parse our own output");
        // now_unix_secs() vs parsed should be within a few seconds.
        let now = now_unix_secs();
        let delta = secs.abs_diff(now);
        assert!(delta < 3, "parse roundtrip off by {delta}s: got {secs} vs now {now} (input {s})");
    }

    #[test]
    fn parse_iso8601_utc_rejects_malformed() {
        assert!(parse_iso8601_utc("").is_none());
        assert!(parse_iso8601_utc("2026-04-14").is_none(), "date-only must fail");
        assert!(parse_iso8601_utc("2026-13-01T00:00:00Z").is_none(), "month > 12 must fail");
        assert!(parse_iso8601_utc("2026-04-14T25:00:00Z").is_none(), "hour > 23 must fail");
        assert!(parse_iso8601_utc("not-a-date").is_none());
    }

    #[test]
    fn fleet_concern_failure_increments_counter() {
        with_framework_cwd(|tmp| {
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("first failure");
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("second failure");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");

            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(2));
            assert!(hub["first_failure_at"].is_string(), "first_failure_at should be set");
            assert_eq!(hub["last_class"].as_str(), Some("auth-mismatch"));
            assert_eq!(hub["concern_registered"].as_bool(), Some(false),
                "2 failures, no age → no concern yet");
        });
    }

    #[test]
    fn fleet_concern_success_resets_counter() {
        with_framework_cwd(|tmp| {
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("failure");
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("failure");
            // Pass → reset
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", None).expect("pass");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");
            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(0));
            assert!(hub["first_failure_at"].is_null(), "first_failure_at cleared on success");
            assert_eq!(hub["concern_registered"].as_bool(), Some(false));
        });
    }

    #[test]
    fn fleet_concern_fresh_failures_do_not_register() {
        with_framework_cwd(|tmp| {
            // 3 failures in quick succession — threshold count met but age <24h.
            for _ in 0..5 {
                maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                    .expect("failure");
            }
            let concerns = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns");
            assert!(
                !concerns.contains("ring20-management"),
                "must not register concern for hub failing <24h",
            );

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");
            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(5));
            assert_eq!(hub["concern_registered"].as_bool(), Some(false));
        });
    }

    #[test]
    fn fleet_concern_registers_when_aged_past_threshold() {
        with_framework_cwd(|tmp| {
            // Manually seed state with first_failure_at > 24h ago.
            let long_ago = now_unix_secs().saturating_sub(86_400 * 2); // 2 days ago
            // Build an ISO-8601 by round-tripping via our formatter indirectly —
            // since we control the format, we can manually construct a valid one.
            // Easiest: fabricate a known date-string that we know parses.
            let long_ago_iso = {
                // Simple: reuse now() then rewrite the year back — but that's fragile.
                // Use a fixed known-old string instead.
                "2026-01-01T00:00:00Z"
            };
            let seed = serde_json::json!({
                "hubs": {
                    "ring20-management": {
                        "consecutive_failures": 2,
                        "first_failure_at": long_ago_iso,
                        "last_failure_at": long_ago_iso,
                        "last_class": "auth-mismatch",
                        "concern_registered": false,
                    }
                }
            });
            let state_path = tmp.join(".context/working/.fleet-failure-state.json");
            std::fs::write(&state_path, serde_json::to_string_pretty(&seed).unwrap())
                .expect("seed state");
            let _ = long_ago; // suppress unused warning when asserts below don't need it

            // One more failure should push count to 3 and age past 24h → concern.
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("threshold-breaking failure");

            let concerns = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns");
            assert!(
                concerns.contains("ring20-management"),
                "concern must be registered: {concerns}",
            );
            assert!(concerns.contains("type: gap"), "concern must be gap-typed");
            assert!(concerns.contains("severity: high"), "concern must be high severity");
            assert!(concerns.contains("status: watching"), "concern must start in watching");
            assert!(concerns.contains("T-1053"), "spec_reference must mention T-1053");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&state_path).expect("state exists"),
            ).expect("state parses");
            assert_eq!(
                state["hubs"]["ring20-management"]["concern_registered"].as_bool(),
                Some(true),
                "state must flag concern_registered after write",
            );

            // Subsequent failure must NOT write a second concern.
            let before_second = concerns.matches("ring20-management").count();
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("subsequent failure");
            let after_second = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns").matches("ring20-management").count();
            assert_eq!(before_second, after_second, "dedupe: must not add a second concern");
        });
    }

    // -------------------------------------------------------------------
    // T-1054: fleet reauth — heal incantation renderer
    // -------------------------------------------------------------------

    #[test]
    fn fleet_reauth_render_with_secret_file_includes_expected_sections() {
        let entry = crate::config::HubEntry {
            address: "192.168.10.109:9100".to_string(),
            secret_file: Some("/root/.termlink/secrets/192.168.10.109.hex".to_string()),
            secret: None,
            scope: Some("execute".to_string()),
            bootstrap_from: None,
        };
        let out = render_fleet_reauth_plan("ring20-management", &entry);

        // Header carries profile name
        assert!(out.contains("ring20-management"), "profile name missing: {out}");
        // Address visible
        assert!(out.contains("192.168.10.109:9100"), "address missing: {out}");
        // Secret source line
        assert!(out.contains("file → /root/.termlink/secrets/192.168.10.109.hex"),
            "secret file path missing: {out}");
        // R2 compliance: trust anchor must be explicitly out-of-band
        assert!(out.contains("OUT-OF-BAND"), "trust anchor warning missing: {out}");
        assert!(out.contains("T-1055"), "forward pointer to bootstrap variant missing: {out}");
        // SSH read uses just the hostname, not the full host:port
        assert!(out.contains("ssh 192.168.10.109 -- sudo cat /var/lib/termlink/hub.secret"),
            "ssh read command missing or malformed: {out}");
        // Local write uses the full secret_file path
        assert!(out.contains("echo \"<paste-the-hex-from-step-1>\" > /root/.termlink/secrets/192.168.10.109.hex"),
            "local write step missing: {out}");
        // chmod 600 appears
        assert!(out.contains("chmod 600"), "chmod 600 missing: {out}");
        // Verify step points to fleet doctor
        assert!(out.contains("termlink fleet doctor"), "verify command missing: {out}");
    }

    #[test]
    fn fleet_reauth_render_with_inline_secret_warns() {
        let entry = crate::config::HubEntry {
            address: "10.0.0.5:9100".to_string(),
            secret_file: None,
            secret: Some("aa".repeat(32)),
            scope: None,
            bootstrap_from: None,
        };
        let out = render_fleet_reauth_plan("inline-hub", &entry);
        assert!(out.contains("inline in hubs.toml"), "inline-source label missing: {out}");
        assert!(out.contains("WARNING"), "inline-secret warning missing: {out}");
        assert!(out.contains("[hubs.inline-hub]"), "toml edit example missing: {out}");
    }

    #[test]
    fn fleet_reauth_render_with_no_secret_flags_missing() {
        let entry = crate::config::HubEntry {
            address: "10.0.0.9:9100".to_string(),
            secret_file: None,
            secret: None,
            scope: None,
            bootstrap_from: None,
        };
        let out = render_fleet_reauth_plan("broken-hub", &entry);
        assert!(out.contains("NONE configured"), "missing-secret warning missing: {out}");
    }

    #[test]
    fn fleet_reauth_errors_on_unknown_profile() {
        // Isolate HOME to a tempdir with a hubs.toml containing one unrelated profile.
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1054-unknown-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink")).expect("create .termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            r#"
[hubs.other]
address = "10.0.0.1:9100"
secret_file = "/tmp/other.hex"
"#,
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("does-not-exist", None, false);

        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("must error on unknown profile");
        let msg = format!("{err:#}");
        assert!(msg.contains("Unknown hub profile"), "error message shape: {msg}");
        assert!(msg.contains("does-not-exist"), "error must name the bad profile: {msg}");
        assert!(msg.contains("other"), "error must list known profiles: {msg}");
    }

    // -------------------------------------------------------------------
    // T-1055: fleet reauth --bootstrap-from <SOURCE>
    // -------------------------------------------------------------------

    #[test]
    fn fleet_reauth_hex_validator_accepts_valid_secret() {
        let hex = "0".repeat(64);
        assert_eq!(normalize_and_validate_secret_hex(&hex).unwrap(), hex);

        // Trimming whitespace is part of the contract (files often end with \n).
        let with_ws = format!("  {hex}\n");
        assert_eq!(normalize_and_validate_secret_hex(&with_ws).unwrap(), hex);

        // Uppercase is fine.
        let upper = "ABCDEF0123456789".repeat(4);
        assert_eq!(normalize_and_validate_secret_hex(&upper).unwrap(), upper);
    }

    #[test]
    fn fleet_reauth_hex_validator_rejects_wrong_length() {
        let short = "abcd";
        let err = normalize_and_validate_secret_hex(short).expect_err("short must error");
        assert!(format!("{err}").contains("expected 64 hex characters"), "{err}");

        let long = "a".repeat(100);
        let err = normalize_and_validate_secret_hex(&long).expect_err("long must error");
        assert!(format!("{err}").contains("expected 64 hex characters"), "{err}");
    }

    #[test]
    fn fleet_reauth_hex_validator_rejects_non_hex() {
        let bad = "z".repeat(64);
        let err = normalize_and_validate_secret_hex(&bad).expect_err("non-hex must error");
        assert!(format!("{err}").contains("non-hex characters"), "{err}");
    }

    #[test]
    fn fleet_reauth_bootstrap_unknown_prefix_errors() {
        let err = fetch_bootstrap_secret("random-junk").expect_err("unknown prefix must error");
        let msg = format!("{err:#}");
        assert!(msg.contains("Unknown bootstrap source"), "{msg}");
        assert!(msg.contains("file:"), "help text must mention file: form");
        assert!(msg.contains("ssh:"), "help text must mention ssh: form");
    }

    #[test]
    fn fleet_reauth_bootstrap_empty_prefixes_error() {
        let err = fetch_bootstrap_secret("file:").expect_err("file: alone must error");
        assert!(format!("{err:#}").contains("file: source requires a path"));
        let err = fetch_bootstrap_secret("ssh:").expect_err("ssh: alone must error");
        assert!(format!("{err:#}").contains("ssh: source requires a host"));
    }

    #[test]
    fn fleet_reauth_bootstrap_file_source_happy_path() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-file-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");
        std::fs::create_dir_all(tmp.join(".termlink/secrets")).expect("secrets dir");

        // Seed a bootstrap file + an existing stale secret_file.
        let new_secret = "ab".repeat(32);
        let bootstrap_path = tmp.join("new-secret.hex");
        std::fs::write(&bootstrap_path, format!("{new_secret}\n")).expect("seed bootstrap file");

        let secret_path = tmp.join(".termlink/secrets/192.168.10.109.hex");
        std::fs::write(&secret_path, "cd".repeat(32)).expect("seed stale secret");

        // Seed hubs.toml referencing the stale secret.
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            format!(
                "[hubs.ring20]\naddress = \"192.168.10.109:9100\"\nsecret_file = \"{}\"\n",
                secret_path.display(),
            ),
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let source = format!("file:{}", bootstrap_path.display());
        let result = cmd_fleet_reauth("ring20", Some(&source), false);

        // Capture state before restoring env.
        let written = std::fs::read_to_string(&secret_path).ok();
        let backup = std::fs::read_to_string(secret_path.with_extension("hex.bak")).ok();

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        result.expect("heal must succeed");
        assert_eq!(written.as_deref(), Some(new_secret.as_str()),
            "secret_file must contain the new secret");
        assert_eq!(backup.as_deref(), Some("cd".repeat(32).as_str()),
            ".bak must contain the prior secret");
    }

    #[test]
    fn fleet_reauth_bootstrap_rejects_invalid_hex_from_file() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-badhex-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");

        let bootstrap_path = tmp.join("bad.hex");
        std::fs::write(&bootstrap_path, "not a hex secret").expect("seed");

        let secret_path = tmp.join("target.hex");
        std::fs::write(&secret_path, "00".repeat(32)).expect("seed target");

        std::fs::create_dir_all(tmp.join(".termlink")).expect(".termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            format!(
                "[hubs.bad]\naddress = \"h:1\"\nsecret_file = \"{}\"\n",
                secret_path.display(),
            ),
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let source = format!("file:{}", bootstrap_path.display());
        let result = cmd_fleet_reauth("bad", Some(&source), false);

        // Capture pre-restore state.
        let target_content_after = std::fs::read_to_string(&secret_path).ok();

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("invalid hex must error");
        let msg = format!("{err:#}");
        assert!(msg.contains("not valid 64-char hex"), "err shape: {msg}");
        // The existing file must NOT have been overwritten.
        assert_eq!(target_content_after.as_deref(), Some("00".repeat(32).as_str()),
            "target file must be untouched when bootstrap source is invalid");
    }

    #[test]
    fn fleet_reauth_bootstrap_refuses_inline_secret_profile() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-inline-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink")).expect("create .termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            "[hubs.inline]\naddress = \"h:1\"\nsecret = \"aa\"\n",
        ).expect("seed");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("inline", Some("file:/dev/null"), false);

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("inline-secret profile must refuse --bootstrap-from");
        let msg = format!("{err:#}");
        assert!(msg.contains("inline secret"), "{msg}");
        assert!(msg.contains("Migrate first"), "must give actionable migration hint: {msg}");
    }

    #[test]
    fn fleet_reauth_errors_on_empty_hubs_config() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1054-empty-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("anything", None, false);

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("must error on empty hubs config");
        let msg = format!("{err:#}");
        assert!(msg.contains("No hubs configured"), "error shape: {msg}");
        assert!(msg.contains("termlink profile add"), "error must suggest profile add: {msg}");
    }

    // -------------------------------------------------------------------
    // T-1291: fleet reauth --bootstrap-from auto (declared trust anchor)
    // -------------------------------------------------------------------

    #[test]
    fn fleet_reauth_bootstrap_from_auto_resolves_declared_channel() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1291-auto-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink/secrets")).expect("dirs");

        // Seed bootstrap file + stale secret_file + declared bootstrap_from.
        let new_secret = "ab".repeat(32);
        let bootstrap_path = tmp.join("declared-anchor.hex");
        std::fs::write(&bootstrap_path, format!("{new_secret}\n")).expect("seed bootstrap");

        let secret_path = tmp.join(".termlink/secrets/declared.hex");
        std::fs::write(&secret_path, "cd".repeat(32)).expect("seed stale");

        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            format!(
                "[hubs.declared]\naddress = \"h:1\"\nsecret_file = \"{}\"\nbootstrap_from = \"file:{}\"\n",
                secret_path.display(),
                bootstrap_path.display(),
            ),
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("declared", Some("auto"), false);
        let written = std::fs::read_to_string(&secret_path).ok();

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        result.expect("auto-resolved heal must succeed");
        assert_eq!(written.as_deref(), Some(new_secret.as_str()),
            "secret_file must contain the new secret pulled via declared bootstrap_from");
    }

    #[test]
    fn fleet_reauth_bootstrap_from_auto_missing_declaration_errors() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1291-auto-missing-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink")).expect(".termlink");

        // Profile has no bootstrap_from declared.
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            "[hubs.bare]\naddress = \"h:1\"\nsecret_file = \"/tmp/whatever.hex\"\n",
        ).expect("seed");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("bare", Some("auto"), false);

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("missing bootstrap_from + auto must error");
        let msg = format!("{err:#}");
        assert!(msg.contains("no `bootstrap_from` declared"), "error must name the missing field: {msg}");
        assert!(msg.contains("ssh:<host>") || msg.contains("ssh:"), "error must give actionable example: {msg}");
        assert!(msg.contains("declared.") || msg.contains("declare it") || msg.contains("--bootstrap-from"), "error must offer alternative: {msg}");
    }

    #[test]
    fn heal_bootstrap_hint_with_declared_channel_returns_auto() {
        use crate::config::HubEntry;
        let entry = HubEntry {
            address: "192.168.10.122:9100".to_string(),
            secret: None,
            secret_file: Some("~/.termlink/secrets/ring20-management.hex".to_string()),
            scope: None,
            bootstrap_from: Some("ssh:192.168.10.122".to_string()),
        };
        let hint = heal_bootstrap_hint(&entry, &entry.address);
        assert_eq!(hint, "--bootstrap-from auto",
            "profile with declared bootstrap_from must yield `auto`");
    }

    #[test]
    fn heal_bootstrap_hint_without_declaration_yields_ssh_form_plus_tip() {
        use crate::config::HubEntry;
        let entry = HubEntry {
            address: "192.168.10.122:9100".to_string(),
            secret: None,
            secret_file: Some("/tmp/x.hex".to_string()),
            scope: None,
            bootstrap_from: None,
        };
        let hint = heal_bootstrap_hint(&entry, &entry.address);
        assert!(hint.starts_with("--bootstrap-from ssh:192.168.10.122"),
            "undeclared profile must yield literal ssh:<host>: got {hint}");
        assert!(hint.contains("bootstrap_from"),
            "undeclared profile hint must nudge operator toward declaring bootstrap_from: got {hint}");
        assert!(hint.contains("auto"),
            "tip must reference the `auto` mechanism it unlocks: got {hint}");
    }

    #[test]
    fn heal_readiness_footer_fires_when_any_profile_undeclared() {
        use crate::config::HubEntry;
        let mut hubs = std::collections::HashMap::new();
        hubs.insert("declared".to_string(), HubEntry {
            address: "10.0.0.1:9100".to_string(),
            secret: None, secret_file: Some("/x".to_string()), scope: None,
            bootstrap_from: Some("ssh:10.0.0.1".to_string()),
        });
        hubs.insert("undeclared-a".to_string(), HubEntry {
            address: "10.0.0.2:9100".to_string(),
            secret: None, secret_file: Some("/y".to_string()), scope: None,
            bootstrap_from: None,
        });
        hubs.insert("undeclared-b".to_string(), HubEntry {
            address: "10.0.0.3:9100".to_string(),
            secret: None, secret_file: Some("/z".to_string()), scope: None,
            bootstrap_from: None,
        });
        let msg = heal_readiness_footer(&hubs).expect("must emit when any undeclared");
        assert!(msg.starts_with("2 profile(s) lack"),
            "must name the undeclared count (2): got {msg}");
        assert!(msg.contains("--bootstrap-from"),
            "must point operator at the declarative flag: got {msg}");
        assert!(msg.contains("T-1291"),
            "must cite the feature task for traceability: got {msg}");
    }

    #[test]
    fn heal_readiness_footer_silent_when_all_declared() {
        use crate::config::HubEntry;
        let mut hubs = std::collections::HashMap::new();
        hubs.insert("a".to_string(), HubEntry {
            address: "10.0.0.1:9100".to_string(),
            secret: None, secret_file: Some("/x".to_string()), scope: None,
            bootstrap_from: Some("ssh:10.0.0.1".to_string()),
        });
        hubs.insert("b".to_string(), HubEntry {
            address: "10.0.0.2:9100".to_string(),
            secret: None, secret_file: Some("/y".to_string()), scope: None,
            bootstrap_from: Some("file:/etc/x".to_string()),
        });
        assert!(heal_readiness_footer(&hubs).is_none(),
            "must suppress nag when all profiles declared (don't pester configured fleets)");
    }

    #[test]
    fn heal_readiness_footer_silent_when_no_profiles() {
        let hubs = std::collections::HashMap::new();
        assert!(heal_readiness_footer(&hubs).is_none(),
            "empty fleet → no recommendation (handled by empty-state output upstream)");
    }

    #[test]
    fn fleet_doctor_hmac_diagnosis_uses_auto_when_declared() {
        use crate::config::HubEntry;
        let entry = HubEntry {
            address: "192.168.10.122:9100".to_string(),
            secret: None,
            secret_file: Some("~/.termlink/secrets/ring20-management.hex".to_string()),
            scope: None,
            bootstrap_from: Some("ssh:192.168.10.122".to_string()),
        };
        let diag = format_hmac_mismatch_diagnosis("ring20-management", &entry);
        assert!(diag.starts_with("HMAC secret mismatch"),
            "diagnosis must lead with the symptom: got {diag}");
        assert!(diag.contains("termlink fleet reauth ring20-management"),
            "diagnosis must name the profile, not <profile>: got {diag}");
        assert!(diag.contains("--bootstrap-from auto"),
            "declared-channel profile must point at `auto`, not the literal SSH form: got {diag}");
    }

    #[test]
    fn fleet_doctor_hmac_diagnosis_falls_back_to_ssh_without_declaration() {
        use crate::config::HubEntry;
        let entry = HubEntry {
            address: "192.168.10.121:9100".to_string(),
            secret: None,
            secret_file: Some("/tmp/x.hex".to_string()),
            scope: None,
            bootstrap_from: None,
        };
        let diag = format_hmac_mismatch_diagnosis("ring20-dashboard", &entry);
        assert!(diag.contains("termlink fleet reauth ring20-dashboard"),
            "diagnosis must name the profile: got {diag}");
        assert!(diag.contains("--bootstrap-from ssh:192.168.10.121"),
            "undeclared profile must use literal ssh:<host>: got {diag}");
        assert!(diag.contains("bootstrap_from"),
            "diagnosis must nudge operator toward declarative path: got {diag}");
    }

    #[test]
    #[cfg(unix)]
    fn secret_file_perms_warning_silent_for_chmod_600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::NamedTempFile::new().expect("create tmp");
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o600)).expect("chmod 600");
        assert!(
            secret_file_perms_warning(tmp.path()).is_none(),
            "0o600 is the canonical safe mode — must not warn"
        );
    }

    #[test]
    #[cfg(unix)]
    fn secret_file_perms_warning_fires_for_chmod_644() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::NamedTempFile::new().expect("create tmp");
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o644)).expect("chmod 644");
        let warning = secret_file_perms_warning(tmp.path())
            .expect("0o644 is world-readable — must warn (the original G-011 incident mode)");
        assert!(warning.contains("0o644"),
            "warning must include the actual mode for operator clarity: {warning}");
        assert!(warning.contains("chmod 600"),
            "warning must name the remediation: {warning}");
        assert!(warning.contains(tmp.path().to_str().unwrap()),
            "warning must include the path so operator can paste it: {warning}");
    }

    #[test]
    #[cfg(unix)]
    fn secret_file_perms_warning_fires_for_group_read() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::NamedTempFile::new().expect("create tmp");
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o660)).expect("chmod 660");
        let warning = secret_file_perms_warning(tmp.path())
            .expect("group-readable (0o660) is also a leak — must warn");
        assert!(warning.contains("0o660"), "warning must include the actual mode: {warning}");
    }

    #[test]
    #[cfg(unix)]
    fn secret_file_perms_warning_silent_for_missing_path() {
        let nonexistent = std::path::PathBuf::from("/tmp/T-1652-does-not-exist-12345.hex");
        assert!(
            secret_file_perms_warning(&nonexistent).is_none(),
            "absent files are handled by other error paths — perms check stays silent to avoid double-firing"
        );
    }

    #[test]
    fn expand_secret_file_path_substitutes_home_for_tilde() {
        // Only the leading `~/` form is expanded — bare `~` or mid-string is left alone.
        unsafe { std::env::set_var("HOME", "/home/testuser"); }
        let expanded = expand_secret_file_path("~/.termlink/secrets/foo.hex");
        assert_eq!(expanded, std::path::PathBuf::from("/home/testuser/.termlink/secrets/foo.hex"));

        // Absolute paths pass through unchanged.
        let absolute = expand_secret_file_path("/var/lib/termlink/hub.secret");
        assert_eq!(absolute, std::path::PathBuf::from("/var/lib/termlink/hub.secret"));
    }

    #[test]
    fn fleet_learning_no_op_outside_framework_project() {
        // Run in a fresh tempdir with NO .context/ present — must silently succeed.
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Anchor CWD against a leaked `tempfile::tempdir` from an unrelated test.
        std::env::set_current_dir("/").expect("cd to /");

        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1052-noframework-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");
        std::env::set_current_dir(&tmp).expect("cd into tmp");

        let result = maybe_record_auth_mismatch_learning(
            "somehub",
            "127.0.0.1:9100",
            "Token validation failed: invalid signature",
        );

        std::env::set_current_dir("/").expect("restore cwd to /");
        let _ = std::fs::remove_dir_all(&tmp);

        assert!(result.is_ok(), "must be best-effort outside framework projects: {result:?}");
    }

    // ---- T-1459: cut-readiness verdict ----
    // Aim: every output of `compute_cut_readiness_verdict` is covered, plus the
    // ACTIVE/decay boundary at ACTIVE_TRAFFIC_THRESHOLD_SECS so future tweaks
    // can't silently flip a hub's classification.

    fn now_ms_for_test() -> u128 {
        // Frozen "now" so test data with fixed last_ts values has predictable age.
        1_700_000_000_000
    }

    fn ts_seconds_ago(secs: u64) -> u128 {
        now_ms_for_test() - (secs as u128) * 1000
    }

    #[test]
    fn cut_readiness_verdict_all_clean_returns_cut_ready() {
        let verdict = compute_cut_readiness_verdict(
            &[],
            &[],
            &[],
            &["hub-a".to_string(), "hub-b".to_string()],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "CUT-READY");
    }

    #[test]
    fn cut_readiness_verdict_residue_only_returns_decaying() {
        // 10 minutes ago — well beyond the 5-min threshold.
        let with_traffic = vec![("hub-a".to_string(), 5u64, ts_seconds_ago(600))];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &["hub-b".to_string()],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "CUT-READY-DECAYING");
    }

    #[test]
    fn cut_readiness_verdict_active_traffic_returns_wait() {
        // 30 seconds ago — under the threshold, definitely live caller.
        let with_traffic = vec![("hub-a".to_string(), 5u64, ts_seconds_ago(30))];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &["hub-b".to_string()],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "WAIT");
    }

    #[test]
    fn cut_readiness_verdict_mixed_active_and_residue_returns_wait() {
        // One ACTIVE hub forces WAIT regardless of how clean the rest are.
        let with_traffic = vec![
            ("hub-a".to_string(), 5u64, ts_seconds_ago(60)),
            ("hub-b".to_string(), 100u64, ts_seconds_ago(7200)),
        ];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "WAIT");
    }

    #[test]
    fn cut_readiness_verdict_unsupported_hubs_return_uncertain() {
        let verdict = compute_cut_readiness_verdict(
            &[],
            &["legacy-hub".to_string()],
            &[],
            &["hub-a".to_string()],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "UNCERTAIN");
    }

    #[test]
    fn cut_readiness_verdict_no_audit_hubs_return_uncertain() {
        let verdict = compute_cut_readiness_verdict(
            &[],
            &[],
            &["fresh-hub".to_string()],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "UNCERTAIN");
    }

    #[test]
    fn cut_readiness_verdict_decaying_with_unsupported_degrades_to_uncertain() {
        // If we cannot measure some hubs, residue elsewhere does not let
        // us assert there are no live callers anywhere.
        let with_traffic = vec![("hub-a".to_string(), 1u64, ts_seconds_ago(7200))];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &["legacy-hub".to_string()],
            &[],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "UNCERTAIN");
    }

    #[test]
    fn cut_readiness_verdict_no_hubs_at_all_returns_uncertain() {
        let verdict = compute_cut_readiness_verdict(&[], &[], &[], &[], now_ms_for_test());
        assert_eq!(verdict, "UNCERTAIN");
    }

    #[test]
    fn cut_readiness_verdict_threshold_boundary_just_under_is_active() {
        // 1 second below threshold → ACTIVE → WAIT.
        let with_traffic = vec![(
            "hub-a".to_string(),
            1u64,
            ts_seconds_ago(ACTIVE_TRAFFIC_THRESHOLD_SECS - 1),
        )];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "WAIT");
    }

    #[test]
    fn cut_readiness_verdict_threshold_boundary_at_threshold_is_decaying() {
        // Exactly at threshold = NOT active (tag uses `<`, not `<=`).
        let with_traffic = vec![(
            "hub-a".to_string(),
            1u64,
            ts_seconds_ago(ACTIVE_TRAFFIC_THRESHOLD_SECS),
        )];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "CUT-READY-DECAYING");
    }

    // ---- T-1461: fleet-wide top_callers aggregate ----

    fn build_hub_top_callers(
        entries: &[(&str, &[(&str, u64)])],
    ) -> std::collections::BTreeMap<String, Vec<(String, u64)>> {
        let mut m = std::collections::BTreeMap::new();
        for (hub, callers) in entries {
            let v: Vec<(String, u64)> = callers.iter().map(|(id, c)| (id.to_string(), *c)).collect();
            m.insert(hub.to_string(), v);
        }
        m
    }

    #[test]
    fn fleet_top_callers_empty_input_returns_empty() {
        let m = std::collections::BTreeMap::new();
        let out = aggregate_fleet_top_callers(&m);
        assert!(out.is_empty());
    }

    #[test]
    fn fleet_top_callers_single_hub_passes_through_sorted() {
        let m = build_hub_top_callers(&[("hub-a", &[("addr:1.1.1.1", 10), ("addr:2.2.2.2", 3)])]);
        let out = aggregate_fleet_top_callers(&m);
        assert_eq!(out, vec![("addr:1.1.1.1".to_string(), 10), ("addr:2.2.2.2".to_string(), 3)]);
    }

    #[test]
    fn fleet_top_callers_same_caller_across_hubs_sums() {
        // The headline case: ring20-dashboard polls 3 hubs, each shows 579 calls.
        // Fleet-wide should be one entry with 1737.
        let m = build_hub_top_callers(&[
            ("local-test", &[("addr:192.168.10.121", 579)]),
            ("ring20-management", &[("addr:192.168.10.121", 579)]),
            ("workstation-107-public", &[("addr:192.168.10.121", 579)]),
        ]);
        let out = aggregate_fleet_top_callers(&m);
        assert_eq!(out, vec![("addr:192.168.10.121".to_string(), 1737)]);
    }

    #[test]
    fn fleet_top_callers_different_callers_sorted_by_count_desc() {
        let m = build_hub_top_callers(&[
            ("hub-a", &[("addr:A", 5), ("addr:B", 3)]),
            ("hub-b", &[("addr:C", 10), ("addr:A", 2)]),
        ]);
        let out = aggregate_fleet_top_callers(&m);
        // C: 10, A: 7, B: 3
        assert_eq!(out[0], ("addr:C".to_string(), 10));
        assert_eq!(out[1], ("addr:A".to_string(), 7));
        assert_eq!(out[2], ("addr:B".to_string(), 3));
    }

    #[test]
    fn fleet_top_callers_ties_broken_by_id_for_determinism() {
        let m = build_hub_top_callers(&[
            ("hub-a", &[("addr:zebra", 5), ("addr:alpha", 5)]),
        ]);
        let out = aggregate_fleet_top_callers(&m);
        // Same count → alphabetical id order.
        assert_eq!(out[0].0, "addr:alpha");
        assert_eq!(out[1].0, "addr:zebra");
    }

    #[test]
    fn cut_readiness_verdict_zero_last_ts_does_not_count_as_active() {
        // last_ts=0 means "no timestamp recorded" — should be treated as residue,
        // not active (otherwise hubs without ts metadata would always force WAIT).
        let with_traffic = vec![("hub-a".to_string(), 5u64, 0u128)];
        let verdict = compute_cut_readiness_verdict(
            &with_traffic,
            &[],
            &[],
            &[],
            now_ms_for_test(),
        );
        assert_eq!(verdict, "CUT-READY-DECAYING");
    }

    // ---- T-1467: derive_top_callers_from_by_method (pre-T-1460 hub fallback) ----

    #[test]
    fn derive_top_callers_empty_object_returns_empty() {
        let bm = serde_json::json!({});
        assert!(derive_top_callers_from_by_method(&bm).is_empty());
    }

    #[test]
    fn derive_top_callers_non_object_returns_empty() {
        // Defensive: hub returned `null` or an array — don't panic.
        assert!(derive_top_callers_from_by_method(&serde_json::Value::Null).is_empty());
        assert!(derive_top_callers_from_by_method(&serde_json::json!([])).is_empty());
    }

    #[test]
    fn derive_top_callers_single_method_one_caller() {
        let bm = serde_json::json!({
            "inbox.status": {
                "callers": [{"from": "tl-abc", "count": 42}],
                "count": 42,
                "last_ts_ms": 0,
            }
        });
        let out = derive_top_callers_from_by_method(&bm);
        assert_eq!(out, vec![("tl-abc".to_string(), 42)]);
    }

    #[test]
    fn derive_top_callers_multiple_methods_overlapping_callers_sums() {
        // The headline case: same caller hits both inbox.status and event.broadcast.
        // Derived list should have one entry summing both counts.
        let bm = serde_json::json!({
            "inbox.status": {
                "callers": [
                    {"from": "tl-poller", "count": 100},
                    {"from": "tl-other", "count": 5},
                ],
            },
            "event.broadcast": {
                "callers": [
                    {"from": "tl-poller", "count": 7},
                    {"from": "tl-third", "count": 50},
                ],
            },
        });
        let out = derive_top_callers_from_by_method(&bm);
        // poller: 107, third: 50, other: 5 — sorted by count desc.
        assert_eq!(out[0], ("tl-poller".to_string(), 107));
        assert_eq!(out[1], ("tl-third".to_string(), 50));
        assert_eq!(out[2], ("tl-other".to_string(), 5));
    }

    #[test]
    fn derive_top_callers_skips_zero_counts() {
        // Zero entries are noise — filter them out.
        let bm = serde_json::json!({
            "inbox.status": {
                "callers": [
                    {"from": "tl-zero", "count": 0},
                    {"from": "tl-one", "count": 1},
                ],
            }
        });
        let out = derive_top_callers_from_by_method(&bm);
        assert_eq!(out, vec![("tl-one".to_string(), 1)]);
    }

    #[test]
    fn derive_top_callers_skips_malformed_entries() {
        // Missing `from` or `count` field — skip silently rather than panic.
        let bm = serde_json::json!({
            "inbox.status": {
                "callers": [
                    {"from": "tl-good", "count": 5},
                    {"count": 99},                 // no `from`
                    {"from": "tl-no-count"},        // no `count`
                    {"from": 42, "count": 1},       // wrong type
                ],
            }
        });
        let out = derive_top_callers_from_by_method(&bm);
        assert_eq!(out, vec![("tl-good".to_string(), 5)]);
    }

    #[test]
    fn derive_top_callers_method_without_callers_array_skipped() {
        // Hub may include a method block with no `callers` field at all.
        let bm = serde_json::json!({
            "inbox.status": { "count": 99, "last_ts_ms": 0 },
            "event.broadcast": {
                "callers": [{"from": "tl-real", "count": 3}],
            },
        });
        let out = derive_top_callers_from_by_method(&bm);
        assert_eq!(out, vec![("tl-real".to_string(), 3)]);
    }

    #[test]
    fn derive_top_callers_ties_broken_by_id_alphabetically() {
        let bm = serde_json::json!({
            "inbox.status": {
                "callers": [
                    {"from": "tl-zebra", "count": 5},
                    {"from": "tl-alpha", "count": 5},
                ],
            }
        });
        let out = derive_top_callers_from_by_method(&bm);
        assert_eq!(out[0].0, "tl-alpha");
        assert_eq!(out[1].0, "tl-zebra");
    }

    // ===== T-1468: legacy_trend + sparkline tests =====

    fn snap(label: &str, total: u64, ts: Option<u64>) -> (String, serde_json::Value) {
        let mut doc = serde_json::json!({
            "legacy_summary": {"total_legacy_fleet": total},
        });
        if let Some(t) = ts {
            doc["_snapshot_ts_ms"] = serde_json::json!(t);
        }
        (label.to_string(), doc)
    }

    fn refs(s: &[(String, serde_json::Value)]) -> Vec<(String, &serde_json::Value)> {
        s.iter().map(|(l, v)| (l.clone(), v)).collect()
    }

    #[test]
    fn legacy_trend_empty_input_yields_flat_no_points() {
        let (points, traj) = compute_legacy_trend(&[]);
        assert!(points.is_empty());
        assert_eq!(traj, Trajectory::Flat);
    }

    #[test]
    fn legacy_trend_single_snapshot_is_flat_no_delta() {
        let s = vec![snap("2026-05-01", 100, Some(1_000_000))];
        let (points, traj) = compute_legacy_trend(&refs(&s));
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].total, 100);
        assert_eq!(points[0].delta_from_prior, None);
        assert_eq!(traj, Trajectory::Flat);
    }

    #[test]
    fn legacy_trend_monotonic_decrease_is_decreasing() {
        let s = vec![
            snap("2026-05-01", 1000, None),
            snap("2026-05-02", 700, None),
            snap("2026-05-03", 400, None),
        ];
        let (points, traj) = compute_legacy_trend(&refs(&s));
        assert_eq!(points.len(), 3);
        assert_eq!(points[0].delta_from_prior, None);
        assert_eq!(points[1].delta_from_prior, Some(-300));
        assert_eq!(points[2].delta_from_prior, Some(-300));
        assert_eq!(traj, Trajectory::Decreasing);
    }

    #[test]
    fn legacy_trend_monotonic_increase_is_increasing() {
        let s = vec![
            snap("a", 10, None),
            snap("b", 20, None),
            snap("c", 50, None),
        ];
        let (_, traj) = compute_legacy_trend(&refs(&s));
        assert_eq!(traj, Trajectory::Increasing);
    }

    #[test]
    fn legacy_trend_plateau_is_flat() {
        // Same total in first and last → Flat even with bumps in between.
        let s = vec![
            snap("a", 100, None),
            snap("b", 110, None),
            snap("c", 100, None),
        ];
        let (_, traj) = compute_legacy_trend(&refs(&s));
        assert_eq!(traj, Trajectory::Flat);
    }

    #[test]
    fn legacy_trend_skips_snapshots_without_total() {
        // Pre-T-1459 snapshot has no total_legacy_fleet — silently dropped.
        let pre = (
            "old".to_string(),
            serde_json::json!({"legacy_summary": {"verdict": "CUT-READY"}}),
        );
        let post = snap("new", 5, None);
        let s = vec![pre, post];
        let (points, _) = compute_legacy_trend(&refs(&s));
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].label, "new");
    }

    #[test]
    fn legacy_trend_carries_ts_ms_when_present() {
        let s = vec![snap("a", 10, Some(1234567890))];
        let (points, _) = compute_legacy_trend(&refs(&s));
        assert_eq!(points[0].ts_ms, Some(1234567890));
    }

    #[test]
    fn legacy_trend_sparkline_empty_returns_empty_string() {
        assert_eq!(render_sparkline(&[]), "");
    }

    #[test]
    fn legacy_trend_sparkline_all_zero_returns_empty_string() {
        // All-zero series has no useful range to render — return empty so the
        // print path can suppress the "sparkline:" line.
        assert_eq!(render_sparkline(&[0, 0, 0]), "");
    }

    #[test]
    fn legacy_trend_sparkline_single_value_renders_one_block() {
        let s = render_sparkline(&[42]);
        assert_eq!(s.chars().count(), 1);
        assert_eq!(s, "█"); // single value normalizes to max → top block
    }

    #[test]
    fn legacy_trend_sparkline_min_max_render_extremes() {
        let s = render_sparkline(&[0, 100]);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0], '▁'); // 0 → bottom block
        assert_eq!(chars[1], '█'); // max → top block
    }

    #[test]
    fn legacy_trend_sparkline_monotonic_decreasing_renders_visibly_descending() {
        let s = render_sparkline(&[100, 50, 0]);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 3);
        // First should be tallest, last should be shortest.
        assert_eq!(chars[0], '█');
        assert!(chars[1] != '█');
        assert_eq!(chars[2], '▁');
    }

    // ===== T-1470: eta_to_zero tests =====

    fn tp(label: &str, ts_ms: Option<u64>, total: u64) -> TrendPoint {
        TrendPoint { label: label.to_string(), ts_ms, total, delta_from_prior: None }
    }

    const ONE_DAY_MS: u64 = 86_400_000;

    #[test]
    fn eta_to_zero_fewer_than_two_points_returns_none() {
        let pts = vec![tp("a", Some(0), 100)];
        assert!(compute_eta_to_zero(&pts, 0).is_none());
        assert!(compute_eta_to_zero(&[], 0).is_none());
    }

    #[test]
    fn eta_to_zero_flat_series_returns_none() {
        let pts = vec![
            tp("a", Some(0), 100),
            tp("b", Some(ONE_DAY_MS), 100),
            tp("c", Some(2 * ONE_DAY_MS), 100),
        ];
        assert!(compute_eta_to_zero(&pts, 2 * ONE_DAY_MS).is_none());
    }

    #[test]
    fn eta_to_zero_growing_series_returns_none() {
        let pts = vec![
            tp("a", Some(0), 100),
            tp("b", Some(ONE_DAY_MS), 200),
            tp("c", Some(2 * ONE_DAY_MS), 300),
        ];
        assert!(compute_eta_to_zero(&pts, 2 * ONE_DAY_MS).is_none());
    }

    #[test]
    fn eta_to_zero_clean_linear_decay_predicts_correct_date() {
        // 100 → 80 → 60 over two days: -10/day per snapshot, -20/day between.
        // Wait — slope should be -10 per day if we have 3 points each 1 day apart.
        // y = 100, 80, 60 at x = 0, 1, 2 days. Slope = -20/day. y=0 at x = 5d.
        let pts = vec![
            tp("a", Some(0), 100),
            tp("b", Some(ONE_DAY_MS), 80),
            tp("c", Some(2 * ONE_DAY_MS), 60),
        ];
        // now = the last observation time
        let eta = compute_eta_to_zero(&pts, 2 * ONE_DAY_MS).expect("decay should forecast");
        // From now, zero is 3 more days out (5d total - 2d elapsed).
        assert!((eta.days_from_now - 3.0).abs() < 0.01,
            "expected ~3.0 days from now, got {}", eta.days_from_now);
        // Slope is -20 calls/day.
        assert!((eta.slope_per_day - -20.0).abs() < 0.01,
            "expected slope_per_day ~-20, got {}", eta.slope_per_day);
    }

    #[test]
    fn eta_to_zero_noisy_decay_still_produces_forward_date() {
        // Real-world signal: not perfectly linear but trending down.
        // 1000, 950, 920, 800 at days 0,1,2,3. Net -200 over 3d.
        let pts = vec![
            tp("a", Some(0), 1000),
            tp("b", Some(ONE_DAY_MS), 950),
            tp("c", Some(2 * ONE_DAY_MS), 920),
            tp("d", Some(3 * ONE_DAY_MS), 800),
        ];
        let eta = compute_eta_to_zero(&pts, 3 * ONE_DAY_MS).expect("noisy decay should forecast");
        // ETA must be positive (in the future) and slope must be negative.
        assert!(eta.days_from_now > 0.0);
        assert!(eta.slope_per_day < 0.0);
    }

    #[test]
    fn eta_to_zero_already_zero_total_returns_none() {
        // Last point is zero — we're already at zero, no future ETA needed.
        let pts = vec![
            tp("a", Some(0), 50),
            tp("b", Some(ONE_DAY_MS), 0),
        ];
        assert!(compute_eta_to_zero(&pts, ONE_DAY_MS).is_none());
    }

    #[test]
    fn eta_to_zero_unix_secs_to_iso_date_known_dates() {
        // 0 → epoch
        assert_eq!(unix_secs_to_iso_date(0), "1970-01-01");
        // 2026-05-04 00:00:00 UTC == 1777881600
        assert_eq!(unix_secs_to_iso_date(1777881600), "2026-05-04");
        // 2000-01-01 == 946684800
        assert_eq!(unix_secs_to_iso_date(946684800), "2000-01-01");
        // 2024-02-29 (leap year) == 1709164800
        assert_eq!(unix_secs_to_iso_date(1709164800), "2024-02-29");
    }

    // ===== T-1462: legacy_diff tests =====

    fn ls_clean(hubs: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "verdict": "CUT-READY",
            "total_legacy_fleet": 0,
            "hubs_clean": hubs,
            "hubs_with_traffic": [],
            "hubs_unsupported": [],
            "hubs_no_audit": [],
            "top_callers_fleet": [],
        })
    }

    fn ls_with(traffic: &[(&str, u64)], callers: &[(&str, u64)]) -> serde_json::Value {
        let total: u64 = traffic.iter().map(|(_, c)| *c).sum();
        let traf_arr: Vec<serde_json::Value> = traffic
            .iter()
            .map(|(n, c)| serde_json::json!({"hub": n, "count": c, "last_ts_ms": 0u64}))
            .collect();
        let cal_arr: Vec<serde_json::Value> = callers
            .iter()
            .map(|(id, c)| serde_json::json!({"id": id, "count": c}))
            .collect();
        serde_json::json!({
            "verdict": "CUT-READY-DECAYING",
            "total_legacy_fleet": total,
            "hubs_clean": [],
            "hubs_with_traffic": traf_arr,
            "hubs_unsupported": [],
            "hubs_no_audit": [],
            "top_callers_fleet": cal_arr,
        })
    }

    #[test]
    fn legacy_diff_clean_to_clean_yields_zero() {
        let prior = ls_clean(&["a", "b"]);
        let cur = ls_clean(&["a", "b"]);
        let d = compute_legacy_diff(&prior, &cur, Some(1000), 2000);
        assert_eq!(d.total_fleet_delta, 0);
        // Both hubs are present on both sides with delta 0 → filtered from
        // human output (filtering is in the printer, not in the struct).
        assert!(d.per_hub.iter().all(|h| h.delta() == 0));
        assert_eq!(d.elapsed_ms, Some(1000));
    }

    #[test]
    fn legacy_diff_decay_yields_negative_total() {
        let prior = ls_with(&[("a", 100)], &[("addr:1.2.3.4", 100)]);
        let cur = ls_with(&[("a", 60)], &[("addr:1.2.3.4", 60)]);
        let d = compute_legacy_diff(&prior, &cur, Some(0), 60_000);
        assert_eq!(d.total_fleet_delta, -40);
        assert_eq!(d.elapsed_ms, Some(60_000));
        // -40 calls in 60s = -40 calls/min
        let rate = d.rate_per_min.expect("rate computable");
        assert!((rate - -40.0).abs() < 0.001, "rate was {}", rate);
        assert_eq!(d.per_caller[0].id, "addr:1.2.3.4");
        assert_eq!(d.per_caller[0].delta(), -40);
    }

    #[test]
    fn legacy_diff_growth_yields_positive_total() {
        let prior = ls_clean(&["a"]);
        let cur = ls_with(&[("a", 12)], &[("pid:42", 12)]);
        let d = compute_legacy_diff(&prior, &cur, Some(0), 30_000);
        assert_eq!(d.total_fleet_delta, 12);
        let rate = d.rate_per_min.expect("rate computable");
        assert!((rate - 24.0).abs() < 0.001, "rate was {}", rate); // 12 over 30s = 24/min
    }

    #[test]
    fn legacy_diff_hub_vanished_appears_in_per_hub() {
        let prior = ls_with(&[("dropped", 5), ("kept", 3)], &[]);
        let cur = ls_with(&[("kept", 3)], &[]);
        let d = compute_legacy_diff(&prior, &cur, Some(0), 1000);
        let dropped = d.per_hub.iter().find(|h| h.hub == "dropped").unwrap();
        assert_eq!(dropped.prior_count, Some(5));
        assert_eq!(dropped.current_count, None);
        assert_eq!(dropped.delta(), -5);
    }

    #[test]
    fn legacy_diff_hub_appeared_appears_in_per_hub() {
        let prior = ls_with(&[("kept", 3)], &[]);
        let cur = ls_with(&[("kept", 3), ("new", 7)], &[]);
        let d = compute_legacy_diff(&prior, &cur, Some(0), 1000);
        let appeared = d.per_hub.iter().find(|h| h.hub == "new").unwrap();
        assert_eq!(appeared.prior_count, None);
        assert_eq!(appeared.current_count, Some(7));
        assert_eq!(appeared.delta(), 7);
    }

    #[test]
    fn legacy_diff_caller_dominance_shift() {
        let prior = ls_with(
            &[("a", 100)],
            &[("addr:old", 80), ("addr:new", 20)],
        );
        let cur = ls_with(
            &[("a", 100)],
            &[("addr:old", 10), ("addr:new", 90)],
        );
        let d = compute_legacy_diff(&prior, &cur, Some(0), 60_000);
        // total delta is 0 (caller redistribution within same total)
        assert_eq!(d.total_fleet_delta, 0);
        // First entry has largest absolute delta — should be one of old/new (both = 70)
        let top = &d.per_caller[0];
        assert!(top.id == "addr:old" || top.id == "addr:new");
        assert_eq!(top.delta().abs(), 70);
    }

    #[test]
    fn legacy_diff_no_prior_ts_yields_no_rate() {
        let prior = ls_clean(&[]);
        let cur = ls_clean(&[]);
        let d = compute_legacy_diff(&prior, &cur, None, 1000);
        assert_eq!(d.elapsed_ms, None);
        assert_eq!(d.rate_per_min, None);
    }

    // ===== T-1465: verdict_to_exit_code tests =====

    #[test]
    fn verdict_to_exit_code_cut_ready_is_zero() {
        assert_eq!(verdict_to_exit_code("CUT-READY"), 0);
    }

    #[test]
    fn verdict_to_exit_code_decaying_is_zero() {
        // Decay residue is acceptable — operator may cut.
        assert_eq!(verdict_to_exit_code("CUT-READY-DECAYING"), 0);
    }

    #[test]
    fn verdict_to_exit_code_wait_is_ten() {
        assert_eq!(verdict_to_exit_code("WAIT"), 10);
    }

    #[test]
    fn verdict_to_exit_code_uncertain_is_eleven() {
        assert_eq!(verdict_to_exit_code("UNCERTAIN"), 11);
    }

    #[test]
    fn verdict_to_exit_code_unknown_is_uncertain_eleven() {
        // Forward-compatibility: any future verdict string gets the
        // operator-actionable exit code rather than slipping through as 0.
        assert_eq!(verdict_to_exit_code("FUTURE-VERDICT"), 11);
        assert_eq!(verdict_to_exit_code(""), 11);
    }

    #[test]
    fn legacy_diff_to_json_round_trip_keys_present() {
        let prior = ls_with(&[("a", 5)], &[("pid:9", 5)]);
        let cur = ls_with(&[("a", 2)], &[("pid:9", 2)]);
        let d = compute_legacy_diff(&prior, &cur, Some(0), 60_000);
        let j = legacy_diff_to_json(&d);
        for k in ["total_fleet_delta", "elapsed_ms", "rate_per_min", "per_hub", "per_caller"] {
            assert!(j.get(k).is_some(), "missing key {k}: {j}");
        }
        assert_eq!(j["total_fleet_delta"].as_i64(), Some(-3));
    }

    // === T-1688: bootstrap-check tests ===

    #[test]
    fn bootstrap_check_classify_no_anchor_when_none() {
        let s = classify_bootstrap_check(None);
        assert_eq!(s, BootstrapCheckStatus::NoAnchor);
        assert_eq!(s.as_str(), "no-anchor");
        assert!(s.error().is_none());
    }

    #[test]
    fn bootstrap_check_classify_ok_when_valid_file() {
        let tmp = std::env::temp_dir().join(format!("tl-bootstrap-check-ok-{}.hex", std::process::id()));
        // 64 hex chars
        std::fs::write(&tmp, "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n").unwrap();
        let source = format!("file:{}", tmp.display());
        let s = classify_bootstrap_check(Some(&source));
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(s, BootstrapCheckStatus::Ok);
    }

    #[test]
    fn bootstrap_check_classify_fetch_fail_when_file_missing() {
        let source = format!("file:/tmp/nonexistent-bootstrap-{}.hex", std::process::id());
        let s = classify_bootstrap_check(Some(&source));
        assert_eq!(s.as_str(), "fetch-fail");
        assert!(s.error().is_some());
    }

    #[test]
    fn bootstrap_check_classify_invalid_format_when_short() {
        let tmp = std::env::temp_dir()
            .join(format!("tl-bootstrap-check-short-{}.hex", std::process::id()));
        std::fs::write(&tmp, "not-64-chars").unwrap();
        let source = format!("file:{}", tmp.display());
        let s = classify_bootstrap_check(Some(&source));
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(s.as_str(), "invalid-format");
        assert!(s.error().is_some());
    }

    #[test]
    fn bootstrap_check_classify_fetch_fail_unknown_scheme() {
        let s = classify_bootstrap_check(Some("command:echo deadbeef"));
        assert_eq!(s.as_str(), "fetch-fail");
        assert!(s.error().unwrap().contains("Unknown bootstrap source"));
    }

    #[test]
    fn bootstrap_check_exit_code_all_ok() {
        let statuses = vec![BootstrapCheckStatus::Ok, BootstrapCheckStatus::Ok];
        assert_eq!(bootstrap_check_exit_code(&statuses, true), 0);
        assert_eq!(bootstrap_check_exit_code(&statuses, false), 0);
    }

    #[test]
    fn bootstrap_check_exit_code_any_fetch_fail_is_1() {
        let statuses = vec![
            BootstrapCheckStatus::Ok,
            BootstrapCheckStatus::FetchFail("boom".into()),
        ];
        assert_eq!(bootstrap_check_exit_code(&statuses, true), 1);
        assert_eq!(bootstrap_check_exit_code(&statuses, false), 1);
    }

    #[test]
    fn bootstrap_check_exit_code_any_invalid_format_is_1() {
        let statuses = vec![BootstrapCheckStatus::InvalidFormat("short".into())];
        assert_eq!(bootstrap_check_exit_code(&statuses, true), 1);
    }

    #[test]
    fn bootstrap_check_exit_code_all_no_anchor_under_all_is_2() {
        let statuses = vec![BootstrapCheckStatus::NoAnchor, BootstrapCheckStatus::NoAnchor];
        assert_eq!(bootstrap_check_exit_code(&statuses, true), 2);
        // Under single-profile mode, no-anchor isn't a failure (exit 0).
        assert_eq!(bootstrap_check_exit_code(&statuses, false), 0);
    }

    #[test]
    fn bootstrap_check_exit_code_mixed_no_anchor_plus_ok_under_all_is_0() {
        let statuses = vec![BootstrapCheckStatus::NoAnchor, BootstrapCheckStatus::Ok];
        assert_eq!(bootstrap_check_exit_code(&statuses, true), 0);
    }

    #[test]
    fn bootstrap_check_verdict_words() {
        assert_eq!(
            bootstrap_check_verdict(&[BootstrapCheckStatus::Ok], false),
            "ok"
        );
        assert_eq!(
            bootstrap_check_verdict(&[BootstrapCheckStatus::FetchFail("x".into())], true),
            "fail"
        );
        assert_eq!(
            bootstrap_check_verdict(&[BootstrapCheckStatus::NoAnchor, BootstrapCheckStatus::Ok], true),
            "mixed"
        );
        assert_eq!(
            bootstrap_check_verdict(&[BootstrapCheckStatus::NoAnchor], true),
            "no-anchor"
        );
        // Under single-profile mode (all=false), a sole no-anchor profile is informational, not failure.
        assert_eq!(
            bootstrap_check_verdict(&[BootstrapCheckStatus::NoAnchor], false),
            "ok"
        );
    }

    // T-1690: PL-021 flap analyzer tests
    fn rot(hub: &str, ts: &str, old_conn: &str, new_conn: &str, old_pin: &str, new_pin: &str) -> serde_json::Value {
        serde_json::json!({
            "ts": ts,
            "hub": hub,
            "kind": "transition",
            "old_conn": old_conn,
            "new_conn": new_conn,
            "old_pin": old_pin,
            "new_pin": new_pin,
        })
    }

    #[test]
    fn analyze_pl021_empty_log_classifies_nothing() {
        let entries: Vec<&serde_json::Value> = Vec::new();
        let report = analyze_pl021(&entries);
        assert!(report.is_empty());
    }

    #[test]
    fn analyze_pl021_only_new_entries_skipped() {
        // `kind: "new"` (first-time observation) is not a transition.
        let e = serde_json::json!({
            "ts": "2026-05-01T00:00:00Z", "hub": "h1", "kind": "new",
            "old_conn": "", "new_conn": "auth-mismatch",
            "old_pin": "-", "new_pin": "drift",
        });
        let report = analyze_pl021(&[&e]);
        assert!(report.is_empty(), "non-transition kinds must not be classified");
    }

    #[test]
    fn analyze_pl021_cert_only_rotation() {
        let e = rot("h1", "2026-05-01T00:00:00Z", "ok", "ok", "ok", "drift");
        let report = analyze_pl021(&[&e]);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].verdict, HubFlapVerdict::CertOnly);
        assert_eq!(report[0].cert_transitions, 1);
        assert_eq!(report[0].secret_transitions, 0);
        assert_eq!(report[0].double_rotations, 0);
    }

    #[test]
    fn analyze_pl021_secret_only_rotation() {
        let e = rot("h1", "2026-05-01T00:00:00Z", "ok", "auth-mismatch", "ok", "ok");
        let report = analyze_pl021(&[&e]);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].verdict, HubFlapVerdict::SecretOnly);
        assert_eq!(report[0].secret_transitions, 1);
    }

    #[test]
    fn analyze_pl021_single_double_rotation_not_candidate() {
        let e = rot("h1", "2026-05-01T00:00:00Z", "ok", "auth-mismatch", "ok", "drift");
        let report = analyze_pl021(&[&e]);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].verdict, HubFlapVerdict::SingleDoubleRotation);
        assert_eq!(report[0].double_rotations, 1);
        assert_eq!(report[0].last_double_rotation.as_deref(), Some("2026-05-01T00:00:00Z"));
    }

    #[test]
    fn analyze_pl021_two_double_rotations_is_candidate() {
        let e1 = rot("h1", "2026-05-01T00:00:00Z", "ok", "auth-mismatch", "ok", "drift");
        let e2 = rot("h1", "2026-05-02T00:00:00Z", "ok", "auth-mismatch", "ok", "drift");
        let report = analyze_pl021(&[&e1, &e2]);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].verdict, HubFlapVerdict::Pl021Candidate);
        assert_eq!(report[0].double_rotations, 2);
        assert_eq!(report[0].last_double_rotation.as_deref(), Some("2026-05-02T00:00:00Z"));
    }

    #[test]
    fn analyze_pl021_does_not_cross_contaminate_hubs() {
        // h1 has 1 double-rotation, h2 has 1 — neither alone is a candidate.
        let e1 = rot("h1", "2026-05-01T00:00:00Z", "ok", "auth-mismatch", "ok", "drift");
        let e2 = rot("h2", "2026-05-02T00:00:00Z", "ok", "auth-mismatch", "ok", "drift");
        let report = analyze_pl021(&[&e1, &e2]);
        assert_eq!(report.len(), 2);
        for r in &report {
            assert_eq!(r.verdict, HubFlapVerdict::SingleDoubleRotation,
                "hub {} must not inherit other hubs' transitions", r.hub);
        }
    }

    #[test]
    fn analyze_pl021_recovery_transitions_not_counted_as_rotations() {
        // drift→ok is recovery, not rotation. auth-mismatch→ok same.
        let e = rot("h1", "2026-05-01T00:00:00Z", "auth-mismatch", "ok", "drift", "ok");
        let report = analyze_pl021(&[&e]);
        assert!(report.is_empty(), "recovery transitions must not count as rotations");
    }

    #[test]
    fn analyze_pl021_already_drifted_no_new_transition() {
        // Both old and new are drift/auth-mismatch — no fresh rotation here.
        let e = rot("h1", "2026-05-01T00:00:00Z", "auth-mismatch", "auth-mismatch", "drift", "drift");
        let report = analyze_pl021(&[&e]);
        assert!(report.is_empty(), "stable-drifted state must not register as a transition");
    }

    // T-1820: classify_secret_file unit tests.

    #[test]
    fn secrets_audit_classifier_ok_perms_ok_format_referenced() {
        let hex = "0".repeat(64);
        let (status, reasons) = super::classify_secret_file(0o600, &hex, false, None);
        assert_eq!(status, "ok");
        assert!(reasons.is_empty(), "expected no reasons, got {reasons:?}");
    }

    #[test]
    fn secrets_audit_classifier_warn_perms_g011_incident() {
        // The exact G-011 incident mode (proxmox4.hex at 0o644).
        let hex = "0".repeat(64);
        let (status, reasons) = super::classify_secret_file(0o644, &hex, false, None);
        assert_eq!(status, "warn-perms");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("0o644"), "reason: {}", reasons[0]);
    }

    #[test]
    fn secrets_audit_classifier_warn_format_truncated() {
        // 17 chars — not a valid 32-byte hex secret.
        let (status, reasons) =
            super::classify_secret_file(0o600, "abcdef0123456789a", false, None);
        assert_eq!(status, "warn-format");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("17 chars"), "reason: {}", reasons[0]);
    }

    #[test]
    fn secrets_audit_classifier_perms_takes_priority_over_format() {
        // Both warn-perms AND warn-format — perms is the higher severity.
        let (status, reasons) = super::classify_secret_file(0o644, "short", false, None);
        assert_eq!(status, "warn-perms");
        assert_eq!(reasons.len(), 2, "expected both reasons recorded: {reasons:?}");
    }

    #[test]
    fn secrets_audit_classifier_owner_only_variants_are_ok() {
        // 0o400 and 0o700 are both group-and-other-empty; both must pass.
        let hex = "f".repeat(64);
        for mode in [0o400u32, 0o600, 0o700] {
            let (status, reasons) = super::classify_secret_file(mode, &hex, false, None);
            assert_eq!(status, "ok", "mode 0o{:o} must classify as ok: {reasons:?}", mode);
        }
    }

    #[test]
    fn secrets_audit_classifier_orphan_only_when_other_checks_pass() {
        let hex = "a".repeat(64);
        // Referenced + ok perms + ok format = "ok".
        let (status, _) = super::classify_secret_file(0o600, &hex, false, None);
        assert_eq!(status, "ok");
        // Orphan + ok perms + ok format = "info-orphan".
        let (status, reasons) = super::classify_secret_file(0o600, &hex, true, None);
        assert_eq!(status, "info-orphan");
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("not referenced"), "reason: {}", reasons[0]);
        // Orphan + warn-perms = "warn-perms" (orphan suppressed by higher severity).
        let (status, _) = super::classify_secret_file(0o644, &hex, true, None);
        assert_eq!(status, "warn-perms");
    }

    // T-1822: drift-check classifier tests.

    #[test]
    fn secrets_audit_classifier_ok_mirror_when_content_matches_authoritative() {
        let hex = "deadbeef".repeat(8); // 64 chars, valid hex
        let (status, reasons) = super::classify_secret_file(0o600, &hex, false, Some(&hex));
        assert_eq!(status, "ok-mirror");
        assert!(reasons.is_empty(), "ok-mirror should be reason-free: {reasons:?}");
    }

    #[test]
    fn secrets_audit_classifier_warn_drift_when_content_differs_from_authoritative() {
        let cache_hex = "a".repeat(64);
        let auth_hex = "b".repeat(64);
        let (status, reasons) =
            super::classify_secret_file(0o600, &cache_hex, false, Some(&auth_hex));
        assert_eq!(status, "warn-drift");
        assert_eq!(reasons.len(), 1);
        assert!(
            reasons[0].contains("differs from authoritative"),
            "reason: {}",
            reasons[0]
        );
    }

    #[test]
    fn secrets_audit_classifier_drift_check_skipped_when_format_bad() {
        // Cache hex is malformed — warn-format must win, no drift verdict
        // (you can't compare apples to a half-banana).
        let cache = "short";
        let auth = "f".repeat(64);
        let (status, reasons) =
            super::classify_secret_file(0o600, cache, false, Some(&auth));
        assert_eq!(status, "warn-format");
        // Reasons should mention format only, not drift.
        assert!(!reasons.iter().any(|r| r.contains("differs from authoritative")),
            "drift reason must not appear when cache is malformed: {reasons:?}");
    }

    #[test]
    fn secrets_audit_classifier_perms_outranks_drift() {
        // World-readable AND drifted — perms is the higher-severity actionable
        // verdict (G-011 incident class), drift detail still recorded in reasons.
        let cache = "a".repeat(64);
        let auth = "b".repeat(64);
        let (status, reasons) = super::classify_secret_file(0o644, &cache, false, Some(&auth));
        assert_eq!(status, "warn-perms");
        assert!(
            reasons.iter().any(|r| r.contains("0o644"))
                && reasons.iter().any(|r| r.contains("differs from authoritative")),
            "both perms AND drift reasons must be recorded for full audit trail: {reasons:?}"
        );
    }

    #[test]
    fn secrets_audit_classifier_drift_case_insensitive() {
        // Some hub.secret writes use uppercase hex; the comparison must
        // normalize so the same 32-byte value isn't flagged as drift.
        let cache = "DEADBEEF".repeat(8);
        let auth = "deadbeef".repeat(8);
        let (status, _) = super::classify_secret_file(0o600, &cache, false, Some(&auth));
        assert_eq!(status, "ok-mirror");
    }

    // === T-1824: --target-cache scanner-level tests ===

    fn write_hex_file(dir: &std::path::Path, name: &str, hex: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, hex).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        p
    }

    #[test]
    fn secrets_audit_target_cache_narrows_drift_check_to_named_file() {
        // Two caches in dir. Target = first one. Authoritative content =
        // first one's content (so first → ok-mirror, second → plain ok
        // because target narrowing keeps drift check OFF for non-target rows).
        let tmp = tempfile::tempdir().unwrap();
        let auth_hex = "deadbeef".repeat(8); // 64 chars
        let other_hex = "cafebabe".repeat(8);
        let target = write_hex_file(tmp.path(), "self.hex", &auth_hex);
        let _other = write_hex_file(tmp.path(), "peer.hex", &other_hex);

        let rows = super::scan_secrets_dir(
            tmp.path(),
            |_p: &std::path::Path| true, // all referenced (no orphans)
            Some(&auth_hex),
            Some(&target),
        );
        assert_eq!(rows.len(), 2);

        let mut found_self = false;
        let mut found_other = false;
        for (path, _mode, _size, status, _reasons) in &rows {
            if path.ends_with("self.hex") {
                assert_eq!(status, "ok-mirror", "target self.hex must be drift-checked");
                found_self = true;
            } else if path.ends_with("peer.hex") {
                // Crucially: with broad mode this would be "warn-drift". With
                // target narrowing, peer.hex skips the drift comparison
                // entirely and falls back to plain "ok".
                assert_eq!(status, "ok", "non-target peer.hex must NOT be drift-checked");
                found_other = true;
            }
        }
        assert!(found_self && found_other, "both files must be in rows");
    }

    #[test]
    fn secrets_audit_target_cache_warn_drift_when_target_differs() {
        // Target file's content differs from authoritative → warn-drift.
        // Other file stays at plain ok regardless.
        let tmp = tempfile::tempdir().unwrap();
        let auth_hex = "deadbeef".repeat(8);
        let target_hex = "cafebabe".repeat(8); // differs from auth
        let other_hex = "12345678".repeat(8);
        let target = write_hex_file(tmp.path(), "self.hex", &target_hex);
        let _other = write_hex_file(tmp.path(), "peer.hex", &other_hex);

        let rows = super::scan_secrets_dir(
            tmp.path(),
            |_p: &std::path::Path| true,
            Some(&auth_hex),
            Some(&target),
        );
        let target_row = rows.iter().find(|(p, ..)| p.ends_with("self.hex")).unwrap();
        assert_eq!(target_row.3, "warn-drift");
        let other_row = rows.iter().find(|(p, ..)| p.ends_with("peer.hex")).unwrap();
        assert_eq!(other_row.3, "ok");
    }

    #[test]
    fn secrets_audit_target_cache_broad_mode_unchanged_when_target_none() {
        // Backward compat: when target_cache=None, broad-mode behavior
        // matches T-1822 — every cache gets drift-checked.
        let tmp = tempfile::tempdir().unwrap();
        let auth_hex = "deadbeef".repeat(8);
        let target_hex = "cafebabe".repeat(8);
        let other_hex = "12345678".repeat(8);
        let _t = write_hex_file(tmp.path(), "self.hex", &target_hex);
        let _o = write_hex_file(tmp.path(), "peer.hex", &other_hex);

        let rows = super::scan_secrets_dir(
            tmp.path(),
            |_p: &std::path::Path| true,
            Some(&auth_hex),
            None, // BROAD MODE
        );
        // Both should be warn-drift (both differ from authoritative).
        assert_eq!(rows.len(), 2);
        for (_, _, _, status, _) in &rows {
            assert_eq!(status, "warn-drift", "broad mode = all rows drift-checked");
        }
    }
}

