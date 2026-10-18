//! Supporting tools (supervise script and systemd template) shipped inside the sidecar binary.
//! The sidecar's supporting tools, shipped inside the binary (PH-2 row P9,
//! operator ruling P9-1 A; R-39, SI-27).
//!
//! The supervise script and the systemd template are the files tested in the
//! repository (`scripts/sidecar-supervise.sh`, `systemd-templates/termlink-
//! sidecar@.service`), embedded verbatim at build time. `tools install` writes
//! them under `<prefix>/share/termlink/sidecar/` with every reference into the
//! source checkout rewritten to the installed location, so a host with only the
//! release installed can start and supervise sidecars. The backstop crontab is
//! generated for the installed paths.
//!
//! `binary_check` is the SI-27 half: the running binary's sha256 compared with
//! the release checksum record `install.sh` keeps.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const SUPERVISE_SRC: &str = include_str!("../../../scripts/sidecar-supervise.sh");
pub const UNIT_SRC: &str = include_str!("../../../systemd-templates/termlink-sidecar@.service");

const CHECKOUT_SUPERVISE: &str = "/opt/termlink/scripts/sidecar-supervise.sh";
/// Where the installed backstop keeps its completion heartbeat (the checkout's
/// default sits under the checkout's `.context/working/`).
pub const BACKSTOP_HEARTBEAT: &str = "/var/lib/termlink-sidecar/.backstop.heartbeat";
pub const BACKSTOP_LOG: &str = "/var/log/termlink-sidecar-backstop.log";

/// `<prefix>/share/termlink/sidecar`.
pub fn share_dir(prefix: &Path) -> PathBuf {
    prefix.join("share").join("termlink").join("sidecar")
}

/// The supervise script as installed.
pub fn supervise_installed() -> String {
    SUPERVISE_SRC
        .replace(
            "$ROOT/.context/working/.sidecar-backstop-canary.heartbeat",
            BACKSTOP_HEARTBEAT,
        )
        .replace("rerun /opt/termlink/runme.sh", "rerun 'termlink sidecar tools install --system'")
}

/// The unit template as installed under `prefix`.
pub fn unit_installed(prefix: &Path) -> String {
    let sup = share_dir(prefix).join("sidecar-supervise.sh");
    UNIT_SRC
        .lines()
        .filter(|l| !l.starts_with("Documentation=file:/opt/termlink"))
        .map(|l| {
            l.replace(CHECKOUT_SUPERVISE, &sup.display().to_string())
                .replace("Installed by /opt/termlink/runme.sh (row P3 action) to", "Installed by `termlink sidecar tools install --system` to")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// The backstop crontab for an installed host (USER-field syntax, /etc/cron.d).
pub fn crontab_installed(prefix: &Path) -> String {
    let sup = share_dir(prefix).join("sidecar-supervise.sh");
    format!(
        "# arc-011 sidecar backstop (row P3; installed by `termlink sidecar tools install --system`, P9).\n\
         # Every 10 minutes: repairs a project's sidecar THROUGH systemd (or starts it where\n\
         # there is none) when SIDECAR_KEEP_RUNNING is on and the project was not stopped.\n\
         # Each action is a line in {log} and a needs-attention note. Empty log = healthy.\n\
         SHELL=/bin/bash\n\
         PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\n\
         \n\
         */10 * * * * root SIDECAR_BACKSTOP_HEARTBEAT_FILE={hb} bash {sup} check --quiet >> {log} 2>> {log}.stderr\n",
        log = BACKSTOP_LOG,
        hb = BACKSTOP_HEARTBEAT,
        sup = sup.display()
    )
}

/// Where the system half goes; each `None` skips that part.
pub struct SystemDirs {
    pub unit_dir: Option<PathBuf>,
    pub cron_dir: Option<PathBuf>,
    pub systemctl: Option<PathBuf>,
}

impl SystemDirs {
    /// The real locations, each only when the host has it; env seams for tests.
    pub fn from_host() -> SystemDirs {
        let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
        let exists = |p: &str| Path::new(p).is_dir().then(|| PathBuf::from(p));
        SystemDirs {
            unit_dir: env("TERMLINK_SIDECAR_TOOLS_UNIT_DIR")
                .or_else(|| Path::new("/run/systemd/system").is_dir().then(|| PathBuf::from("/etc/systemd/system"))),
            cron_dir: env("TERMLINK_SIDECAR_TOOLS_CRON_DIR").or_else(|| exists("/etc/cron.d")),
            systemctl: match std::env::var("TERMLINK_SIDECAR_TOOLS_SYSTEMCTL") {
                Ok(s) if s == "none" => None,
                Ok(s) => Some(PathBuf::from(s)),
                Err(_) => Path::new("/run/systemd/system").is_dir().then(|| PathBuf::from("systemctl")),
            },
        }
    }
}

fn write_if_changed(path: &Path, body: &str, mode: u32, out: &mut Vec<String>) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    if std::fs::read_to_string(path).ok().as_deref() == Some(body) {
        out.push(format!("unchanged  {}", path.display()));
        return Ok(false);
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let tmp = path.with_extension("tmp-install");
    std::fs::write(&tmp, body).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(mode)).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))?;
    if std::fs::read_to_string(path).ok().as_deref() != Some(body) {
        return Err(format!("{} does not read back", path.display()));
    }
    out.push(format!("written    {}", path.display()));
    Ok(true)
}

/// Install the tools under `prefix`, and the system half when `system` is given.
/// Returns one line per file (`written` / `unchanged`) and per action.
pub fn install(prefix: &Path, system: Option<&SystemDirs>) -> Result<Vec<String>, String> {
    let mut out = vec![];
    let share = share_dir(prefix);
    write_if_changed(&share.join("sidecar-supervise.sh"), &supervise_installed(), 0o755, &mut out)?;
    write_if_changed(&share.join("termlink-sidecar@.service"), &unit_installed(prefix), 0o644, &mut out)?;
    write_if_changed(&share.join("sidecar-backstop.crontab"), &crontab_installed(prefix), 0o644, &mut out)?;
    let Some(sys) = system else { return Ok(out) };
    let mut unit_changed = false;
    match &sys.unit_dir {
        Some(d) => unit_changed = write_if_changed(&d.join("termlink-sidecar@.service"), &unit_installed(prefix), 0o644, &mut out)?,
        None => out.push("skipped    systemd unit (no systemd on this host: the cron backstop supervises)".into()),
    }
    match &sys.cron_dir {
        Some(d) => {
            write_if_changed(&d.join("termlink-sidecar-backstop"), &crontab_installed(prefix), 0o644, &mut out)?;
        }
        None => out.push("skipped    backstop crontab (no /etc/cron.d)".into()),
    }
    if unit_changed && let Some(sc) = &sys.systemctl {
        let ok = std::process::Command::new(sc).arg("daemon-reload").status().map(|s| s.success()).unwrap_or(false);
        if !ok {
            return Err(format!("{} daemon-reload failed after installing the unit", sc.display()));
        }
        out.push("reloaded   systemd".into());
    }
    Ok(out)
}

/// SI-27: the running binary's sha256 against the release checksum record.
/// The record is `<exe dir>/../share/termlink/checksums.txt` (written by
/// `install.sh`), or `TERMLINK_CHECKSUMS` when set.
pub fn binary_check() -> serde_json::Value {
    let exe = std::env::current_exe().ok();
    // Hashed once per process: the running image cannot change under us (an
    // upgrade replaces the file, not this process), and hashing tens of MB on
    // every status call made status slow enough to skew its own deadlines.
    static SHA: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    let sha = SHA
        .get_or_init(|| {
            exe.as_ref()
                .and_then(|p| std::fs::read(p).ok())
                .map(|b| Sha256::digest(&b).iter().map(|x| format!("{x:02x}")).collect::<String>())
        })
        .clone();
    let record = std::env::var_os("TERMLINK_CHECKSUMS").map(PathBuf::from).or_else(|| {
        exe.as_ref()
            .and_then(|p| p.parent())
            .and_then(|d| d.parent())
            .map(|r| r.join("share").join("termlink").join("checksums.txt"))
    });
    let verified = match (&sha, record.as_ref().and_then(|r| std::fs::read_to_string(r).ok())) {
        (None, _) => "unreadable",
        (Some(_), None) => "no-record",
        (Some(s), Some(text)) => {
            if text.lines().any(|l| l.split_whitespace().next() == Some(s.as_str())) {
                "yes"
            } else {
                "no"
            }
        }
    };
    serde_json::json!({
        "path": exe.map(|p| p.display().to_string()),
        "sha256": sha,
        "record": record.map(|p| p.display().to_string()),
        "verified": verified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code_lines(s: &str) -> Vec<&str> {
        s.lines().filter(|l| !l.trim_start().starts_with('#')).collect()
    }

    #[test]
    fn installed_files_point_at_the_install_never_at_the_checkout() {
        let p = Path::new("/usr/local");
        for (name, body) in [
            ("supervise", supervise_installed()),
            ("unit", unit_installed(p)),
            ("crontab", crontab_installed(p)),
        ] {
            for l in code_lines(&body) {
                assert!(!l.contains("/opt/termlink"), "{name} still points into the checkout: {l}");
            }
        }
        assert!(unit_installed(p).contains("Environment=SIDECAR_SUPERVISE=/usr/local/share/termlink/sidecar/sidecar-supervise.sh"));
        assert!(supervise_installed().contains(BACKSTOP_HEARTBEAT));
        assert!(crontab_installed(p).contains("bash /usr/local/share/termlink/sidecar/sidecar-supervise.sh check --quiet"));
    }

    #[test]
    fn the_embedded_files_are_the_tested_ones_apart_from_the_rewrites() {
        // Only the rewritten lines differ; everything else is the repo's file.
        let changed = SUPERVISE_SRC.lines().zip(supervise_installed().lines()).filter(|(a, b)| a != b).count();
        assert_eq!(changed, 2, "exactly the heartbeat default and the runme hint are rewritten");
        assert_eq!(SUPERVISE_SRC.lines().count(), supervise_installed().lines().count());
        let unit = unit_installed(Path::new("/usr/local"));
        assert_eq!(UNIT_SRC.lines().count(), unit.lines().count() + 1, "only the Documentation line is dropped");
    }

    #[test]
    fn install_is_idempotent_and_writes_the_system_half_on_request() {
        let d = std::env::temp_dir().join(format!("tl-tools-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let sys = SystemDirs { unit_dir: Some(d.join("unit")), cron_dir: Some(d.join("cron")), systemctl: None };
        let first = install(&d.join("prefix"), Some(&sys)).unwrap();
        assert_eq!(first.iter().filter(|l| l.starts_with("written")).count(), 5, "{first:?}");
        let second = install(&d.join("prefix"), Some(&sys)).unwrap();
        assert!(second.iter().all(|l| l.starts_with("unchanged")), "{second:?}");
        use std::os::unix::fs::PermissionsExt;
        let m = std::fs::metadata(share_dir(&d.join("prefix")).join("sidecar-supervise.sh")).unwrap().permissions().mode();
        assert_eq!(m & 0o777, 0o755);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn binary_check_reads_the_record() {
        let d = std::env::temp_dir().join(format!("tl-sum-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let exe = std::env::current_exe().unwrap();
        let sha: String = Sha256::digest(std::fs::read(&exe).unwrap()).iter().map(|x| format!("{x:02x}")).collect();
        let rec = d.join("checksums.txt");
        // SAFETY: test-local variable, read only by binary_check in this thread.
        unsafe { std::env::set_var("TERMLINK_CHECKSUMS", &rec) };
        assert_eq!(binary_check()["verified"], "no-record");
        std::fs::write(&rec, format!("{}  termlink-other\n", "0".repeat(64))).unwrap();
        assert_eq!(binary_check()["verified"], "no");
        std::fs::write(&rec, format!("{sha}  termlink-linux-x86_64\n")).unwrap();
        assert_eq!(binary_check()["verified"], "yes");
        unsafe { std::env::remove_var("TERMLINK_CHECKSUMS") };
        let _ = std::fs::remove_dir_all(&d);
    }
}
