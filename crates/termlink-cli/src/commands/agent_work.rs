//! T-3470 (T-2026 slice 2): `termlink agent checkout` and `termlink agent commit`,
//! the worker-side git verbs of the claim-scoped work model.
//!
//! Shape and rationale: `docs/reports/T-3469-agent-launch-surface-design.md` §2
//! (locked signatures) and §2a (worked sequence). The hub gets NO new RPC and stores
//! no git state. Ownership is the shipped `channel.renew` (§2.2): every verb renews
//! the claim first, so a scope that is not a live claim held by the caller is
//! refused before git runs. Each call that knows its topic posts one event to
//! `agent-work:<claim topic>` (§2.5).
//!
//! Git runs on the worker's own host, in a clone under `TERMLINK_WORK_DIR`
//! (default `/var/lib/termlink/work`), never in the operator's checkout and never as
//! a `git worktree` (CLAUDE.md, session rules).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use termlink_protocol::control::error_code;
use termlink_protocol::transport::TransportAddr;
use termlink_session::claim_client::{self, ClaimError, ClaimSummary};

use super::infrastructure::{local_hub_fingerprint, resolve_hub_paths};

/// `WORK_GUARD_MISSING`: the checkout bootstrap found a required guard hook absent.
/// Defined here rather than in `control.rs` because the protocol crate is outside
/// this task's write set; move it next to T-3484's codes when those land.
pub(crate) const WORK_GUARD_MISSING: i64 = -32029;

/// Hooks a framework-vendored clone must carry before a worker may commit in it.
pub(crate) const REQUIRED_GUARD_HOOKS: &[&str] = &["pre-commit", "commit-msg"];

pub(crate) const DEFAULT_WORK_ROOT: &str = "/var/lib/termlink/work";
const DEFAULT_TTL_MS: u32 = 600_000;
const EVENT_TOPIC_PREFIX: &str = "agent-work:";
const EVENT_RETENTION: &str = "messages:1000";
const EVENT_POST_TIMEOUT: Duration = Duration::from_secs(10);

// ── Clone strategy ──────────────────────────────────────────────────────────

/// How a worker clone is materialised. Selected by `TERMLINK_WORKTREE_STRATEGY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloneStrategy {
    /// `cp -a --reflink=always` of a per-repo template kept current by fetch.
    Reflink,
    /// `btrfs subvolume snapshot` of a per-repo template subvolume.
    Snapshot,
    /// `git clone --dissociate`. The fallback off btrfs, or when a copy fails.
    Clone,
}

impl CloneStrategy {
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "reflink" => Ok(Self::Reflink),
            "snapshot" => Ok(Self::Snapshot),
            "clone" => Ok(Self::Clone),
            other => Err(format!(
                "unknown TERMLINK_WORKTREE_STRATEGY '{other}' — expected reflink, snapshot or clone"
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Reflink => "reflink",
            Self::Snapshot => "snapshot",
            Self::Clone => "clone",
        }
    }
}

/// The env value wins. Otherwise reflink on btrfs (copies share extents), clone elsewhere.
pub(crate) fn choose_strategy(env: Option<&str>, fs_type: Option<&str>) -> Result<CloneStrategy, String> {
    match env.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => CloneStrategy::parse(s),
        None if fs_type == Some("btrfs") => Ok(CloneStrategy::Reflink),
        None => Ok(CloneStrategy::Clone),
    }
}

/// Filesystem type of the nearest existing ancestor of `path` (`stat -f -c %T`).
pub(crate) fn fs_type(path: &Path) -> Option<String> {
    let mut probe = path;
    while !probe.exists() {
        probe = probe.parent()?;
    }
    let out = Command::new("stat").args(["-f", "-c", "%T"]).arg(probe).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ── Paths and identifiers ───────────────────────────────────────────────────

/// `TERMLINK_WORK_DIR`, else the default root outside every project tree.
pub(crate) fn work_root() -> PathBuf {
    std::env::var("TERMLINK_WORK_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WORK_ROOT))
}

pub(crate) fn branch_for(hub8: &str, claim_id: &str) -> String {
    format!("work/{hub8}/{claim_id}")
}

pub(crate) fn dir_for(root: &Path, hub8: &str, claim_id: &str) -> PathBuf {
    root.join(hub8).join(claim_id)
}

/// Claim ids are `clm-<ns>-<seq>-<tag>-<offset>` with a sanitised tag, so only
/// `[A-Za-z0-9_-]` is legal. That also makes the id safe as a path component and a
/// ref name, with no `..` or `/` to escape the work root.
pub(crate) fn validate_claim_id(claim_id: &str) -> Result<(), WorkError> {
    let legal = !claim_id.is_empty()
        && claim_id.len() <= 200
        && claim_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if legal {
        Ok(())
    } else {
        Err(WorkError::BadClaimId(claim_id.to_string()))
    }
}

/// First 8 hex digits of a hub fingerprint (`sha256:<hex>`, or bare hex). Claim ids
/// are per hub and hubs do not federate (G-060), so the hub id keys the work dir.
pub(crate) fn hub8_of(fingerprint: &str) -> Option<String> {
    let hex = fingerprint.strip_prefix("sha256:").unwrap_or(fingerprint);
    let head = hex.get(..8)?;
    head.chars().all(|c| c.is_ascii_hexdigit()).then(|| head.to_ascii_lowercase())
}

/// Template clone for a repo: one per repo string, under the work root.
fn template_dir(root: &Path, repo: &str) -> PathBuf {
    let mut h = DefaultHasher::new();
    repo.hash(&mut h);
    root.join(".templates").join(format!("{:016x}", h.finish()))
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) enum WorkError {
    /// The hub refused the ownership check (or was unreachable). Codes pass through.
    Claim(ClaimError),
    BadClaimId(String),
    /// A framework-vendored clone is missing a guard hook. Fail-closed.
    GuardMissing { dir: PathBuf, missing: Vec<String> },
    /// A git invocation failed. `output` is stderr, then stdout, so a hook's refusal
    /// reaches the operator intact.
    Git { what: String, output: String },
    NoCheckout(PathBuf),
    /// The commit exists but the claim could not be released after it.
    ReleaseAfterCommit { commit: String, source: ClaimError },
    Other(String),
}

impl std::fmt::Display for WorkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claim(e) => write!(f, "{e}{}", claim_hint(e)),
            Self::BadClaimId(id) => write!(
                f,
                "claim id {id:?} is not a plain token — expected only letters, digits, '-' and '_' (as printed by `termlink channel claim`)"
            ),
            Self::GuardMissing { dir, missing } => write!(
                f,
                "work clone {} is missing required guard hook(s): {}. Refusing fail-closed: the framework's guards must run on every commit. Install them with `.agentic-framework/bin/fw git install-hooks` in that clone, then retry",
                dir.display(),
                missing.join(", ")
            ),
            Self::Git { what, output } => write!(f, "git {what} failed: {output}"),
            Self::NoCheckout(dir) => write!(
                f,
                "no checkout at {} — run `termlink agent checkout --claim-id <id> --repo <url>` first",
                dir.display()
            ),
            Self::ReleaseAfterCommit { commit, source } => write!(
                f,
                "commit {commit} was made, but releasing the claim failed: {source}. The claim is still held; release it with `termlink channel release`"
            ),
            Self::Other(msg) => f.write_str(msg),
        }
    }
}

fn claim_hint(e: &ClaimError) -> &'static str {
    match e {
        ClaimError::NotOwned { .. } => {
            " — you are not the holder; run `termlink channel claims <topic>` to see who is"
        }
        ClaimError::NotFound { .. } => {
            " — the lease lapsed or was released; re-acquire with `termlink channel claim <topic> <offset>`"
        }
        ClaimError::Expired { .. } => {
            " — the lease expired; re-acquire with `termlink channel claim <topic> <offset>`"
        }
        _ => "",
    }
}

fn claim_error_code(e: &ClaimError) -> Option<i64> {
    match e {
        ClaimError::Conflict { .. } => Some(error_code::CLAIM_CONFLICT),
        ClaimError::NotFound { .. } => Some(error_code::CLAIM_NOT_FOUND),
        ClaimError::NotOwned { .. } => Some(error_code::CLAIM_NOT_OWNED),
        ClaimError::Expired { .. } => Some(error_code::CLAIM_EXPIRED),
        ClaimError::Hub { code, .. } => Some(*code),
        ClaimError::Transport(_) | ClaimError::Protocol(_) => None,
    }
}

impl WorkError {
    fn code(&self) -> Option<i64> {
        match self {
            Self::Claim(e) => claim_error_code(e),
            Self::BadClaimId(_) => None,
            Self::GuardMissing { .. } => Some(WORK_GUARD_MISSING),
            Self::ReleaseAfterCommit { source, .. } => claim_error_code(source),
            Self::Git { .. } | Self::NoCheckout(_) | Self::Other(_) => None,
        }
    }

    /// `{ok:false, error, code, data}`: the JSON the verb prints under `--json`.
    pub(crate) fn to_json(&self) -> Value {
        let data = match self {
            Self::GuardMissing { dir, missing } => {
                json!({"dir": dir.display().to_string(), "missing": missing})
            }
            Self::ReleaseAfterCommit { commit, .. } => json!({"commit": commit}),
            _ => json!({}),
        };
        json!({"ok": false, "error": self.to_string(), "code": self.code(), "data": data})
    }
}

// ── Hub seam ────────────────────────────────────────────────────────────────

/// The three hub calls the verbs make. `LiveHub` is the real one; tests substitute a
/// fake so the ownership and commit-failure paths run without a hub.
pub(crate) trait ClaimHub {
    /// `channel.renew`: the ownership check and lease extension in one call (§2.2).
    async fn renew(&self, claim_id: &str, claimer: &str, ttl_ms: u32) -> Result<ClaimSummary, ClaimError>;
    /// `channel.release` with `ack=true`: the work is done.
    async fn release_acked(&self, claim_id: &str, claimer: &str) -> Result<(), ClaimError>;
    /// One audit event to `agent-work:<topic>`. Best effort: never fails the verb (§2.5).
    async fn post_event(&self, topic: &str, body: &Value);
}

pub(crate) struct LiveHub {
    pub(crate) addr: TransportAddr,
}

impl ClaimHub for LiveHub {
    async fn renew(&self, claim_id: &str, claimer: &str, ttl_ms: u32) -> Result<ClaimSummary, ClaimError> {
        claim_client::channel_renew(&self.addr, claim_id, claimer, ttl_ms).await
    }

    async fn release_acked(&self, claim_id: &str, claimer: &str) -> Result<(), ClaimError> {
        claim_client::channel_release(&self.addr, claim_id, claimer, true).await.map(|_| ())
    }

    async fn post_event(&self, topic: &str, body: &Value) {
        // Subprocess the CLI rather than reimplement the signed post: the same path
        // every other `termlink channel post` takes. `create` is idempotent and sets
        // the bounded retention; a failure of either step is absorbed by the queue.
        let Ok(exe) = std::env::current_exe() else { return };
        let ev = format!("{EVENT_TOPIC_PREFIX}{topic}");
        let steps: [Vec<String>; 2] = [
            vec!["channel", "create", &ev, "--retention", EVENT_RETENTION]
                .into_iter()
                .map(String::from)
                .collect(),
            vec!["channel", "post", &ev, "--msg-type", "agent-work", "--payload", &body.to_string()]
                .into_iter()
                .map(String::from)
                .collect(),
        ];
        for args in steps {
            let mut cmd = tokio::process::Command::new(&exe);
            cmd.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true);
            let _ = tokio::time::timeout(EVENT_POST_TIMEOUT, cmd.status()).await;
        }
    }
}

fn event_body(verb: &str, agent_id: &str, hub8: &str, claim_id: &str, outcome: Value) -> Value {
    let mut body = outcome;
    if let Some(obj) = body.as_object_mut() {
        obj.insert("verb".into(), json!(verb));
        obj.insert("agent_id".into(), json!(agent_id));
        obj.insert("hub8".into(), json!(hub8));
        obj.insert("claim_id".into(), json!(claim_id));
    }
    body
}

fn outcome_json(result: &Result<Value, WorkError>) -> Value {
    match result {
        Ok(v) => v.clone(),
        Err(e) => e.to_json(),
    }
}

// ── Git helpers ─────────────────────────────────────────────────────────────

/// Run `git [-C dir] args`. Returns stdout (trimmed) on success.
fn git(dir: Option<&Path>, args: &[&str]) -> Result<String, WorkError> {
    let mut cmd = Command::new("git");
    if let Some(d) = dir {
        cmd.arg("-C").arg(d);
    }
    let out = cmd
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| WorkError::Other(format!("cannot run git: {e}")))?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string());
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let output = if stderr.is_empty() { stdout } else { stderr };
    Err(WorkError::Git { what: args.join(" "), output })
}

fn clone_into(repo: &str, dir: &Path) -> Result<(), WorkError> {
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| WorkError::Other(format!("cannot create {}: {e}", parent.display())))?;
    }
    let dir_s = dir.to_string_lossy().into_owned();
    git(None, &["clone", "--quiet", "--dissociate", repo, &dir_s]).map(|_| ())
}

/// Resolve `ref` to a commit: `origin/<ref>` first (a branch), then `<ref>` (a tag or sha).
fn resolve_base(dir: &Path, git_ref: &str) -> Result<String, WorkError> {
    for candidate in [format!("origin/{git_ref}^{{commit}}"), format!("{git_ref}^{{commit}}")] {
        if let Ok(sha) = git(Some(dir), &["rev-parse", "--verify", "--quiet", &candidate]) {
            if !sha.is_empty() {
                return Ok(sha);
            }
        }
    }
    Err(WorkError::Git {
        what: "resolve ref".into(),
        output: format!("ref '{git_ref}' not found in {}", dir.display()),
    })
}

fn hook_installed(dir: &Path, name: &str) -> bool {
    let Ok(rel) = git(Some(dir), &["rev-parse", "--git-path", &format!("hooks/{name}")]) else {
        return false;
    };
    let path = dir.join(rel);
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(&path)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// The guard bootstrap. A clone that vendors the framework (`.agentic-framework/bin/fw`)
/// must end up with every REQUIRED_GUARD_HOOK installed and executable, or the verb
/// refuses. A clone without the framework has no guards to enforce, so there is
/// nothing to check. The install step is best effort. A failed install shows up as a
/// missing hook, which is the error the operator needs.
fn bootstrap_guards(dir: &Path) -> Result<(), WorkError> {
    let fw = dir.join(".agentic-framework/bin/fw");
    if !fw.exists() {
        return Ok(());
    }
    let _ = Command::new(&fw)
        .args(["git", "install-hooks"])
        .current_dir(dir)
        .stdin(Stdio::null())
        .output();
    let missing: Vec<String> = REQUIRED_GUARD_HOOKS
        .iter()
        .filter(|h| !hook_installed(dir, h))
        .map(|h| h.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(WorkError::GuardMissing { dir: dir.to_path_buf(), missing })
    }
}

// ── checkout ────────────────────────────────────────────────────────────────

pub(crate) struct CheckoutReq<'a> {
    pub claim_id: &'a str,
    pub claimer: &'a str,
    pub repo: &'a str,
    pub git_ref: &'a str,
    pub ttl_ms: u32,
    pub hub8: &'a str,
    pub root: &'a Path,
    /// Explicit clone dir. Defaults to `root/<hub8>/<claim_id>`.
    pub work_dir: Option<&'a Path>,
    pub strategy: CloneStrategy,
}

/// Try the template copy strategies, then fall back to a plain clone with a note.
fn materialize(req: &CheckoutReq<'_>, dir: &Path, notes: &mut Vec<String>) -> Result<CloneStrategy, WorkError> {
    match req.strategy {
        CloneStrategy::Clone => {
            clone_into(req.repo, dir)?;
            Ok(CloneStrategy::Clone)
        }
        copy @ (CloneStrategy::Reflink | CloneStrategy::Snapshot) => {
            match copy_from_template(copy, req, dir) {
                Ok(()) => Ok(copy),
                Err(why) => {
                    notes.push(format!("{} unavailable ({why}); fell back to clone", copy.as_str()));
                    let _ = std::fs::remove_dir_all(dir);
                    clone_into(req.repo, dir)?;
                    Ok(CloneStrategy::Clone)
                }
            }
        }
    }
}

fn copy_from_template(strategy: CloneStrategy, req: &CheckoutReq<'_>, dir: &Path) -> Result<(), String> {
    let tpl = template_dir(req.root, req.repo);
    ensure_template(strategy, req.repo, &tpl)?;
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let out = match strategy {
        CloneStrategy::Reflink => Command::new("cp")
            .args(["-a", "--reflink=always"])
            .arg(&tpl)
            .arg(dir)
            .output(),
        CloneStrategy::Snapshot => Command::new("btrfs")
            .args(["subvolume", "snapshot"])
            .arg(&tpl)
            .arg(dir)
            .output(),
        CloneStrategy::Clone => return Err("clone is not a template copy".into()),
    }
    .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Create the template on first use. Every later use fetches, so a copy starts from a
/// current `origin/*`.
fn ensure_template(strategy: CloneStrategy, repo: &str, tpl: &Path) -> Result<(), String> {
    if tpl.join(".git").exists() {
        return git(Some(tpl), &["fetch", "--quiet", "--prune", "origin"])
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    std::fs::create_dir_all(tpl.parent().unwrap_or(tpl)).map_err(|e| e.to_string())?;
    if strategy == CloneStrategy::Snapshot {
        let out = Command::new("btrfs")
            .args(["subvolume", "create"])
            .arg(tpl)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
    }
    let tpl_s = tpl.to_string_lossy().into_owned();
    git(None, &["clone", "--quiet", repo, &tpl_s]).map(|_| ()).map_err(|e| e.to_string())
}

fn checkout_git(s: &ClaimSummary, req: &CheckoutReq<'_>, dir: &Path) -> Result<Value, WorkError> {
    let branch = branch_for(req.hub8, req.claim_id);
    let mut notes = Vec::new();
    let strategy_used = if dir.join(".git").exists() {
        // Idempotent re-run for the same claim: reuse the clone, but only on its own branch.
        let cur = git(Some(dir), &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if cur != branch {
            return Err(WorkError::Other(format!(
                "{} is checked out on '{cur}', not '{branch}' — refusing to reuse it",
                dir.display()
            )));
        }
        None
    } else {
        let non_empty = dir.exists()
            && dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(true);
        if non_empty {
            return Err(WorkError::Other(format!(
                "{} exists and is not a git checkout — refusing to overwrite it",
                dir.display()
            )));
        }
        Some(materialize(req, dir, &mut notes)?)
    };
    let base = resolve_base(dir, req.git_ref)?;
    if strategy_used.is_some() {
        git(Some(dir), &["checkout", "--quiet", "-B", &branch, &base])?;
    }
    bootstrap_guards(dir)?;
    let head = git(Some(dir), &["rev-parse", "HEAD"])?;
    Ok(json!({
        "ok": true,
        "claim_id": req.claim_id,
        "topic": s.topic,
        "offset": s.offset,
        "dir": dir.display().to_string(),
        "branch": branch,
        "base": base,
        "head": head,
        "claimed_until": s.claimed_until,
        "strategy": strategy_used.map(|s| s.as_str()).unwrap_or("reused"),
        "notes": notes,
    }))
}

/// `agent.checkout`: renew (ownership), materialise the clone, bootstrap guards, branch.
pub(crate) async fn checkout_with<H: ClaimHub>(hub: &H, req: &CheckoutReq<'_>) -> Result<Value, WorkError> {
    validate_claim_id(req.claim_id)?;
    let summary = hub
        .renew(req.claim_id, req.claimer, req.ttl_ms)
        .await
        .map_err(WorkError::Claim)?;
    let dir = req
        .work_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dir_for(req.root, req.hub8, req.claim_id));
    let result = checkout_git(&summary, req, &dir);
    let body = event_body("checkout", req.claimer, req.hub8, req.claim_id, outcome_json(&result));
    hub.post_event(&summary.topic, &body).await;
    result
}

// ── commit ──────────────────────────────────────────────────────────────────

pub(crate) struct CommitReq<'a> {
    pub claim_id: &'a str,
    pub claimer: &'a str,
    pub message: &'a str,
    /// Empty means "everything" (`git add -A`).
    pub paths: &'a [String],
    pub ttl_ms: u32,
    /// Release the claim with ack=true after the commit exists.
    pub release: bool,
    pub hub8: &'a str,
    pub dir: &'a Path,
}

struct CommitOut {
    sha: String,
    added: Vec<String>,
    modified: Vec<String>,
    deleted: Vec<String>,
}

/// `git diff --cached --name-status` lines into (added, modified, deleted).
fn parse_name_status(text: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (mut added, mut modified, mut deleted) = (Vec::new(), Vec::new(), Vec::new());
    for line in text.lines() {
        let Some((status, path)) = line.split_once('\t') else { continue };
        let bucket = match status.chars().next() {
            Some('A') => &mut added,
            Some('D') => &mut deleted,
            _ => &mut modified,
        };
        bucket.push(path.to_string());
    }
    (added, modified, deleted)
}

fn commit_git(dir: &Path, message: &str, paths: &[String]) -> Result<CommitOut, WorkError> {
    if !dir.join(".git").exists() {
        return Err(WorkError::NoCheckout(dir.to_path_buf()));
    }
    let mut add: Vec<&str> = vec!["add", "-A"];
    if !paths.is_empty() {
        add.push("--");
        add.extend(paths.iter().map(String::as_str));
    }
    git(Some(dir), &add)?;
    let staged = git(Some(dir), &["diff", "--cached", "--no-renames", "--name-status"])?;
    let (added, modified, deleted) = parse_name_status(&staged);
    // The repo's own hooks run here (task reference, secret scan, large file). A
    // refusal comes back as this error with the hook's output, and the claim is untouched.
    git(Some(dir), &["commit", "--quiet", "-m", message])?;
    let sha = git(Some(dir), &["rev-parse", "HEAD"])?;
    Ok(CommitOut { sha, added, modified, deleted })
}

/// `agent.commit`: renew (ownership), commit in the clone, optionally release after.
pub(crate) async fn commit_with<H: ClaimHub>(hub: &H, req: &CommitReq<'_>) -> Result<Value, WorkError> {
    validate_claim_id(req.claim_id)?;
    if req.message.trim().is_empty() {
        return Err(WorkError::Other("commit message is empty — pass -m \"T-NNNN: what changed\"".into()));
    }
    let summary = hub
        .renew(req.claim_id, req.claimer, req.ttl_ms)
        .await
        .map_err(WorkError::Claim)?;
    let result = match commit_git(req.dir, req.message, req.paths) {
        // The claim was renewed above and is NOT released: the work is not in the repo.
        Err(e) => Err(e),
        Ok(c) if req.release => match hub.release_acked(req.claim_id, req.claimer).await {
            Ok(()) => Ok(commit_json(req, &summary, &c, true)),
            Err(source) => Err(WorkError::ReleaseAfterCommit { commit: c.sha, source }),
        },
        Ok(c) => Ok(commit_json(req, &summary, &c, false)),
    };
    let body = event_body("commit", req.claimer, req.hub8, req.claim_id, outcome_json(&result));
    hub.post_event(&summary.topic, &body).await;
    result
}

fn commit_json(req: &CommitReq<'_>, s: &ClaimSummary, c: &CommitOut, released: bool) -> Value {
    json!({
        "ok": true,
        "claim_id": req.claim_id,
        "topic": s.topic,
        "commit": c.sha,
        "paths_added": c.added,
        "paths_modified": c.modified,
        "paths_deleted": c.deleted,
        "claimed_until": s.claimed_until,
        "released": released,
    })
}

// ── CLI entry points ────────────────────────────────────────────────────────

/// Connect to the local hub. The message is the one `hub_socket` uses, so tests and
/// the MCP parity pair see the same refusal.
fn live_hub() -> Result<LiveHub, WorkError> {
    let (_, sock) = resolve_hub_paths();
    if !sock.exists() {
        return Err(WorkError::Other(format!(
            "Hub is not running (no socket at {}) — start it with 'termlink hub start'",
            sock.display()
        )));
    }
    Ok(LiveHub { addr: TransportAddr::unix(sock) })
}

/// `--claimer` → `$TERMLINK_AGENT_ID` → `~/.termlink/be-reachable.state`. Never invented.
fn resolve_claimer(flag: Option<&str>) -> Result<String, WorkError> {
    let from_state = || -> Option<String> {
        let home = std::env::var("HOME").ok()?;
        let raw = std::fs::read_to_string(Path::new(&home).join(".termlink/be-reachable.state")).ok()?;
        let v: Value = serde_json::from_str(&raw).ok()?;
        v.get("agent_id")?.as_str().map(str::to_string)
    };
    let nonempty = |s: String| (!s.trim().is_empty()).then_some(s);
    flag.map(str::to_string)
        .and_then(nonempty)
        .or_else(|| std::env::var("TERMLINK_AGENT_ID").ok().and_then(nonempty))
        .or_else(from_state)
        .ok_or_else(|| {
            WorkError::Other(
                "no claimer: pass --claimer, set TERMLINK_AGENT_ID, or run `/be-reachable` — the verb never invents an identity"
                    .into(),
            )
        })
}

fn local_hub8() -> Result<String, WorkError> {
    // Test seam (PL-213): pins the hub id without a TLS cert on disk.
    if let Some(v) = std::env::var("TERMLINK_WORK_HUB8").ok().filter(|s| !s.trim().is_empty()) {
        return hub8_of(&v).ok_or_else(|| WorkError::Other(format!("TERMLINK_WORK_HUB8 '{v}' is not hex")));
    }
    let (_, fp) = local_hub_fingerprint().map_err(|e| WorkError::Other(e.to_string()))?;
    hub8_of(&fp).ok_or_else(|| WorkError::Other(format!("hub fingerprint '{fp}' has no 8-hex prefix")))
}

fn finish(res: Result<Value, WorkError>, json: bool, render: fn(&Value)) -> Result<()> {
    match res {
        Ok(v) => {
            if json {
                println!("{v}");
            } else {
                render(&v);
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(e.to_json());
            }
            Err(anyhow!("{e}"))
        }
    }
}

fn render_checkout(v: &Value) {
    for key in ["claim_id", "topic", "offset", "dir", "branch", "base", "head", "strategy", "claimed_until"] {
        if let Some(val) = v.get(key) {
            let shown = val.as_str().map(str::to_string).unwrap_or_else(|| val.to_string());
            println!("{key:<14} {shown}");
        }
    }
    if let Some(notes) = v.get("notes").and_then(Value::as_array) {
        for n in notes.iter().filter_map(Value::as_str) {
            println!("note:          {n}");
        }
    }
}

fn render_commit(v: &Value) {
    println!("claim_id:      {}", v["claim_id"].as_str().unwrap_or("?"));
    println!("commit:        {}", v["commit"].as_str().unwrap_or("?"));
    for key in ["paths_added", "paths_modified", "paths_deleted"] {
        let n = v[key].as_array().map(Vec::len).unwrap_or(0);
        println!("{key:<14} {n}");
    }
    println!("released:      {}", v["released"]);
    println!("claimed_until: {}", v["claimed_until"]);
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_agent_checkout(
    claim_id: &str,
    claimer: Option<&str>,
    repo: &str,
    git_ref: &str,
    work_dir: Option<&str>,
    ttl_ms: Option<u32>,
    json: bool,
) -> Result<()> {
    let res = run_checkout(claim_id, claimer, repo, git_ref, work_dir, ttl_ms).await;
    finish(res, json, render_checkout)
}

async fn run_checkout(
    claim_id: &str,
    claimer: Option<&str>,
    repo: &str,
    git_ref: &str,
    work_dir: Option<&str>,
    ttl_ms: Option<u32>,
) -> Result<Value, WorkError> {
    let hub = live_hub()?;
    let claimer = resolve_claimer(claimer)?;
    let hub8 = local_hub8()?;
    let root = work_root();
    let strategy = choose_strategy(
        std::env::var("TERMLINK_WORKTREE_STRATEGY").ok().as_deref(),
        fs_type(&root).as_deref(),
    )
    .map_err(WorkError::Other)?;
    let req = CheckoutReq {
        claim_id,
        claimer: &claimer,
        repo,
        git_ref,
        ttl_ms: ttl_ms.unwrap_or(DEFAULT_TTL_MS),
        hub8: &hub8,
        root: &root,
        work_dir: work_dir.map(Path::new),
        strategy,
    };
    checkout_with(&hub, &req).await
}

pub(crate) async fn cmd_agent_commit(
    claim_id: &str,
    claimer: Option<&str>,
    message: &str,
    paths: &[String],
    ttl_ms: Option<u32>,
    release: bool,
    json: bool,
) -> Result<()> {
    let res = run_commit(claim_id, claimer, message, paths, ttl_ms, release).await;
    finish(res, json, render_commit)
}

async fn run_commit(
    claim_id: &str,
    claimer: Option<&str>,
    message: &str,
    paths: &[String],
    ttl_ms: Option<u32>,
    release: bool,
) -> Result<Value, WorkError> {
    let hub = live_hub()?;
    let claimer = resolve_claimer(claimer)?;
    let hub8 = local_hub8()?;
    let dir = dir_for(&work_root(), &hub8, claim_id);
    let req = CommitReq {
        claim_id,
        claimer: &claimer,
        message,
        paths,
        ttl_ms: ttl_ms.unwrap_or(DEFAULT_TTL_MS),
        release,
        hub8: &hub8,
        dir: &dir,
    };
    commit_with(&hub, &req).await
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env_lock::ENV_LOCK;
    use std::cell::{Cell, RefCell};

    /// Scripted hub: renew answers from `renew_as`, release can be made to fail, and
    /// every call is counted so tests can assert what did NOT happen.
    struct FakeHub {
        renew_as: Renew,
        release_fails: bool,
        renews: Cell<u32>,
        releases: Cell<u32>,
        events: RefCell<Vec<Value>>,
    }

    #[derive(Clone, Copy)]
    enum Renew {
        Live,
        NotOwned,
    }

    impl FakeHub {
        fn new(renew_as: Renew) -> Self {
            Self {
                renew_as,
                release_fails: false,
                renews: Cell::new(0),
                releases: Cell::new(0),
                events: RefCell::new(Vec::new()),
            }
        }
    }

    impl ClaimHub for FakeHub {
        async fn renew(&self, claim_id: &str, claimer: &str, _ttl_ms: u32) -> Result<ClaimSummary, ClaimError> {
            self.renews.set(self.renews.get() + 1);
            match self.renew_as {
                Renew::Live => Ok(ClaimSummary {
                    claim_id: claim_id.into(),
                    topic: "work-queue".into(),
                    offset: 7,
                    claimer: claimer.into(),
                    claimed_at: 1,
                    claimed_until: 2,
                }),
                Renew::NotOwned => Err(ClaimError::NotOwned { claim_id: claim_id.into() }),
            }
        }

        async fn release_acked(&self, _claim_id: &str, _claimer: &str) -> Result<(), ClaimError> {
            self.releases.set(self.releases.get() + 1);
            if self.release_fails {
                Err(ClaimError::Hub { code: error_code::CLAIM_NOT_FOUND, message: "gone".into() })
            } else {
                Ok(())
            }
        }

        async fn post_event(&self, topic: &str, body: &Value) {
            self.events.borrow_mut().push(json!({"topic": topic, "body": body}));
        }
    }

    const CLAIM: &str = "clm-1-1-work_queue-7";
    const HUB8: &str = "cacc73ea";

    /// A source repo on `main`. With `framework_script`, it vendors `.agentic-framework/bin/fw`
    /// running that script. Commits use the identity from `set_test_git_identity`.
    fn source_repo(base: &Path, framework_script: Option<&str>) -> PathBuf {
        let repo = base.join("source");
        std::fs::create_dir_all(&repo).unwrap();
        let run = |args: &[&str]| {
            let st = Command::new("git").arg("-C").arg(&repo).args(args).status().unwrap();
            assert!(st.success(), "git {args:?} failed");
        };
        run(&["init", "-q", "-b", "main"]);
        std::fs::write(repo.join("README.md"), "hello\n").unwrap();
        if let Some(script) = framework_script {
            let fw = repo.join(".agentic-framework/bin/fw");
            std::fs::create_dir_all(fw.parent().unwrap()).unwrap();
            std::fs::write(&fw, script).unwrap();
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fw, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "init"]);
        repo
    }

    /// Stub `fw git install-hooks`: installs both guard hooks as executables that accept.
    const FW_INSTALLS_HOOKS: &str = "#!/bin/sh\n\
        d=$(git rev-parse --git-path hooks) || exit 1\n\
        mkdir -p \"$d\"\n\
        for h in pre-commit commit-msg; do printf '#!/bin/sh\\nexit 0\\n' > \"$d/$h\"; chmod +x \"$d/$h\"; done\n";

    /// Stub that installs nothing: the clone ends up with no guard hooks.
    const FW_INSTALLS_NOTHING: &str = "#!/bin/sh\nexit 0\n";

    /// Git identity for commits made by the verb, via a throwaway global config.
    /// Must be called while holding ENV_LOCK.
    fn set_test_git_identity(base: &Path) {
        let cfg = base.join("gitconfig");
        std::fs::write(&cfg, "[user]\n\tname = termlink-test\n\temail = test@termlink.invalid\n").unwrap();
        unsafe { std::env::set_var("GIT_CONFIG_GLOBAL", &cfg) };
    }

    fn req<'a>(root: &'a Path, repo: &'a str, strategy: CloneStrategy) -> CheckoutReq<'a> {
        CheckoutReq {
            claim_id: CLAIM,
            claimer: "w1",
            repo,
            git_ref: "main",
            ttl_ms: 60_000,
            hub8: HUB8,
            root,
            work_dir: None,
            strategy,
        }
    }

    fn git_count(dir: &Path, args: &[&str]) -> usize {
        let out = Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
        String::from_utf8_lossy(&out.stdout).lines().count()
    }

    #[test]
    fn strategy_env_wins_then_filesystem_decides() {
        assert_eq!(choose_strategy(None, Some("btrfs")).unwrap(), CloneStrategy::Reflink);
        assert_eq!(choose_strategy(None, Some("ext4")).unwrap(), CloneStrategy::Clone);
        assert_eq!(choose_strategy(None, None).unwrap(), CloneStrategy::Clone);
        assert_eq!(choose_strategy(Some("snapshot"), Some("ext4")).unwrap(), CloneStrategy::Snapshot);
        assert_eq!(choose_strategy(Some(" clone "), Some("btrfs")).unwrap(), CloneStrategy::Clone);
        assert!(choose_strategy(Some("worktree"), None).is_err(), "unknown strategy is refused, not guessed");
    }

    #[test]
    fn claim_id_must_be_a_plain_token() {
        assert!(validate_claim_id(CLAIM).is_ok());
        for bad in ["", "../etc", "a/b", "clm 1", "clm.1", ".."] {
            assert!(validate_claim_id(bad).is_err(), "{bad:?} must be refused");
        }
    }

    #[test]
    fn work_paths_key_on_hub8_and_claim_id() {
        assert_eq!(branch_for(HUB8, CLAIM), format!("work/{HUB8}/{CLAIM}"));
        assert_eq!(dir_for(Path::new("/r"), HUB8, CLAIM), Path::new("/r").join(HUB8).join(CLAIM));
        assert_eq!(hub8_of("sha256:CACC73EA0123").as_deref(), Some("cacc73ea"));
        assert_eq!(hub8_of("cacc73ea").as_deref(), Some("cacc73ea"));
        assert_eq!(hub8_of("sha256:zzzz"), None);
    }

    #[tokio::test]
    async fn checkout_clones_branches_and_posts_one_event() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let hub = FakeHub::new(Renew::Live);

        let out = checkout_with(&hub, &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .expect("checkout succeeds");

        let dir = dir_for(&root, HUB8, CLAIM);
        assert_eq!(out["dir"], json!(dir.display().to_string()));
        assert_eq!(out["branch"], json!(format!("work/{HUB8}/{CLAIM}")));
        assert_eq!(out["strategy"], json!("clone"));
        let branch = git(Some(&dir), &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap();
        assert_eq!(branch, format!("work/{HUB8}/{CLAIM}"));
        assert!(dir.join(".git/hooks/pre-commit").exists());
        assert_eq!(hub.renews.get(), 1, "ownership is checked once, through channel.renew");
        let events = hub.events.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], json!("work-queue"));
        assert_eq!(events[0]["body"]["verb"], json!("checkout"));
        assert_eq!(events[0]["body"]["ok"], json!(true));
    }

    #[tokio::test]
    async fn checkout_refuses_when_a_guard_hook_is_missing() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_NOTHING));
        let root = tmp.path().join("work");
        let hub = FakeHub::new(Renew::Live);

        let err = checkout_with(&hub, &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .expect_err("a framework clone without guards must be refused");

        match &err {
            WorkError::GuardMissing { missing, .. } => {
                assert_eq!(missing, &vec!["pre-commit".to_string(), "commit-msg".to_string()]);
            }
            other => panic!("expected GuardMissing, got {other}"),
        }
        let j = err.to_json();
        assert_eq!(j["ok"], json!(false));
        assert_eq!(j["code"], json!(WORK_GUARD_MISSING));
        assert_eq!(j["data"]["missing"], json!(["pre-commit", "commit-msg"]));
        let events = hub.events.borrow();
        assert_eq!(events[0]["body"]["ok"], json!(false), "the refusal is an audited event too");
    }

    #[tokio::test]
    async fn checkout_for_an_unowned_claim_touches_nothing() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let hub = FakeHub::new(Renew::NotOwned);

        let err = checkout_with(&hub, &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .expect_err("not the holder");

        assert!(matches!(err, WorkError::Claim(ClaimError::NotOwned { .. })));
        assert_eq!(err.to_json()["code"], json!(error_code::CLAIM_NOT_OWNED));
        assert!(!dir_for(&root, HUB8, CLAIM).exists(), "no clone for a claim the caller does not hold");
        assert!(hub.events.borrow().is_empty(), "no topic is known, so no event");
    }

    #[tokio::test]
    async fn checkout_rerun_for_the_same_claim_reuses_the_clone() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let hub = FakeHub::new(Renew::Live);
        let r = req(&root, repo.to_str().unwrap(), CloneStrategy::Clone);

        checkout_with(&hub, &r).await.unwrap();
        let second = checkout_with(&hub, &r).await.expect("re-run is idempotent");
        assert_eq!(second["strategy"], json!("reused"));
    }

    #[tokio::test]
    async fn commit_refuses_a_scope_not_held_by_the_caller() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let dir = dir_for(&root, HUB8, CLAIM);
        checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .unwrap();
        std::fs::write(dir.join("new.txt"), "x\n").unwrap();
        let before = git_count(&dir, &["rev-list", "--all"]);
        let hub = FakeHub::new(Renew::NotOwned);

        let err = commit_with(&hub, &commit_req(&dir, false, &[])).await.expect_err("refused");

        assert!(matches!(err, WorkError::Claim(ClaimError::NotOwned { .. })));
        assert_eq!(git_count(&dir, &["rev-list", "--all"]), before, "no commit was made");
        assert_eq!(hub.releases.get(), 0);
    }

    fn commit_req<'a>(dir: &'a Path, release: bool, paths: &'a [String]) -> CommitReq<'a> {
        CommitReq {
            claim_id: CLAIM,
            claimer: "w1",
            message: "T-3470: test commit",
            paths,
            ttl_ms: 60_000,
            release,
            hub8: HUB8,
            dir,
        }
    }

    #[tokio::test]
    async fn commit_success_reports_paths_and_keeps_the_claim_without_release() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let dir = dir_for(&root, HUB8, CLAIM);
        checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .unwrap();
        std::fs::write(dir.join("added.txt"), "a\n").unwrap();
        std::fs::write(dir.join("README.md"), "changed\n").unwrap();
        let hub = FakeHub::new(Renew::Live);

        let out = commit_with(&hub, &commit_req(&dir, false, &[])).await.expect("commit");

        assert_eq!(out["paths_added"], json!(["added.txt"]));
        assert_eq!(out["paths_modified"], json!(["README.md"]));
        assert_eq!(out["paths_deleted"], json!([]));
        assert_eq!(out["released"], json!(false));
        assert_eq!(out["commit"].as_str().map(str::len), Some(40));
        assert_eq!(hub.releases.get(), 0, "without --release the claim is kept");
        assert_eq!(hub.events.borrow()[0]["body"]["verb"], json!("commit"));
    }

    #[tokio::test]
    async fn commit_refused_by_a_hook_keeps_the_claim() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let dir = dir_for(&root, HUB8, CLAIM);
        checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .unwrap();
        // Replace the pre-commit guard with one that refuses, as the secret scan would.
        let hook = dir.join(".git/hooks/pre-commit");
        std::fs::write(&hook, "#!/bin/sh\necho 'secret-scan: refused' >&2\nexit 1\n").unwrap();
        std::fs::write(dir.join("secret.txt"), "x\n").unwrap();
        let hub = FakeHub::new(Renew::Live);

        let err = commit_with(&hub, &commit_req(&dir, true, &[])).await.expect_err("hook refuses");

        match &err {
            WorkError::Git { output, .. } => assert!(output.contains("secret-scan: refused"), "{output}"),
            other => panic!("expected the hook refusal, got {other}"),
        }
        assert_eq!(hub.renews.get(), 1);
        assert_eq!(hub.releases.get(), 0, "a failed commit must not release the claim");
    }

    #[tokio::test]
    async fn commit_with_release_releases_only_after_the_commit_exists() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let dir = dir_for(&root, HUB8, CLAIM);
        checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .unwrap();
        std::fs::write(dir.join("a.txt"), "a\n").unwrap();
        let hub = FakeHub::new(Renew::Live);

        let out = commit_with(&hub, &commit_req(&dir, true, &[])).await.expect("commit + release");

        assert_eq!(out["released"], json!(true));
        assert_eq!(hub.releases.get(), 1);
    }

    #[tokio::test]
    async fn release_failure_after_commit_reports_the_commit() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let dir = dir_for(&root, HUB8, CLAIM);
        checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Clone))
            .await
            .unwrap();
        std::fs::write(dir.join("b.txt"), "b\n").unwrap();
        let mut hub = FakeHub::new(Renew::Live);
        hub.release_fails = true;

        let err = commit_with(&hub, &commit_req(&dir, true, &[])).await.expect_err("release failed");

        match err {
            WorkError::ReleaseAfterCommit { commit, .. } => {
                assert_eq!(git(Some(&dir), &["rev-parse", "HEAD"]).unwrap(), commit, "the commit exists");
            }
            other => panic!("expected ReleaseAfterCommit, got {other}"),
        }
    }

    #[tokio::test]
    async fn commit_without_a_checkout_says_so() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("nowhere");
        let hub = FakeHub::new(Renew::Live);
        let err = commit_with(&hub, &commit_req(&dir, false, &[])).await.expect_err("no clone");
        assert!(matches!(err, WorkError::NoCheckout(_)));
    }

    fn btrfs_here(path: &Path) -> bool {
        fs_type(path).as_deref() == Some("btrfs")
    }

    /// Reflink and snapshot are btrfs features. Off btrfs the copy must fail, and the
    /// verb must fall back to a clone and say so. On btrfs there is nothing to fall back from.
    async fn copy_strategy_case(strategy: CloneStrategy) {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        if btrfs_here(tmp.path()) {
            eprintln!("SKIP {}: temp dir is on btrfs, so the copy succeeds and no fallback happens", strategy.as_str());
            return;
        }
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let hub = FakeHub::new(Renew::Live);
        let out = checkout_with(&hub, &req(&root, repo.to_str().unwrap(), strategy))
            .await
            .expect("fallback clone succeeds");
        assert_eq!(out["strategy"], json!("clone"));
        let notes = out["notes"].as_array().unwrap();
        assert!(notes.iter().any(|n| n.as_str().unwrap().contains(strategy.as_str())), "{notes:?}");
        assert!(dir_for(&root, HUB8, CLAIM).join(".git").exists());
    }

    #[tokio::test]
    async fn reflink_falls_back_to_clone_off_btrfs() {
        copy_strategy_case(CloneStrategy::Reflink).await;
    }

    #[tokio::test]
    async fn snapshot_falls_back_to_clone_off_btrfs() {
        copy_strategy_case(CloneStrategy::Snapshot).await;
    }

    #[tokio::test]
    async fn reflink_copies_a_template_on_btrfs() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        if !btrfs_here(tmp.path()) {
            eprintln!("SKIP reflink_copies_a_template_on_btrfs: temp dir is not on btrfs (fs={:?})", fs_type(tmp.path()));
            return;
        }
        set_test_git_identity(tmp.path());
        let repo = source_repo(tmp.path(), Some(FW_INSTALLS_HOOKS));
        let root = tmp.path().join("work");
        let out = checkout_with(&FakeHub::new(Renew::Live), &req(&root, repo.to_str().unwrap(), CloneStrategy::Reflink))
            .await
            .expect("reflink checkout");
        assert_eq!(out["strategy"], json!("reflink"));
    }

    #[test]
    fn name_status_parser_buckets_add_modify_delete() {
        let (a, m, d) = parse_name_status("A\tnew.txt\nM\tsrc/lib.rs\nD\told.txt\nT\tlink\n");
        assert_eq!(a, vec!["new.txt"]);
        assert_eq!(m, vec!["src/lib.rs", "link"]);
        assert_eq!(d, vec!["old.txt"]);
    }
}
