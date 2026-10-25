//! The environment every process TermLink starts for another session (T-3550, SEV-1 T-3549).
//!
//! T-3445 tried to keep projects out of spawned processes with a deny-list: remove
//! `PROJECT_ROOT` and a few others, pass the rest through. On 2026-10-22 that list was missing
//! `TASKS_DIR`/`CONTEXT_DIR`, and a dispatch from another project seeded the host's tmux server
//! with them. Every later session on that server ran with that project's task directory.
//! A list of names to remove can only be as complete as its author's memory, so it fails
//! whenever a new project variable appears.
//!
//! This builds the child environment by construction instead: start empty, copy only the host
//! settings named in [`HOST_ENV_ALLOW`] (plus `LC_*`), and set a fixed `PATH`. Nothing that
//! names a project, an agent's sockets or credentials survives unless it is added here
//! deliberately. Project values reach a session only explicitly, through the launch spec
//! (`--env K=V`, `--workdir`), never by inheritance from whichever agent happened to spawn it.

use std::path::Path;

/// Host-level settings a child may inherit. Exact names; `LC_*` is matched separately.
/// Adding a name here is a decision that it can never identify a project or carry a secret.
pub const HOST_ENV_ALLOW: &[&str] = &[
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "TERM",
    "COLORTERM",
    "LANG",
    "LANGUAGE",
    "TZ",
    "TMPDIR",
    "TMUX_TMPDIR",
    "XDG_RUNTIME_DIR",
    "TERMLINK_RUNTIME_DIR",
];

/// System directories every child gets on `PATH`, after the per-user ones.
const SYSTEM_PATH: &[&str] = &["/usr/local/sbin", "/usr/local/bin", "/usr/sbin", "/usr/bin", "/sbin", "/bin"];

/// True when a variable of this name may pass from the caller to a child.
pub fn is_allowed(name: &str) -> bool {
    HOST_ENV_ALLOW.contains(&name) || name.starts_with("LC_")
}

/// The fixed `PATH`: the directory of the running binary (so the child finds this termlink),
/// then `$HOME/.local/bin` and `$HOME/.cargo/bin`, then the system directories. The caller's
/// own `PATH` is not used: a project can put its own `bin/` first, and a child would then run
/// that project's `fw` (T-3549 review, antigravity).
pub fn fixed_path(home: Option<&str>, exe_dir: Option<&Path>) -> String {
    let mut dirs: Vec<String> = Vec::new();
    if let Some(d) = exe_dir {
        dirs.push(d.to_string_lossy().into_owned());
    }
    if let Some(h) = home.filter(|h| !h.is_empty()) {
        dirs.push(format!("{h}/.local/bin"));
        dirs.push(format!("{h}/.cargo/bin"));
    }
    dirs.extend(SYSTEM_PATH.iter().map(|s| s.to_string()));
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|d| seen.insert(d.clone()));
    dirs.join(":")
}

/// The child environment for a caller whose environment is `vars`: the allowed host settings
/// plus the fixed `PATH`, in a stable order.
pub fn child_env_from<I, K, V>(vars: I, exe_dir: Option<&Path>) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let mut out: Vec<(String, String)> = vars
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .filter(|(k, _)| is_allowed(k))
        .collect();
    out.sort();
    out.dedup_by(|a, b| a.0 == b.0);
    let home = out.iter().find(|(k, _)| k == "HOME").map(|(_, v)| v.clone());
    out.push(("PATH".to_string(), fixed_path(home.as_deref(), exe_dir)));
    out
}

/// The child environment for this process.
pub fn child_env() -> Vec<(String, String)> {
    let exe = std::env::current_exe().ok();
    child_env_from(std::env::vars(), exe.as_deref().and_then(Path::parent))
}

/// Replace `cmd`'s environment with [`child_env`]: nothing the caller has reaches the child
/// unless the allow-list names it.
pub fn apply(cmd: &mut std::process::Command) {
    cmd.env_clear();
    cmd.envs(child_env());
}

/// The environment for a restarted hub: [`child_env`] plus the caller's `TERMLINK_*` settings,
/// which carry hub tuning (`TERMLINK_MAX_CONNECTIONS`, `TERMLINK_RATE_LIMIT_PER_SEC`, ...) that
/// is too long and too often extended to list. That is safe only because the hub starts no
/// child processes (T-3550, verified by grep over `termlink-hub`): nothing it inherits can
/// reach a session. Project variables still never reach it.
pub fn hub_env() -> Vec<(String, String)> {
    let exe = std::env::current_exe().ok();
    hub_env_from(std::env::vars().collect(), exe.as_deref().and_then(Path::parent))
}

/// [`hub_env`] for a caller whose environment is `vars`.
pub fn hub_env_from(vars: Vec<(String, String)>, exe_dir: Option<&Path>) -> Vec<(String, String)> {
    let mut env = child_env_from(vars.clone(), exe_dir);
    for (k, v) in vars {
        if k.starts_with("TERMLINK_") && !env.iter().any(|(e, _)| e == &k) {
            env.push((k, v));
        }
    }
    env
}

/// [`apply`] for a hub process: see [`hub_env`].
pub fn apply_hub(cmd: &mut std::process::Command) {
    cmd.env_clear();
    cmd.envs(hub_env());
}

/// Variables that name the project a process works in (T-3551). `FRAMEWORK_ROOT` is not one:
/// shared-tooling mode points it outside the project on purpose.
pub const PROJECT_VARS: &[&str] = &["PROJECT_ROOT", "TASKS_DIR", "CONTEXT_DIR", "CLAUDE_PROJECT_DIR", "_FW_PATHS_DERIVED_BY"];

/// The project `path` lies in: the nearest ancestor (or itself) holding `.framework.yaml`,
/// else the nearest holding `.git`; a git worktree counts as its main checkout. `None` outside
/// any project. `scripts/lib/env-leak-scan.py` applies the same rule.
pub fn project_of(path: &Path) -> Option<std::path::PathBuf> {
    let has = |marker: &str| path.ancestors().find(|a| a.join(marker).exists()).map(Path::to_path_buf);
    has(".framework.yaml").or_else(|| has(".git")).map(|root| main_checkout(&root))
}

/// A worktree's `.git` is a file `gitdir: <main>/.git/worktrees/<name>`; map it to `<main>`.
fn main_checkout(root: &Path) -> std::path::PathBuf {
    let g = root.join(".git");
    if g.is_file()
        && let Ok(text) = std::fs::read_to_string(&g) {
            let line = text.lines().next().unwrap_or("").trim();
            if let Some(dir) = line.strip_prefix("gitdir:")
                && let Some((main, _)) = dir.trim().split_once("/.git/worktrees/") {
                    return std::path::PathBuf::from(main);
                }
        }
    root.to_path_buf()
}

/// A project variable whose value lies in a different project than the process's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignVar {
    pub name: String,
    pub value: String,
    /// The project the value belongs to.
    pub value_project: std::path::PathBuf,
    /// The project the process works in (from its working directory).
    pub own_project: std::path::PathBuf,
}

/// The project variables in `vars` that point into another project than the one `cwd` is in
/// (T-3551, SEV-1 T-3549). Nothing is judged when `cwd` is outside every project, and a value
/// outside every project (a test's temp dir) is never foreign: unknown is not a finding.
pub fn foreign_project_vars<I, K, V>(vars: I, cwd: &Path) -> Vec<ForeignVar>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let Some(own) = project_of(cwd) else { return Vec::new() };
    let mut out = Vec::new();
    for (k, v) in vars {
        let (k, v): (String, String) = (k.into(), v.into());
        if !PROJECT_VARS.contains(&k.as_str()) || !v.starts_with('/') {
            continue;
        }
        if let Some(vp) = project_of(Path::new(&v))
            && vp != own {
                out.push(ForeignVar { name: k, value: v, value_project: vp, own_project: own.clone() });
            }
    }
    out
}

/// The refusal text for a process that carries foreign project variables.
pub fn foreign_refusal(found: &[ForeignVar]) -> String {
    let mut s = String::from(
        "refusing to register: this process carries another project's variables (T-3551, SEV-1 T-3549):\n",
    );
    for f in found {
        s.push_str(&format!(
            "  {}={} belongs to {}, but this process works in {}\n",
            f.name,
            f.value,
            f.value_project.display(),
            f.own_project.display()
        ));
    }
    s.push_str("Unset them (or start from the project they name) and retry. They leaked in from whatever started this shell.");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_projects() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        for p in [&a, &b] {
            std::fs::create_dir_all(p.join("sub")).unwrap();
            std::fs::write(p.join(".framework.yaml"), "").unwrap();
        }
        (d, a, b)
    }

    #[test]
    fn another_projects_paths_are_foreign() {
        let (_d, a, b) = two_projects();
        let found = foreign_project_vars(
            [
                ("TASKS_DIR", format!("{}/.tasks", b.display())),
                ("CONTEXT_DIR", format!("{}/.context", b.display())),
                ("PROJECT_ROOT", a.display().to_string()),
                ("FRAMEWORK_ROOT", format!("{}/.agentic-framework", b.display())),
                ("ZZ_OTHER", b.display().to_string()),
            ],
            &a.join("sub"),
        );
        let names: Vec<&str> = found.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["TASKS_DIR", "CONTEXT_DIR"], "{found:?}");
        assert_eq!(found[0].value_project, b);
        assert_eq!(found[0].own_project, a);
        let msg = foreign_refusal(&found);
        assert!(msg.contains("TASKS_DIR=") && msg.contains(&a.display().to_string()), "{msg}");
    }

    #[test]
    fn unknown_is_never_foreign() {
        let (_d, a, _b) = two_projects();
        // Outside every project. Not a temp dir: TMPDIR may itself lie inside a project.
        let tmp = std::path::PathBuf::from("/nonexistent-t3551/scratch");
        // A value outside every project (a test's temp dir).
        assert!(foreign_project_vars([("TASKS_DIR", tmp.display().to_string())], &a).is_empty());
        // A process outside every project.
        assert!(foreign_project_vars([("TASKS_DIR", a.display().to_string())], &tmp).is_empty());
        // A relative value.
        assert!(foreign_project_vars([("TASKS_DIR", ".tasks")], &a).is_empty());
    }

    #[test]
    fn a_worktree_is_its_main_checkout() {
        let (d, a, b) = two_projects();
        let wt = d.path().join("a-wt");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join(".framework.yaml"), "").unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}/.git/worktrees/wt\n", a.display())).unwrap();
        assert_eq!(project_of(&wt), Some(a.clone()));
        assert!(foreign_project_vars([("PROJECT_ROOT", a.display().to_string())], &wt).is_empty());
        assert_eq!(foreign_project_vars([("PROJECT_ROOT", b.display().to_string())], &wt).len(), 1);
    }

    #[test]
    fn unknown_variables_never_pass() {
        // Canary names nobody has heard of: a deny-list would let every one of them through.
        let env = child_env_from(
            [
                ("HOME", "/root"),
                ("ZZ_CANARY_T3550_PROJECT", "/opt/other"),
                ("TASKS_DIR", "/opt/other/.tasks"),
                ("CONTEXT_DIR", "/opt/other/.context"),
                ("PROJECT_ROOT", "/opt/other"),
                ("TERMLINK_SIDECAR_SOCKET", "/x.sock"),
                ("ANTHROPIC_API_KEY", "secret"),
                ("TMUX", "/tmp/tmux-0/default,1,0"),
            ],
            None,
        );
        let names: Vec<&str> = env.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["HOME", "PATH"], "only allowed names and the fixed PATH survive");
    }

    #[test]
    fn host_settings_and_locale_pass_unchanged() {
        let env = child_env_from(
            [("LANG", "C.UTF-8"), ("LC_ALL", "C"), ("TERMLINK_RUNTIME_DIR", "/var/lib/termlink"), ("TERM", "xterm")],
            None,
        );
        assert!(env.contains(&("LC_ALL".into(), "C".into())));
        assert!(env.contains(&("TERMLINK_RUNTIME_DIR".into(), "/var/lib/termlink".into())));
        assert!(env.contains(&("LANG".into(), "C.UTF-8".into())));
    }

    #[test]
    fn path_is_fixed_not_inherited() {
        let env = child_env_from(
            [("HOME", "/root"), ("PATH", "/opt/other/.agentic-framework/bin:/usr/bin")],
            Some(Path::new("/root/.cargo/bin")),
        );
        let path = &env.iter().find(|(k, _)| k == "PATH").unwrap().1;
        assert!(!path.contains("/opt/other"), "a project's bin dir must not reach the child: {path}");
        assert!(path.starts_with("/root/.cargo/bin:/root/.local/bin:"), "{path}");
        assert_eq!(path.matches("/root/.cargo/bin").count(), 1, "duplicates removed: {path}");
        assert!(path.ends_with("/usr/bin:/sbin:/bin"));
    }

    #[test]
    fn apply_clears_before_setting() {
        let mut c = std::process::Command::new("true");
        c.env("ZZ_CANARY_T3550", "1");
        apply(&mut c);
        assert!(
            !c.get_envs().any(|(k, v)| k == "ZZ_CANARY_T3550" && v.is_some()),
            "an explicitly set caller variable must not survive apply()"
        );
    }

    #[test]
    fn a_hub_keeps_its_tuning_but_not_a_project() {
        let vars = [("TERMLINK_MAX_CONNECTIONS", "64"), ("PROJECT_ROOT", "/opt/other"), ("TASKS_DIR", "/x")]
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .to_vec();
        let env = hub_env_from(vars, None);
        assert!(env.contains(&("TERMLINK_MAX_CONNECTIONS".into(), "64".into())));
        assert!(!env.iter().any(|(k, _)| k == "PROJECT_ROOT" || k == "TASKS_DIR"), "{env:?}");
    }

    /// End to end on a real child: a variable the caller has, under a name nobody listed,
    /// is absent in the child.
    #[test]
    fn a_real_child_sees_none_of_the_callers_project() {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "env"]);
        c.env("ZZ_CANARY_T3550_LIVE", "/opt/other");
        c.env("TASKS_DIR", "/opt/other/.tasks");
        apply(&mut c);
        let out = c.output().expect("sh runs");
        let s = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success());
        assert!(s.lines().any(|l| l.starts_with("PATH=")), "control: the child printed its env:\n{s}");
        assert!(!s.contains("ZZ_CANARY_T3550_LIVE"), "{s}");
        assert!(!s.contains("TASKS_DIR"), "{s}");
    }
}
