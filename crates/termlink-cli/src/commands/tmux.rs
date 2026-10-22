//! `termlink tmux socket|list|attach` — reach the per-project tmux servers by PROJECT PATH (T-3552).
//!
//! Since T-3552 a TermLink session runs on its project's own tmux server (`tmux -L tl-<slug>-<hash8>`,
//! [`termlink_session::tmux_server`]), not on the host's default one. These commands are how scripts
//! and cockpits find it without knowing the naming rule: 055-agentic-fleet-cockpit asked for a
//! project path, not a socket path. Every name handed to `tmux -L` here was built or validated by
//! [`termlink_session::tmux_server`]; nothing else reaches tmux. Interface:
//! `docs/reports/T-3552-per-project-tmux-interface.md`.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use termlink_session::tmux_server::{is_valid_server_name, server_name_for, HOST_SERVER};

/// This uid's tmux socket directory: `$TMUX_TMPDIR` (else `/tmp`) + `tmux-<uid>`.
fn socket_dir() -> PathBuf {
    let base = std::env::var_os("TMUX_TMPDIR").filter(|v| !v.is_empty()).unwrap_or_else(|| "/tmp".into());
    // SAFETY: getuid has no preconditions and cannot fail.
    let uid = unsafe { libc::getuid() };
    PathBuf::from(base).join(format!("tmux-{uid}"))
}

/// The TermLink servers present in this uid's socket directory, sorted. Names that fail
/// validation (a human's own `-L` servers, `default`) are never returned.
fn present_servers() -> Vec<String> {
    present_servers_in(&socket_dir())
}

fn present_servers_in(sdir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(sdir)
        .map(|it| it.filter_map(|e| e.ok()?.file_name().into_string().ok()).filter(|n| is_valid_server_name(n)).collect())
        .unwrap_or_default();
    v.sort();
    v
}

fn resolve_dir(path: Option<&Path>) -> Result<PathBuf> {
    match path {
        Some(p) => std::fs::canonicalize(p).with_context(|| format!("no such directory: {}", p.display())),
        None => std::env::current_dir().context("Cannot read the current directory"),
    }
}

/// `tmux -L <server> <args>`; `None` when the server does not answer.
fn tmux_on(server: &str, args: &[&str]) -> Option<String> {
    debug_assert!(is_valid_server_name(server));
    let o = Command::new("tmux").args(["-L", server]).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// `termlink tmux socket <path>`: print the `-L` name of the project's server. Exit 2 when the
/// path lies in no project (its sessions would be on `tl-host`, which is not a project server).
pub(crate) fn cmd_tmux_socket(path: &Path) -> Result<()> {
    let dir = resolve_dir(Some(path))?;
    let server = server_name_for(&dir);
    if server == HOST_SERVER {
        eprintln!("{} is in no project (no .framework.yaml or .git above it); its sessions use {HOST_SERVER}", dir.display());
        std::process::exit(2);
    }
    println!("{server}");
    Ok(())
}

/// A registered TermLink session, as `tmux list` matches it: pid, id, display name, tmux server.
type Reg = (u32, String, String, Option<String>);

/// The registration for a tmux session: one on the same server whose spawn name `tl-<display>`
/// is this session (T-3552 metadata), else one whose pid is the pane's own process.
fn registered_id(regs: &[Reg], server: &str, session: &str, pane_pids: &[u32]) -> Option<String> {
    regs.iter()
        .find(|(_, _, name, srv)| srv.as_deref() == Some(server) && session == format!("tl-{name}"))
        .or_else(|| regs.iter().find(|(pid, ..)| pane_pids.contains(pid)))
        .map(|(_, id, ..)| id.clone())
}

/// One row of `termlink tmux list`.
fn sessions_on(server: &str, project: Option<&Path>, regs: &[Reg]) -> serde_json::Value {
    let present = socket_dir().join(server).exists();
    let Some(sessions) = tmux_on(server, &["list-sessions", "-F", "#{session_name}\t#{session_attached}\t#{session_created}"]) else {
        return serde_json::json!({
            "server": server, "project": project.map(|p| p.display().to_string()),
            "state": if present { "dead" } else { "absent" }, "sessions": [],
        });
    };
    let panes = tmux_on(server, &["list-panes", "-a", "-F", "#{session_name}\t#{pane_pid}\t#{pane_current_command}"]).unwrap_or_default();
    let mut rows = Vec::new();
    for line in sessions.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let pane_rows: Vec<serde_json::Value> = panes
            .lines()
            .filter_map(|l| {
                let p: Vec<&str> = l.split('\t').collect();
                (p.len() >= 3 && p[0] == f[0]).then(|| serde_json::json!({"pid": p[1].parse::<u32>().unwrap_or(0), "cmd": p[2]}))
            })
            .collect();
        let pids: Vec<u32> = pane_rows.iter().filter_map(|p| p["pid"].as_u64().map(|v| v as u32)).collect();
        rows.push(serde_json::json!({
            "session": f[0],
            "registered_session_id": registered_id(regs, server, f[0], &pids),
            "attached": f[1].parse::<u32>().unwrap_or(0) > 0,
            "created": f[2].parse::<u64>().unwrap_or(0),
            "panes": pane_rows,
        }));
    }
    serde_json::json!({
        "server": server, "project": project.map(|p| p.display().to_string()),
        "state": "live", "sessions": rows,
    })
}

/// `termlink tmux list [<path> | --all] [--json]`.
pub(crate) fn cmd_tmux_list(path: Option<&Path>, all: bool, json: bool) -> Result<()> {
    let regs: Vec<Reg> = termlink_session::manager::list_sessions(false)
        .map(|v| v.into_iter().map(|r| (r.pid, r.id.to_string(), r.display_name, r.metadata.tmux_server)).collect())
        .unwrap_or_default();
    let servers: Vec<serde_json::Value> = if all {
        present_servers().iter().map(|s| sessions_on(s, None, &regs)).collect()
    } else {
        let dir = resolve_dir(path)?;
        let project = termlink_session::child_env::project_of(&dir);
        vec![sessions_on(&server_name_for(&dir), project.as_deref(), &regs)]
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"ok": true, "servers": servers}))?);
        return Ok(());
    }
    for s in &servers {
        let state = s["state"].as_str().unwrap_or("?");
        let proj = s["project"].as_str().map(|p| format!("  ({p})")).unwrap_or_default();
        println!("{}  [{state}]{proj}", s["server"].as_str().unwrap_or("?"));
        for r in s["sessions"].as_array().into_iter().flatten() {
            let att = if r["attached"].as_bool() == Some(true) { "attached" } else { "detached" };
            let pids: Vec<String> = r["panes"].as_array().into_iter().flatten().map(|p| p["pid"].to_string()).collect();
            println!("  {}  {att}  pane pid(s) {}", r["session"].as_str().unwrap_or("?"), pids.join(","));
        }
    }
    if servers.is_empty() {
        println!("no TermLink tmux servers in {}", socket_dir().display());
    }
    Ok(())
}

/// `termlink tmux attach <path> [<session>]`: replace this process with a tmux client on the
/// project's server. Without a session, attaches when there is exactly one, else lists them.
pub(crate) fn cmd_tmux_attach(path: &Path, session: Option<&str>) -> Result<()> {
    use std::os::unix::process::CommandExt;
    let dir = resolve_dir(Some(path))?;
    let server = server_name_for(&dir);
    let target = match session {
        Some(s) => s.to_string(),
        None => {
            let names: Vec<String> = tmux_on(&server, &["list-sessions", "-F", "#{session_name}"])
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect();
            match names.as_slice() {
                [one] => one.clone(),
                [] => {
                    eprintln!("no sessions on {server} ({})", dir.display());
                    std::process::exit(1);
                }
                many => {
                    eprintln!("{} sessions on {server}; name one: termlink tmux attach {} <session>", many.len(), dir.display());
                    for n in many {
                        eprintln!("  {n}");
                    }
                    std::process::exit(1);
                }
            }
        }
    };
    let err = Command::new("tmux").args(["-L", &server, "attach-session", "-t", &target]).exec();
    Err(anyhow::anyhow!("could not exec tmux: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_is_matched_to_its_registration() {
        let regs: Vec<Reg> = vec![
            (10, "tl-aaa".into(), "w1".into(), Some("tl-p-0123abcd".into())),
            (20, "tl-bbb".into(), "w1".into(), Some("tl-q-0123abcd".into())),
            (30, "tl-ccc".into(), "shell".into(), None),
        ];
        assert_eq!(registered_id(&regs, "tl-p-0123abcd", "tl-w1", &[99]).as_deref(), Some("tl-aaa"));
        assert_eq!(registered_id(&regs, "tl-q-0123abcd", "tl-w1", &[99]).as_deref(), Some("tl-bbb"), "same name, other server");
        assert_eq!(registered_id(&regs, "tl-p-0123abcd", "other", &[30]).as_deref(), Some("tl-ccc"), "pid fallback");
        assert_eq!(registered_id(&regs, "tl-p-0123abcd", "other", &[99]), None);
    }

    #[test]
    fn foreign_socket_names_are_never_listed() {
        let d = tempfile::tempdir().unwrap();
        for n in ["default", "tl-p-0123abcd", "tl-host", "mine", "tl-x"] {
            std::fs::write(d.path().join(n), "").unwrap();
        }
        assert_eq!(present_servers_in(d.path()), ["tl-host", "tl-p-0123abcd"]);
    }
}
