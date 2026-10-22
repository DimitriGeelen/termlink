//! Starting a tmux session whose environment is built by construction (T-3550, SEV-1 T-3549).
//!
//! `tmux new-session` does two things with its caller's environment. If no server is running,
//! it starts one, and the server takes the client's environment as its GLOBAL environment,
//! which every later session on that server inherits. And if a server is already running, a
//! new session still inherits whatever that server's global environment already holds.
//!
//! T-3445 (2026-10-16) answered the first with a deny-list of project variables. It missed
//! `TASKS_DIR`/`CONTEXT_DIR`, so on 2026-10-22 a dispatch from another project seeded the
//! host's server with them, and every session started on it afterwards worked in that
//! project's task directory (SEV-1 T-3549). It also did nothing about a server that was
//! already polluted.
//!
//! So there are two layers now, neither of which depends on knowing a variable's name:
//! 1. the tmux client runs with [`termlink_session::child_env`] only, so a server it starts
//!    has no project in its global environment;
//! 2. the session's command is wrapped in `env -i <allowed> sh -c '<cmd>'`, so the session's
//!    process sees exactly the allowed environment even on a server someone else polluted.
//!
//! The caller's project variables are not forwarded at all. A session gets project values
//! only from its launch spec (`--env K=V`, `--workdir`), which the shell command carries.

use std::process::Command;

/// Quote `s` for a POSIX shell.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// `/usr/bin/env -i K=V ... /bin/sh -c '<shell_cmd>'`: run `shell_cmd` with exactly `env`,
/// whatever the environment of the process that runs this line.
pub(crate) fn wrap_with_env(env: &[(String, String)], shell_cmd: &str) -> String {
    let mut s = String::from("exec /usr/bin/env -i");
    for (k, v) in env {
        s.push(' ');
        s.push_str(&sh_quote(&format!("{k}={v}")));
    }
    s.push_str(" /bin/sh -c ");
    s.push_str(&sh_quote(shell_cmd));
    s
}

/// Build `tmux [-L socket] new-session -d -s <session> <wrapped shell_cmd>` on `base`
/// (normally `Command::new("tmux")`), with the client's environment replaced by `env`.
/// `socket` is for tests only: production always uses the default server.
pub(crate) fn tmux_new_session_from(
    mut base: Command,
    tmux_session: &str,
    shell_cmd: &str,
    socket: Option<&str>,
    env: &[(String, String)],
) -> Command {
    base.env_clear();
    base.envs(env.iter().map(|(k, v)| (k, v)));
    if let Some(s) = socket {
        base.args(["-L", s]);
    }
    base.args(["new-session", "-d", "-s", tmux_session]);
    base.arg(wrap_with_env(env, shell_cmd));
    base
}

/// The production form: the default tmux server, this process's allowed environment.
pub(crate) fn tmux_new_session(tmux_session: &str, shell_cmd: &str) -> Command {
    tmux_new_session_from(Command::new("tmux"), tmux_session, shell_cmd, None, &termlink_session::child_env::child_env())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(c: &Command) -> Vec<String> {
        c.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }
    fn env_of(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn the_client_env_is_cleared_and_only_the_given_env_is_set() {
        let mut base = Command::new("tmux");
        base.env("ZZ_CANARY_T3550", "/opt/other");
        let c = tmux_new_session_from(base, "tl-x", "sleep 1", None, &env_of(&[("HOME", "/root")]));
        let set: Vec<(String, Option<String>)> = c
            .get_envs()
            .map(|(k, v)| (k.to_string_lossy().into_owned(), v.map(|v| v.to_string_lossy().into_owned())))
            .collect();
        assert_eq!(set, [("HOME".to_string(), Some("/root".to_string()))], "{set:?}");
        let a = args(&c);
        assert_eq!(&a[..4], &["new-session", "-d", "-s", "tl-x"]);
        assert!(!a.iter().any(|x| x == "-e"), "nothing is forwarded with -e any more");
    }

    #[test]
    fn the_session_command_is_wrapped_in_env_i() {
        let c = tmux_new_session_from(Command::new("tmux"), "s", "echo 'hi'", None, &env_of(&[("HOME", "/root")]));
        let last = args(&c).pop().unwrap();
        assert_eq!(last, r"exec /usr/bin/env -i 'HOME=/root' /bin/sh -c 'echo '\''hi'\'''");
    }

    #[test]
    fn socket_is_only_added_when_asked() {
        let prod = tmux_new_session_from(Command::new("tmux"), "s", "true", None, &[]);
        assert!(!args(&prod).iter().any(|a| a == "-L"));
        let t = tmux_new_session_from(Command::new("tmux"), "s", "true", Some("priv"), &[]);
        assert_eq!(&args(&t)[..2], &["-L", "priv"]);
    }

    /// Live, on PRIVATE servers only (`-L`), never the host's default one, and killed only
    /// through that socket (055 T-537: a test that let tmux fall back to the default socket
    /// killed the whole fleet). Canary names nobody listed, and two scenarios: a server our
    /// spawn starts, and a server someone else already polluted. The control shows the test can
    /// see a leak at all.
    #[test]
    fn spawned_sessions_carry_no_caller_project_live() {
        if Command::new("tmux").arg("-V").output().map(|o| !o.status.success()).unwrap_or(true) {
            eprintln!("skip: tmux not installed");
            return;
        }
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("t3550-{pid}"));
        std::fs::create_dir_all(&dir).unwrap();
        let global_has = |sock: &str, var: &str| {
            let o = Command::new("tmux").args(["-L", sock, "show-environment", "-g", var]).output().unwrap();
            String::from_utf8_lossy(&o.stdout).trim().starts_with(&format!("{var}="))
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
            b.env("ZZ_CANARY_T3550", "/opt/other").env("TASKS_DIR", "/opt/other/.tasks");
            b
        };
        // What the session's own process saw, written to a file it leaves behind.
        let wait_for = |p: &std::path::Path| {
            for _ in 0..100 {
                if let Ok(s) = std::fs::read_to_string(p) {
                    if s.contains("PATH=") {
                        return s;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            panic!("the session never wrote {}", p.display());
        };
        let env = termlink_session::child_env::child_env();

        // Control: the pre-T-3445 shape seeds the server with the caller's variables.
        let ctl = format!("tl-t3550-ctl-{pid}");
        let st = leaking_caller().args(["-L", &ctl, "new-session", "-d", "-s", "c", "sleep 30"]).status().unwrap();
        assert!(st.success());
        let control_leaked = global_has(&ctl, "ZZ_CANARY_T3550");

        // Scenario 1: our spawn starts the server.
        let fresh = format!("tl-t3550-{pid}");
        let out1 = dir.join("fresh.env");
        let cmd1 = format!("env > {}; sleep 30", out1.display());
        assert!(tmux_new_session_from(leaking_caller(), "s", &cmd1, Some(&fresh), &env).status().unwrap().success());
        let fresh_global = global_has(&fresh, "ZZ_CANARY_T3550") || global_has(&fresh, "TASKS_DIR");
        let seen1 = wait_for(&out1);

        // Scenario 2: the control's server is already polluted; spawn a session onto it.
        let out2 = dir.join("polluted.env");
        let cmd2 = format!("env > {}; sleep 30", out2.display());
        assert!(tmux_new_session_from(leaking_caller(), "p", &cmd2, Some(&ctl), &env).status().unwrap().success());
        let seen2 = wait_for(&out2);

        kill(&ctl);
        kill(&fresh);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(control_leaked, "control: the unscrubbed spawn should leak, else this test cannot see a leak");
        assert!(!fresh_global, "a server our spawn starts must not carry the caller's variables");
        for (name, seen) in [("fresh server", &seen1), ("polluted server", &seen2)] {
            assert!(!seen.contains("ZZ_CANARY_T3550"), "{name}: canary reached the session:\n{seen}");
            assert!(!seen.contains("TASKS_DIR"), "{name}: TASKS_DIR reached the session:\n{seen}");
        }
    }
}
