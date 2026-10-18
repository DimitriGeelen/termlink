//! Starts a tmux session without leaking the caller's environment into the tmux server's global environment.
//! Starting a tmux session without leaking the caller's project into the tmux server (T-3445).
//!
//! `tmux new-session` run with the caller's environment does two things when no server is
//! running yet: it starts the server, and the server takes the client's environment as its
//! GLOBAL environment, which every session started on that server later inherits. On
//! 2026-10-16 a dispatch from one project started the host's default server, and five fleet
//! agents plus this project's main session then ran with that project's PROJECT_ROOT and
//! FRAMEWORK_ROOT, acting on the wrong repository (reported by 055, its T-536).
//!
//! So the tmux client runs WITHOUT the project-scoped variables, and each one the caller has
//! is handed to the new session alone with `new-session -e K=V`. The session sees exactly what
//! it saw before; the server's global environment never carries a project.

use std::process::Command;

/// Variables that name a project or one agent's sockets. They belong to a session, never to a
/// tmux server shared by every project on the host.
pub(crate) const PROJECT_SCOPED_ENV: &[&str] = &[
    "PROJECT_ROOT",
    "FRAMEWORK_ROOT",
    "_FW_PATHS_DERIVED_BY",
    "FW_ROUTED_FROM", // set by newer AEF; 055 strips it too (T-3446)
    "CLAUDE_PROJECT_DIR",
];

/// T-3463: one agent's sidecar sockets. Unlike PROJECT_SCOPED_ENV they are not passed on to
/// the new session at all: a session joins its sidecar only by being LAUNCHED with the socket
/// (operator ruling P7-1 A). `tl-claude.sh` sets it explicitly in the command it injects, so
/// forwarding it here only made every spawned session (and dispatched worker) able to take
/// the spawning agent's mail.
pub(crate) const LAUNCH_SCOPED_ENV: &[&str] = &["TERMLINK_SIDECAR_SOCKET", "TERMLINK_SIDECAR_STATUS_SOCKET"];

/// Build `tmux [-L socket] new-session -d -s <session> [-e K=V ...] <shell_cmd>` on `base`
/// (normally `Command::new("tmux")`), with every project-scoped variable removed from the tmux
/// client's own environment and the ones `lookup` finds passed to the session with `-e`.
/// `socket` is for tests only: production always uses the default server.
pub(crate) fn tmux_new_session_from(
    mut base: Command,
    tmux_session: &str,
    shell_cmd: &str,
    socket: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Command {
    if let Some(s) = socket {
        base.args(["-L", s]);
    }
    base.args(["new-session", "-d", "-s", tmux_session]);
    for var in PROJECT_SCOPED_ENV {
        base.env_remove(var);
        if let Some(v) = lookup(var) {
            base.arg("-e").arg(format!("{var}={v}"));
        }
    }
    for var in LAUNCH_SCOPED_ENV {
        base.env_remove(var);
    }
    base.arg(shell_cmd);
    base
}

/// The production form: the default tmux server, the caller's real environment.
pub(crate) fn tmux_new_session(tmux_session: &str, shell_cmd: &str) -> Command {
    tmux_new_session_from(Command::new("tmux"), tmux_session, shell_cmd, None, |k| {
        std::env::var(k).ok().filter(|v| !v.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn args(c: &Command) -> Vec<String> {
        c.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }
    fn removed(c: &Command, k: &str) -> bool {
        c.get_envs().any(|(key, val)| key == OsStr::new(k) && val.is_none())
    }

    #[test]
    fn project_vars_leave_the_client_env_and_go_to_the_session() {
        let c = tmux_new_session_from(Command::new("tmux"), "tl-x", "sleep 1", None, |k| match k {
            "PROJECT_ROOT" => Some("/opt/p".into()),
            "FRAMEWORK_ROOT" => Some("/opt/p/.agentic-framework".into()),
            _ => None,
        });
        for var in PROJECT_SCOPED_ENV {
            assert!(removed(&c, var), "{var} must be removed from the tmux client env");
        }
        let a = args(&c);
        assert_eq!(&a[..4], &["new-session", "-d", "-s", "tl-x"]);
        assert!(a.windows(2).any(|w| w == ["-e", "PROJECT_ROOT=/opt/p"]));
        assert!(a.windows(2).any(|w| w == ["-e", "FRAMEWORK_ROOT=/opt/p/.agentic-framework"]));
        assert_eq!(a.last().map(String::as_str), Some("sleep 1"), "the shell command stays last");
    }

    #[test]
    fn t3463_sidecar_sockets_are_never_forwarded_to_the_session() {
        let c = tmux_new_session_from(Command::new("tmux"), "tl-s", "true", None, |k| match k {
            "TERMLINK_SIDECAR_SOCKET" => Some("/var/lib/termlink/sidecar/a.send.sock".into()),
            "TERMLINK_SIDECAR_STATUS_SOCKET" => Some("/var/lib/termlink/sidecar/a.status.sock".into()),
            "PROJECT_ROOT" => Some("/opt/p".into()),
            _ => None,
        });
        for var in LAUNCH_SCOPED_ENV {
            assert!(removed(&c, var), "{var} must be removed from the tmux client env");
            assert!(!args(&c).iter().any(|a| a.starts_with(&format!("{var}="))), "{var} must not be passed with -e");
        }
        assert!(args(&c).windows(2).any(|w| w == ["-e", "PROJECT_ROOT=/opt/p"]), "project vars still go to the session");
    }

    #[test]
    fn fw_routed_from_is_scoped_to_the_session() {
        let c = tmux_new_session_from(Command::new("tmux"), "tl-z", "true", None, |k| {
            (k == "FW_ROUTED_FROM").then(|| "/opt/other".to_string())
        });
        assert!(removed(&c, "FW_ROUTED_FROM"));
        assert!(args(&c).windows(2).any(|w| w == ["-e", "FW_ROUTED_FROM=/opt/other"]));
    }

    #[test]
    fn absent_vars_are_not_invented() {
        let c = tmux_new_session_from(Command::new("tmux"), "tl-y", "true", None, |_| None);
        assert!(!args(&c).iter().any(|a| a == "-e"), "no -e when the caller has none of them");
        assert!(removed(&c, "PROJECT_ROOT"));
    }

    #[test]
    fn socket_is_only_added_when_asked() {
        let prod = tmux_new_session_from(Command::new("tmux"), "s", "true", None, |_| None);
        assert!(!args(&prod).iter().any(|a| a == "-L"));
        let t = tmux_new_session_from(Command::new("tmux"), "s", "true", Some("priv"), |_| None);
        assert_eq!(&args(&t)[..2], &["-L", "priv"]);
    }

    /// Live, on a PRIVATE server only (`-L`), never the host's default one, and killed only
    /// through that socket (055 T-537: a test that let tmux fall back to the default socket
    /// killed the whole fleet). The control shows the test can see a leak at all.
    #[test]
    fn a_server_started_by_spawn_carries_no_project_live() {
        if Command::new("tmux").arg("-V").output().map(|o| !o.status.success()).unwrap_or(true) {
            eprintln!("skip: tmux not installed");
            return;
        }
        let pid = std::process::id();
        let global_has = |sock: &str| {
            let o = Command::new("tmux").args(["-L", sock, "show-environment", "-g", "PROJECT_ROOT"]).output().unwrap();
            String::from_utf8_lossy(&o.stdout).trim() == "PROJECT_ROOT=/leak"
        };
        // kill-server leaves the socket file behind; remove it by the path the server reports.
        let kill = |sock: &str| {
            let p = Command::new("tmux").args(["-L", sock, "display-message", "-p", "#{socket_path}"]).output();
            let _ = Command::new("tmux").args(["-L", sock, "kill-server"]).output();
            if let Ok(o) = p {
                let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if path.ends_with(sock) {
                    let _ = std::fs::remove_file(path);
                }
            }
        };
        let leaking_caller = || {
            let mut b = Command::new("tmux");
            b.env("PROJECT_ROOT", "/leak");
            b
        };

        // Control: the old shape (plain new-session with the caller's env) seeds the server.
        let ctl = format!("tl-t3445-ctl-{pid}");
        let st = leaking_caller().args(["-L", &ctl, "new-session", "-d", "-s", "c", "sleep 30"]).status().unwrap();
        assert!(st.success());
        let control_leaked = global_has(&ctl);
        kill(&ctl);
        assert!(control_leaked, "control: the unscrubbed spawn should leak, else this test cannot see a leak");

        // The fix: same caller, through the builder.
        let sock = format!("tl-t3445-{pid}");
        let st = tmux_new_session_from(leaking_caller(), "s", "sleep 30", Some(&sock), |k| {
            (k == "PROJECT_ROOT").then(|| "/leak".to_string())
        })
        .status()
        .unwrap();
        assert!(st.success());
        let leaked = global_has(&sock);
        let o = Command::new("tmux").args(["-L", &sock, "show-environment", "-t", "s", "PROJECT_ROOT"]).output().unwrap();
        let session_has = String::from_utf8_lossy(&o.stdout).trim() == "PROJECT_ROOT=/leak";
        kill(&sock);
        assert!(!leaked, "the server's global env must not carry the caller's PROJECT_ROOT");
        assert!(session_has, "the new session must still see the caller's PROJECT_ROOT");
    }
}
