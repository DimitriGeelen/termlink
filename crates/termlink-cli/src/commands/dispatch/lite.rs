//! Lite dispatch — one short, bounded job through TermLink at a small fraction
//! of a full worker's start cost (T-3386, operator ruling RR-13 / CR-21).
//!
//! RR-13 rules that all delegation goes through TermLink and vendor sub-agents
//! are not used. A full `claude -p` worker started in this project loads
//! CLAUDE.md, every hook, every MCP server and every skill before it reads the
//! job: measured ~159K input tokens and ~$0.15 for a job that answers "OK"
//! (docs/operations/lite-dispatch.md). That cost is what made a sub-agent look
//! cheaper for a one-minute search. A lite worker starts with none of it:
//!
//! - `--safe-mode`: no CLAUDE.md, skills, plugins, hooks, MCP servers or custom
//!   agents (normal auth, unlike `--bare`, which needs an API key);
//! - `--strict-mcp-config` with no `--mcp-config`: no MCP server from anywhere;
//! - `--disable-slash-commands`, `--no-session-persistence`;
//! - `--system-prompt`: a short caller prompt replaces the default one;
//! - `--tools`: a bounded list, read-only (`Read,Grep,Glob`) by default.
//!
//! The answer comes back as a FILE: the worker writes `result.md` (its final
//! text) and `exit_code` into the job dir and emits `task.completed` carrying
//! `result_path` + `exit_code`, which `termlink dispatch`'s existing collect
//! loop already returns. Nothing new on the hub side.
//!
//! Wiring (a `--lite` flag in cli.rs / main.rs) is outside this module's owner's
//! area and is proposed in the T-3386 task Updates; until it lands only the
//! tests call into here.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cli::SpawnBackend;
use crate::util::shell_escape;

/// Read-only by default: a short job that needs to write is not a short job.
pub(crate) const DEFAULT_TOOLS: &[&str] = &["Read", "Grep", "Glob"];

pub(crate) const DEFAULT_SYSTEM_PROMPT: &str = "You are a short-job worker dispatched through TermLink. \
Do exactly the job you are given using only the tools available, then reply with the result alone. \
Do not modify files unless the job says to.";

/// Largest prompt passed as a single argv element. Linux refuses one argument
/// over MAX_ARG_STRLEN (128 KiB) with E2BIG at exec time, after the worker was
/// spawned — refusing here names the problem instead.
pub(crate) const MAX_PROMPT_BYTES: usize = 100_000;

/// One lite job.
pub(crate) struct LiteJob {
    pub prompt: String,
    pub system_prompt: Option<String>,
    /// `None` = [`DEFAULT_TOOLS`]; `Some(vec![])` = no tools at all.
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub max_budget_usd: Option<f64>,
    /// Where `result.md`, `stderr.log` and `exit_code` are written.
    pub job_dir: PathBuf,
    /// Worker binary; `claude` (resolved on PATH) unless a test substitutes a fake.
    pub worker_bin: String,
}

impl LiteJob {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.prompt.trim().is_empty() {
            anyhow::bail!("lite dispatch: the job prompt is empty");
        }
        if self.prompt.len() > MAX_PROMPT_BYTES {
            anyhow::bail!(
                "lite dispatch: prompt is {} bytes, over the {MAX_PROMPT_BYTES}-byte limit for one argument; put the material in a file and ask the worker to Read it",
                self.prompt.len()
            );
        }
        if let Some(b) = self.max_budget_usd
            && !(b.is_finite() && b > 0.0)
        {
            anyhow::bail!("lite dispatch: --max-budget-usd must be a positive number, got {b}");
        }
        Ok(())
    }

    pub(crate) fn result_path(&self) -> PathBuf {
        self.job_dir.join("result.md")
    }
}

/// The worker's argv. The prompt is ONE element: it is never spliced into a
/// shell line, so quotes, `$()` and newlines in it are inert.
pub(crate) fn lite_worker_argv(job: &LiteJob) -> Vec<String> {
    let tools = match &job.tools {
        Some(t) => t.join(","),
        None => DEFAULT_TOOLS.join(","),
    };
    let mut argv = vec![
        job.worker_bin.clone(),
        "-p".to_string(),
        job.prompt.clone(),
        "--safe-mode".to_string(),
        "--strict-mcp-config".to_string(),
        "--disable-slash-commands".to_string(),
        "--no-session-persistence".to_string(),
        "--system-prompt".to_string(),
        job.system_prompt
            .clone()
            .unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string()),
        // `--tools ""` disables every tool, which is what an empty list means.
        "--tools".to_string(),
        tools,
    ];
    if let Some(m) = &job.model {
        argv.push("--model".to_string());
        argv.push(m.clone());
    }
    if let Some(b) = job.max_budget_usd {
        argv.push("--max-budget-usd".to_string());
        argv.push(b.to_string());
    }
    argv
}

/// The shell line the dispatched worker runs: the lite worker with stdout to
/// `result.md`, its rc to `exit_code`, then `task.completed` emitted on the
/// worker's own session (`$TERMLINK_WORKER_NAME`, set by dispatch) with
/// `{"result_path": …, "exit_code": N}`. Exits with the worker's rc.
pub(crate) fn lite_worker_shell_line(job: &LiteJob, termlink_bin: &Path) -> String {
    let argv = lite_worker_argv(job)
        .iter()
        .map(|a| shell_escape(a))
        .collect::<Vec<_>>()
        .join(" ");
    let dir = shell_escape(&job.job_dir.to_string_lossy());
    let result = shell_escape(&job.result_path().to_string_lossy());
    let stderr = shell_escape(&job.job_dir.join("stderr.log").to_string_lossy());
    let exit_file = shell_escape(&job.job_dir.join("exit_code").to_string_lossy());
    // JSON is fixed at build time except the rc, which the shell supplies as
    // digits: prefix and suffix are separate shell words around "$LITE_RC".
    let payload_prefix = format!(
        "{{\"result_path\":{},\"exit_code\":",
        serde_json::to_string(&job.result_path().to_string_lossy()).unwrap_or_else(|_| "null".into())
    );
    format!(
        "unset CLAUDECODE; mkdir -p {dir} && {argv} > {result} 2> {stderr} < /dev/null\n\
LITE_RC=$?\n\
echo \"$LITE_RC\" > {exit_file}\n\
{tl} emit \"$TERMLINK_WORKER_NAME\" task.completed --payload {prefix}\"$LITE_RC\"'}}'\n\
exit $LITE_RC",
        tl = shell_escape(&termlink_bin.to_string_lossy()),
        prefix = shell_escape(&payload_prefix),
    )
}

/// Default job dir: `$TMPDIR/termlink-lite/<pid>-<ms>`, never the project tree.
pub(crate) fn default_job_dir() -> PathBuf {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    std::env::temp_dir()
        .join("termlink-lite")
        .join(format!("{}-{ms}", std::process::id()))
}

/// Options for a lite dispatch (what a `--lite` CLI flag would carry).
pub(crate) struct LiteOpts {
    pub job: LiteJob,
    pub timeout: u64,
    pub backend: SpawnBackend,
    pub json_output: bool,
}

/// Run one lite job through the ordinary dispatch path: same registration,
/// same hub collect, same timeout handling; only the worker command differs.
pub(crate) async fn cmd_dispatch_lite(opts: LiteOpts) -> Result<()> {
    let LiteOpts { job, timeout, backend, json_output } = opts;
    if let Err(e) = job.validate() {
        if json_output {
            super::super::json_error_exit(serde_json::json!({"ok": false, "error": e.to_string()}));
        }
        return Err(e);
    }
    std::fs::create_dir_all(&job.job_dir)
        .with_context(|| format!("lite dispatch: cannot create job dir {}", job.job_dir.display()))?;
    let termlink_bin = std::env::current_exe().context("Failed to determine termlink binary path")?;
    let line = lite_worker_shell_line(&job, &termlink_bin);
    if !json_output {
        eprintln!("Lite job: result will be written to {}", job.result_path().display());
    }
    super::cmd_dispatch(super::DispatchOpts {
        count: 1,
        timeout,
        topic: "task.completed".into(),
        name_prefix: Some("lite".into()),
        roles: vec![],
        tags: vec!["lite".into()],
        cap: vec![],
        env_vars: vec![],
        backend,
        workdir: None,
        isolate: false,
        auto_merge: false,
        json_output,
        command: vec!["sh".into(), "-c".into(), line],
        // The model travels as --model on the worker argv, not TERMLINK_MODEL.
        model: None,
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(dir: &Path) -> LiteJob {
        LiteJob {
            prompt: "find X".into(),
            system_prompt: None,
            tools: None,
            model: None,
            max_budget_usd: None,
            job_dir: dir.to_path_buf(),
            worker_bin: "claude".into(),
        }
    }

    fn has_pair(argv: &[String], flag: &str, val: &str) -> bool {
        argv.windows(2).any(|w| w[0] == flag && w[1] == val)
    }

    #[test]
    fn argv_strips_project_context_hooks_mcp_and_skills() {
        let argv = lite_worker_argv(&job(Path::new("/tmp/j")));
        for f in ["--safe-mode", "--strict-mcp-config", "--disable-slash-commands", "--no-session-persistence"] {
            assert!(argv.iter().any(|a| a == f), "missing {f}: {argv:?}");
        }
        // strict MCP with no config = zero MCP servers; a --mcp-config would undo that.
        assert!(!argv.iter().any(|a| a == "--mcp-config"));
        assert_eq!(argv[0], "claude");
        assert_eq!(argv[1], "-p");
    }

    #[test]
    fn argv_replaces_system_prompt_default_and_custom() {
        let argv = lite_worker_argv(&job(Path::new("/tmp/j")));
        assert!(has_pair(&argv, "--system-prompt", DEFAULT_SYSTEM_PROMPT));
        let mut j = job(Path::new("/tmp/j"));
        j.system_prompt = Some("be brief".into());
        assert!(has_pair(&lite_worker_argv(&j), "--system-prompt", "be brief"));
    }

    #[test]
    fn argv_tools_default_read_only_custom_and_none() {
        assert!(has_pair(&lite_worker_argv(&job(Path::new("/j"))), "--tools", "Read,Grep,Glob"));
        let mut j = job(Path::new("/j"));
        j.tools = Some(vec!["Read".into(), "Bash".into()]);
        assert!(has_pair(&lite_worker_argv(&j), "--tools", "Read,Bash"));
        j.tools = Some(vec![]);
        assert!(has_pair(&lite_worker_argv(&j), "--tools", ""));
    }

    #[test]
    fn argv_model_and_budget_only_when_given() {
        let argv = lite_worker_argv(&job(Path::new("/j")));
        assert!(!argv.iter().any(|a| a == "--model" || a == "--max-budget-usd"));
        let mut j = job(Path::new("/j"));
        j.model = Some("haiku".into());
        j.max_budget_usd = Some(0.05);
        let argv = lite_worker_argv(&j);
        assert!(has_pair(&argv, "--model", "haiku"));
        assert!(has_pair(&argv, "--max-budget-usd", "0.05"));
    }

    #[test]
    fn prompt_is_one_argument_never_interpolated() {
        let hostile = "it's \"$(touch /tmp/pwned)\" `id`\nline2; rm -rf x";
        let mut j = job(Path::new("/j"));
        j.prompt = hostile.into();
        let argv = lite_worker_argv(&j);
        assert_eq!(argv[2], hostile, "prompt must be exactly one argv element");
        let line = lite_worker_shell_line(&j, Path::new("/usr/bin/termlink"));
        // Only ever present in its single-quoted form; the end-to-end test
        // below proves the shell hands it over verbatim.
        assert!(line.contains(&shell_escape(hostile)));
        assert_eq!(line.matches("touch /tmp/pwned").count(), 1);
    }

    #[test]
    fn validate_refuses_empty_oversized_and_bad_budget() {
        let mut j = job(Path::new("/j"));
        j.prompt = "  ".into();
        assert!(j.validate().is_err());
        j.prompt = "x".repeat(MAX_PROMPT_BYTES + 1);
        assert!(j.validate().unwrap_err().to_string().contains("limit"));
        j.prompt = "ok".into();
        j.max_budget_usd = Some(0.0);
        assert!(j.validate().is_err());
        j.max_budget_usd = Some(f64::NAN);
        assert!(j.validate().is_err());
        j.max_budget_usd = Some(0.1);
        assert!(j.validate().is_ok());
    }

    #[test]
    fn shell_line_writes_result_file_and_emits_task_completed() {
        let j = job(Path::new("/tmp/lite job"));
        let line = lite_worker_shell_line(&j, Path::new("/opt/tl/termlink"));
        assert!(line.contains("> '/tmp/lite job/result.md'"));
        assert!(line.contains("> '/tmp/lite job/exit_code'"));
        assert!(line.contains("emit \"$TERMLINK_WORKER_NAME\" task.completed --payload"));
        assert!(line.contains("\"result_path\":\"/tmp/lite job/result.md\""));
        assert!(line.trim_end().ends_with("exit $LITE_RC"));
        assert!(line.starts_with("unset CLAUDECODE;"));
    }

    /// End to end with fakes: a fake worker prints its argv, a fake termlink
    /// records its argv. Proves result.md, exit_code and the emitted payload
    /// (valid JSON, rc filled in) without a hub, a session or a real model.
    #[test]
    fn shell_line_runs_end_to_end_with_fake_worker_and_termlink() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let fake_worker = bin.join("fake-claude");
        std::fs::write(&fake_worker, "#!/bin/sh\nprintf '%s\\n' \"$2\"\necho noise >&2\nexit 3\n").unwrap();
        let fake_tl = bin.join("fake-termlink");
        let emitted = tmp.path().join("emitted");
        std::fs::write(&fake_tl, format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n", emitted.display())).unwrap();
        for f in [&fake_worker, &fake_tl] {
            std::fs::set_permissions(f, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let mut j = job(&tmp.path().join("job dir"));
        j.worker_bin = fake_worker.to_string_lossy().into();
        j.prompt = "it's $(echo nope)".into();
        let line = lite_worker_shell_line(&j, &fake_tl);
        let st = std::process::Command::new("sh")
            .arg("-c")
            .arg(&line)
            .env("TERMLINK_WORKER_NAME", "lite-1")
            .status()
            .unwrap();
        assert_eq!(st.code(), Some(3), "worker rc must be the line's rc");

        let result = std::fs::read_to_string(j.result_path()).unwrap();
        assert_eq!(result, "it's $(echo nope)\n", "prompt must arrive verbatim");
        assert_eq!(std::fs::read_to_string(j.job_dir.join("exit_code")).unwrap().trim(), "3");
        assert!(std::fs::read_to_string(j.job_dir.join("stderr.log")).unwrap().contains("noise"));

        let args: Vec<String> = std::fs::read_to_string(&emitted).unwrap().lines().map(String::from).collect();
        assert_eq!(&args[..4], ["emit", "lite-1", "task.completed", "--payload"]);
        let payload: serde_json::Value = serde_json::from_str(&args[4]).unwrap();
        assert_eq!(payload["exit_code"], 3);
        assert_eq!(payload["result_path"], j.result_path().to_string_lossy().as_ref());
    }

    #[test]
    fn default_job_dir_is_outside_the_project() {
        assert!(default_job_dir().starts_with(std::env::temp_dir().join("termlink-lite")));
    }
}
