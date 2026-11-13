//! T-3770: process facts the session binding needs (parent pid, executable, argv, cwd)
//! read without `/proc`. Linux keeps reading `/proc` where its callers already do; this
//! module carries the pure rules both platforms share and the macOS readers (libproc
//! `proc_pidinfo` / `proc_pidpath`, sysctl `KERN_PROCARGS2`), each behind
//! `cfg(target_os = "macos")`. Everything unreadable is `None`: a caller that cannot
//! read a fact leaves the session unbound and says why, it never guesses.

use std::path::Path;

/// Longest parent chain walked (a deeper one is cut, never looped on).
pub const MAX_ANCESTORS: usize = 64;

/// Ancestors of `pid`, nearest first, through `ppid` (one parent lookup per step).
/// Stops at init (`<= 1`), at an unreadable step, at a pid already seen (a cycle in
/// a stale or fake table) and after [`MAX_ANCESTORS`].
pub fn ancestors_with(pid: u32, mut ppid: impl FnMut(u32) -> Option<u32>) -> Vec<u32> {
    let mut out: Vec<u32> = vec![];
    let mut cur = pid;
    while out.len() < MAX_ANCESTORS {
        let Some(pp) = ppid(cur) else { break };
        if pp <= 1 || pp == pid || out.contains(&pp) {
            break;
        }
        out.push(pp);
        cur = pp;
    }
    out
}

/// B-16 (04 17.4.a): the script an interpreter runs — the first non-empty,
/// non-option argument after argv[0], resolved against `cwd` when relative and
/// canonicalized. `None` for a native binary run without one, or when it cannot
/// be resolved. One rule for the `/proc` and the macOS readers.
pub fn script_from_args<S: AsRef<str>>(args: &[S], cwd: Option<&Path>) -> Option<String> {
    let arg = args.iter().skip(1).map(AsRef::as_ref).find(|a| !a.is_empty() && !a.starts_with('-'))?;
    let p = Path::new(arg);
    let full = if p.is_absolute() { p.to_path_buf() } else { cwd?.join(p) };
    std::fs::canonicalize(full).ok().map(|p| p.to_string_lossy().into_owned())
}

/// The argv in a macOS `KERN_PROCARGS2` buffer: a native-endian `i32` argc, the
/// exec path NUL-terminated, NUL padding, then argc NUL-terminated strings (the
/// environment follows). Returns the argv strings present (fewer than argc when
/// the buffer is cut short); `None` when the header or argv[0] is missing.
pub fn parse_procargs2(buf: &[u8]) -> Option<Vec<String>> {
    let argc = i32::from_ne_bytes(buf.get(..4)?.try_into().ok()?);
    if argc <= 0 {
        return None;
    }
    let rest = &buf[4..];
    // Skip the exec path, then the NUL padding after it.
    let exec_end = rest.iter().position(|b| *b == 0)?;
    let mut i = exec_end;
    while rest.get(i) == Some(&0) {
        i += 1;
    }
    let mut out = vec![];
    while out.len() < argc as usize && i < rest.len() {
        let end = rest[i..].iter().position(|b| *b == 0).map_or(rest.len(), |e| i + e);
        out.push(String::from_utf8_lossy(&rest[i..end]).into_owned());
        i = end + 1;
    }
    (!out.is_empty()).then_some(out)
}

/// The macOS libproc / sysctl readers.
#[cfg(target_os = "macos")]
pub mod macos {
    use std::path::PathBuf;

    /// `proc_pidinfo(PROC_PIDTBSDINFO)`: `None` when the pid is gone or not readable.
    fn bsdinfo(pid: u32) -> Option<libc::proc_bsdinfo> {
        let pid = libc::c_int::try_from(pid).ok()?;
        // SAFETY: proc_bsdinfo is plain old data; all-zero is a valid value.
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        // SAFETY: the buffer is a valid, writable proc_bsdinfo of `size` bytes.
        let n = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, (&raw mut info).cast::<libc::c_void>(), size) };
        (n == size).then_some(info)
    }

    /// Parent pid of `pid`.
    pub fn ppid(pid: u32) -> Option<u32> {
        bsdinfo(pid).map(|i| i.pbi_ppid)
    }

    /// The process's short command name (`pbi_comm`).
    pub fn comm(pid: u32) -> Option<String> {
        let i = bsdinfo(pid)?;
        let bytes: Vec<u8> = i.pbi_comm.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
        (!bytes.is_empty()).then(|| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// The executable's path (`proc_pidpath`), canonicalized like Linux `/proc/<pid>/exe`.
    pub fn exe(pid: u32) -> Option<String> {
        let pid = libc::c_int::try_from(pid).ok()?;
        let mut buf = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        // SAFETY: `buf` is a writable buffer of the stated size.
        let n = unsafe { libc::proc_pidpath(pid, buf.as_mut_ptr().cast::<libc::c_void>(), buf.len() as u32) };
        if n <= 0 {
            return None;
        }
        let raw = String::from_utf8_lossy(&buf[..(n as usize).min(buf.len())]).into_owned();
        Some(std::fs::canonicalize(&raw).map_or(raw, |p| p.to_string_lossy().into_owned()))
    }

    /// The process's current directory (`PROC_PIDVNODEPATHINFO`).
    pub fn cwd(pid: u32) -> Option<PathBuf> {
        let pid = libc::c_int::try_from(pid).ok()?;
        // SAFETY: proc_vnodepathinfo is plain old data; all-zero is a valid value.
        let mut info: libc::proc_vnodepathinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<libc::proc_vnodepathinfo>() as libc::c_int;
        // SAFETY: the buffer is a valid, writable proc_vnodepathinfo of `size` bytes.
        let n = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDVNODEPATHINFO, 0, (&raw mut info).cast::<libc::c_void>(), size) };
        if n != size {
            return None;
        }
        let path: Vec<u8> = info.pvi_cdir.vip_path.iter().flatten().take_while(|c| **c != 0).map(|c| *c as u8).collect();
        (!path.is_empty()).then(|| PathBuf::from(String::from_utf8_lossy(&path).into_owned()))
    }

    /// The process's argv (`sysctl KERN_PROCARGS2`); `None` when not permitted or gone.
    pub fn argv(pid: u32) -> Option<Vec<String>> {
        let pid = libc::c_int::try_from(pid).ok()?;
        let mut argmax: libc::c_int = 0;
        let mut len: libc::size_t = std::mem::size_of::<libc::c_int>();
        let mut mib = [libc::CTL_KERN, libc::KERN_ARGMAX];
        // SAFETY: `mib` names KERN_ARGMAX, `argmax`/`len` describe a writable c_int; no new value.
        let rc = unsafe { libc::sysctl(mib.as_mut_ptr(), 2, (&raw mut argmax).cast::<libc::c_void>(), &mut len, std::ptr::null_mut(), 0) };
        if rc != 0 || argmax <= 0 {
            return None;
        }
        let mut buf = vec![0u8; argmax as usize];
        let mut len: libc::size_t = buf.len();
        let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid];
        // SAFETY: `buf`/`len` describe a writable buffer; no new value is set.
        let rc = unsafe { libc::sysctl(mib.as_mut_ptr(), 3, buf.as_mut_ptr().cast::<libc::c_void>(), &mut len, std::ptr::null_mut(), 0) };
        if rc != 0 {
            return None;
        }
        super::parse_procargs2(&buf[..len.min(buf.len())])
    }

    /// The script an interpreter `pid` runs ([`super::script_from_args`]).
    pub fn script(pid: u32) -> Option<String> {
        super::script_from_args(&argv(pid)?, cwd(pid).as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn table(pairs: &[(u32, u32)]) -> HashMap<u32, u32> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn ancestors_stop_at_init_and_at_an_unreadable_step() {
        let t = table(&[(50, 40), (40, 30), (30, 1)]);
        assert_eq!(ancestors_with(50, |p| t.get(&p).copied()), vec![40, 30]);
        let t = table(&[(50, 40)]); // 40 unreadable
        assert_eq!(ancestors_with(50, |p| t.get(&p).copied()), vec![40]);
        assert!(ancestors_with(50, |_| None).is_empty());
        assert!(ancestors_with(50, |_| Some(0)).is_empty());
    }

    #[test]
    fn ancestors_are_cycle_safe_and_capped() {
        let t = table(&[(50, 40), (40, 30), (30, 40)]);
        assert_eq!(ancestors_with(50, |p| t.get(&p).copied()), vec![40, 30]);
        let t = table(&[(50, 40), (40, 50)]);
        assert_eq!(ancestors_with(50, |p| t.get(&p).copied()), vec![40]);
        // A chain deeper than the cap: cut at MAX_ANCESTORS, never looping.
        let deep = ancestors_with(10_000, |p| Some(p - 1));
        assert_eq!(deep.len(), MAX_ANCESTORS);
        assert_eq!(deep[0], 9_999);
    }

    fn procargs(argc: i32, exec: &str, pad: usize, args: &[&str], env: &[&str]) -> Vec<u8> {
        let mut b = argc.to_ne_bytes().to_vec();
        b.extend_from_slice(exec.as_bytes());
        b.extend(std::iter::repeat_n(0u8, 1 + pad));
        for a in args.iter().chain(env) {
            b.extend_from_slice(a.as_bytes());
            b.push(0);
        }
        b
    }

    #[test]
    fn procargs2_parses_argv_and_stops_before_env() {
        let b = procargs(3, "/usr/local/bin/node", 5, &["node", "/opt/cc/cli.js", "--x"], &["HOME=/r"]);
        assert_eq!(parse_procargs2(&b).unwrap(), vec!["node", "/opt/cc/cli.js", "--x"]);
        let b = procargs(1, "/bin/claude", 0, &["claude"], &[]);
        assert_eq!(parse_procargs2(&b).unwrap(), vec!["claude"]);
    }

    #[test]
    fn procargs2_bad_buffers_never_panic() {
        assert_eq!(parse_procargs2(&[]), None);
        assert_eq!(parse_procargs2(&[1, 0]), None);
        assert_eq!(parse_procargs2(&procargs(0, "/bin/x", 0, &["x"], &[])), None);
        assert_eq!(parse_procargs2(&procargs(-3, "/bin/x", 0, &["x"], &[])), None);
        // No NUL after the exec path: header present, nothing parseable.
        let mut b = 2i32.to_ne_bytes().to_vec();
        b.extend_from_slice(b"/bin/x");
        assert_eq!(parse_procargs2(&b), None);
        // argc larger than the strings present: the strings present.
        let b = procargs(9, "/bin/x", 2, &["x", "y"], &[]);
        assert_eq!(parse_procargs2(&b).unwrap(), vec!["x", "y"]);
        // Cut mid-string: the partial string is kept, nothing past the end is read.
        let mut b = procargs(2, "/bin/x", 0, &["x", "abc"], &[]);
        b.truncate(b.len() - 2);
        assert_eq!(parse_procargs2(&b).unwrap(), vec!["x", "ab"]);
    }

    #[test]
    fn script_rule_skips_argv0_and_options_and_resolves_cwd() {
        let d = std::env::temp_dir().join(format!("t3770-script-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("cli.js"), "").unwrap();
        let canon = std::fs::canonicalize(d.join("cli.js")).unwrap().to_string_lossy().into_owned();
        assert_eq!(script_from_args(&["node", "--inspect", "", "cli.js"], Some(&d)).as_deref(), Some(canon.as_str()));
        assert_eq!(script_from_args(&["node".to_string(), canon.clone()], None).as_deref(), Some(canon.as_str()));
        // Relative without a cwd, nothing after argv[0], or a missing file: None.
        assert_eq!(script_from_args(&["node", "cli.js"], None), None);
        assert_eq!(script_from_args(&["claude"], Some(&d)), None);
        assert_eq!(script_from_args(&["node", "nope.js"], Some(&d)), None);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// macOS (CI job): the native readers see this test process and a child it spawns.
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_readers_see_a_real_child() {
        let me = std::process::id();
        let mut child = std::process::Command::new("/bin/sleep").arg("30").spawn().unwrap();
        let cp = child.id();
        let pp = macos::ppid(cp);
        let exe = macos::exe(cp);
        let argv = macos::argv(cp);
        let anc = ancestors_with(cp, macos::ppid);
        let _ = child.kill();
        let _ = child.wait();
        assert_eq!(pp, Some(me), "parent of the child is this test");
        assert_eq!(anc.first(), Some(&me));
        assert!(exe.as_deref().is_some_and(|e| e.ends_with("/sleep")), "{exe:?}");
        assert_eq!(argv, Some(vec!["/bin/sleep".to_string(), "30".to_string()]));
        let my_exe = std::env::current_exe().unwrap().canonicalize().unwrap().to_string_lossy().into_owned();
        assert_eq!(macos::exe(me).as_deref(), Some(my_exe.as_str()));
        assert!(macos::cwd(me).is_some());
        assert!(macos::comm(me).is_some());
        // Negative control: a pid that does not exist reads as nothing.
        assert_eq!(macos::ppid(u32::MAX >> 1), None);
        assert_eq!(macos::exe(u32::MAX >> 1), None);
    }
}
