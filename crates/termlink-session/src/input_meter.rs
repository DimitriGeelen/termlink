//! Per-source PTY input counters (arc-011 B-27, AL-2; ARC4 6.14.1 S1, 6.14.2).
//!
//! Every write into the PTY's input is tagged with where it came from and
//! counted under exactly one source:
//!
//! | source | writes |
//! |---|---|
//! | `client` | an attached client's input stream (keystrokes, paste); also any write whose source cannot be classified (FCW-10: someone may be typing) |
//! | `client_auto` | a `client` write that is exactly one terminal reply (cursor-position report, focus in/out, device-attribute reply): the terminal answering, not a person |
//! | `client_pointer` | a `client` write made only of mouse reports (SGR `ESC[<b;x;y` `M`/`m`, legacy `ESC[M` + 3 bytes): a person is present but not typing (T-3854; Claude Code turns on any-motion reporting, so moving the mouse writes these) |
//! | `notify` | `command.notify`, the sidecar doorbell |
//! | `inject` | `command.inject` (and every CLI path built on it: `pty inject`, `interact`, batch input) |
//!
//! The sidecar reads them through `session.input_activity` (notify scope,
//! FCW-9). Times are on [`mono_ms`] (`CLOCK_MONOTONIC`, shared by every process
//! of one boot) together with the [`boot_id`], so a reader on another boot can
//! tell the stamps are not comparable (FCW-11). Counters only grow and live in
//! memory: a session that ends takes them with it.

use serde_json::{Value, json};
use std::sync::Mutex;

/// Where a PTY input write came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    Client,
    ClientAuto,
    Notify,
    Inject,
    ClientPointer,
}

impl InputSource {
    pub const ALL: [InputSource; 5] =
        [InputSource::Client, InputSource::ClientAuto, InputSource::Notify, InputSource::Inject, InputSource::ClientPointer];

    pub fn name(self) -> &'static str {
        match self {
            InputSource::Client => "client",
            InputSource::ClientAuto => "client_auto",
            InputSource::Notify => "notify",
            InputSource::Inject => "inject",
            InputSource::ClientPointer => "client_pointer",
        }
    }

    fn idx(self) -> usize {
        self as usize
    }
}

/// One source's counter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counter {
    pub last_mono: Option<u64>,
    pub bytes: u64,
    pub writes: u64,
}

/// The counters of one PTY, one per [`InputSource`].
#[derive(Debug, Default)]
pub struct InputMeter {
    ctr: Mutex<[Counter; 5]>,
}

impl InputMeter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify a write: a `client` write that is exactly one terminal reply is
    /// `client_auto` (AL-2 step 2); one made only of mouse reports is
    /// `client_pointer` (T-3854). Other sources are taken as given.
    pub fn classify(src: InputSource, data: &[u8]) -> InputSource {
        if src != InputSource::Client {
            src
        } else if is_terminal_reply(data) {
            InputSource::ClientAuto
        } else if is_pointer_reports(data) {
            InputSource::ClientPointer
        } else {
            src
        }
    }

    /// Count one write of `len` bytes from `src` at `at_mono` (AL-2 step 3).
    pub fn record_at(&self, src: InputSource, len: usize, at_mono: u64) {
        let mut c = self.ctr.lock().unwrap_or_else(|p| p.into_inner());
        let e = &mut c[src.idx()];
        e.bytes = e.bytes.saturating_add(len as u64);
        e.writes = e.writes.saturating_add(1);
        // Only grows: a clock that read lower (it cannot, on one boot) never moves it back.
        e.last_mono = Some(e.last_mono.map_or(at_mono, |l| l.max(at_mono)));
    }

    /// [`record_at`](Self::record_at) now.
    pub fn record(&self, src: InputSource, len: usize) {
        self.record_at(src, len, mono_ms());
    }

    pub fn get(&self, src: InputSource) -> Counter {
        self.ctr.lock().unwrap_or_else(|p| p.into_inner())[src.idx()]
    }

    /// `{client, client_auto, notify, inject, client_pointer}: {last_mono|null, bytes, writes}`.
    pub fn sources_json(&self) -> Value {
        let c = *self.ctr.lock().unwrap_or_else(|p| p.into_inner());
        let mut m = serde_json::Map::new();
        for s in InputSource::ALL {
            let e = c[s.idx()];
            m.insert(s.name().into(), json!({"last_mono": e.last_mono, "bytes": e.bytes, "writes": e.writes}));
        }
        Value::Object(m)
    }
}

/// Exactly one terminal reply: `ESC [ <digits> ; <digits> R` (cursor position),
/// `ESC [ I` / `ESC [ O` (focus), `ESC [ ? <digits;...> c` (device attributes).
pub fn is_terminal_reply(b: &[u8]) -> bool {
    let Some(rest) = b.strip_prefix(b"\x1b[") else { return false };
    if rest == b"I" || rest == b"O" {
        return true;
    }
    let digits = |s: &[u8]| !s.is_empty() && s.iter().all(u8::is_ascii_digit);
    if let Some(body) = rest.strip_suffix(b"R") {
        let mut it = body.split(|c| *c == b';');
        return matches!((it.next(), it.next(), it.next()), (Some(r), Some(c), None) if digits(r) && digits(c));
    }
    if let Some(body) = rest.strip_prefix(b"?").and_then(|r| r.strip_suffix(b"c")) {
        return body.split(|c| *c == b';').all(digits);
    }
    false
}

/// One or more mouse reports and nothing else (T-3854): SGR (mode 1006)
/// `ESC [ < <digits> ; <digits> ; <digits> M|m`, or legacy (mode 1000)
/// `ESC [ M` followed by three bytes of 32 or more. Anything else in the write
/// (a keystroke, a paste) makes it `client`.
pub fn is_pointer_reports(b: &[u8]) -> bool {
    let digits = |s: &[u8]| !s.is_empty() && s.iter().all(u8::is_ascii_digit);
    let mut rest = b;
    if rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix(b"\x1b[<") {
            let Some(end) = r.iter().position(|c| *c == b'M' || *c == b'm') else { return false };
            let mut it = r[..end].split(|c| *c == b';');
            if !matches!((it.next(), it.next(), it.next(), it.next()), (Some(a), Some(x), Some(y), None) if digits(a) && digits(x) && digits(y)) {
                return false;
            }
            rest = &r[end + 1..];
        } else if let Some(r) = rest.strip_prefix(b"\x1b[M") {
            if r.len() < 3 || r[..3].iter().any(|c| *c < 32) {
                return false;
            }
            rest = &r[3..];
        } else {
            return false;
        }
    }
    true
}

/// Milliseconds on `CLOCK_MONOTONIC` (since boot on Linux, shared by every
/// process of the boot).
pub fn mono_ms() -> u64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes one timespec through a valid pointer.
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    (ts.tv_sec as u64).saturating_mul(1000).saturating_add(ts.tv_nsec as u64 / 1_000_000)
}

/// This boot's id (`/proc/sys/kernel/random/boot_id`); `None` where there is
/// none, and then a reader treats the stamps as not comparable.
pub fn boot_id() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        boot_id_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }
}

/// macOS has no `/proc`; its per-boot UUID is the sysctl `kern.bootsessionuuid`
/// (T-3763: without it S1, the origin check and P-2 ordering were all `unknown` on a Mac).
#[cfg(target_os = "macos")]
fn boot_id_macos() -> Option<String> {
    let mut buf = [0u8; 64];
    let mut len: libc::size_t = buf.len();
    // SAFETY: the name is NUL-terminated, `buf`/`len` describe a valid writable buffer,
    // and no new value is set (null, 0).
    let rc = unsafe {
        libc::sysctlbyname(
            c"kern.bootsessionuuid".as_ptr(),
            buf.as_mut_ptr().cast::<libc::c_void>(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc != 0 {
        return None;
    }
    let raw = &buf[..len.min(buf.len())];
    let s = String::from_utf8_lossy(raw).trim_end_matches('\0').trim().to_ascii_lowercase();
    (!s.is_empty()).then_some(s)
}

/// The PTY's foreground process group (`tcgetpgrp` on the master) with its
/// leader's executable and name. `None` when it cannot be read.
pub fn foreground(master_fd: std::os::fd::RawFd) -> Value {
    // SAFETY: tcgetpgrp reads the fd's foreground group; an invalid fd returns -1.
    let pgid = unsafe { libc::tcgetpgrp(master_fd) };
    if pgid <= 0 {
        return Value::Null;
    }
    #[cfg(target_os = "macos")]
    let (exe, comm, script) = {
        // T-3770: no `/proc` on macOS; libproc and sysctl KERN_PROCARGS2 instead.
        use crate::procinfo::macos;
        let pid = pgid as u32;
        (macos::exe(pid), macos::comm(pid), macos::script(pid))
    };
    #[cfg(not(target_os = "macos"))]
    let (exe, comm, script) = {
        let dir = std::path::Path::new("/proc").join(pgid.to_string());
        let exe = std::fs::read_link(dir.join("exe")).ok().map(|p| p.to_string_lossy().into_owned());
        let comm = std::fs::read_to_string(dir.join("comm")).ok().map(|s| s.trim().to_string());
        (exe, comm, script_path(&dir))
    };
    json!({"pgid": pgid, "leader_pid": pgid, "exe_realpath": exe, "script_path": script, "comm": comm})
}

/// B-16 (04 17.4.a): the script an interpreter runs — the first non-option
/// argument after argv[0], resolved against the process's cwd. `None` for a
/// native binary run without one, or when it cannot be read.
pub fn script_path(proc_dir: &std::path::Path) -> Option<String> {
    let raw = std::fs::read(proc_dir.join("cmdline")).ok()?;
    let args: Vec<String> = raw.split(|b| *b == 0).map(|a| String::from_utf8_lossy(a).into_owned()).collect();
    let cwd = std::fs::read_link(proc_dir.join("cwd")).ok();
    crate::procinfo::script_from_args(&args, cwd.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_writes_move_client_only() {
        let m = InputMeter::new();
        m.record_at(InputSource::Client, 3, 100);
        assert_eq!(m.get(InputSource::Client), Counter { last_mono: Some(100), bytes: 3, writes: 1 });
        for s in [InputSource::Notify, InputSource::Inject, InputSource::ClientAuto] {
            assert_eq!(m.get(s), Counter::default(), "{} must not move", s.name());
        }
    }

    #[test]
    fn notify_and_inject_never_count_as_client() {
        let m = InputMeter::new();
        m.record_at(InputMeter::classify(InputSource::Notify, b"[sidecar] 1 peer message"), 24, 50);
        m.record_at(InputMeter::classify(InputSource::Inject, b"ls\r"), 3, 60);
        assert_eq!(m.get(InputSource::Client).last_mono, None);
        assert_eq!(m.get(InputSource::Notify).last_mono, Some(50));
        assert_eq!(m.get(InputSource::Inject).last_mono, Some(60));
    }

    #[test]
    fn terminal_replies_are_client_auto_and_anything_else_is_client() {
        for r in [&b"\x1b[12;40R"[..], b"\x1b[I", b"\x1b[O", b"\x1b[?1;2c", b"\x1b[?62;22c"] {
            assert_eq!(InputMeter::classify(InputSource::Client, r), InputSource::ClientAuto, "{r:?}");
        }
        // Not exactly one reply: a person (or unknown) -> client, holds the doorbell.
        for r in [&b"\x1b[12;40Rx"[..], b"\x1b[12R", b"\x1b[A", b"hello", b"\x1b[I\x1b[O", b"\x1b[?c;c", b""] {
            assert_eq!(InputMeter::classify(InputSource::Client, r), InputSource::Client, "{r:?}");
        }
        // Only client writes are reclassified.
        assert_eq!(InputMeter::classify(InputSource::Inject, b"\x1b[I"), InputSource::Inject);
    }

    #[test]
    fn mouse_reports_are_client_pointer_and_typing_stays_client() {
        // T-3854: any-motion (1003) + SGR (1006) as Claude Code sets them; one or several reports per write.
        for r in [
            &b"\x1b[<35;40;12M"[..],
            b"\x1b[<64;10;5M",
            b"\x1b[<0;1;1m",
            b"\x1b[<35;40;12M\x1b[<35;41;12M\x1b[<35;42;13M",
            b"\x1b[M #!",
            b"\x1b[M #!\x1b[<35;1;1M",
        ] {
            assert_eq!(InputMeter::classify(InputSource::Client, r), InputSource::ClientPointer, "{r:?}");
        }
        // A keystroke, a paste, or a mouse report mixed with either stays client (FCW-10).
        for r in [
            &b"x"[..],
            b"\r",
            b"\x1b[<35;40;12Mx",
            b"x\x1b[<35;40;12M",
            b"\x1b[<35;40M",
            b"\x1b[<35;40;12;1M",
            b"\x1b[<a;40;12M",
            b"\x1b[<35;40;12",
            b"\x1b[M\x01!!",
            b"\x1b[M #",
            b"\x1b[200~text\x1b[201~",
        ] {
            assert_eq!(InputMeter::classify(InputSource::Client, r), InputSource::Client, "{r:?}");
        }
        // Terminal replies keep their class; other sources are taken as given.
        assert_eq!(InputMeter::classify(InputSource::Client, b"\x1b[12;40R"), InputSource::ClientAuto);
        assert_eq!(InputMeter::classify(InputSource::Notify, b"\x1b[<35;1;1M"), InputSource::Notify);
    }

    #[test]
    fn counters_only_grow() {
        let m = InputMeter::new();
        m.record_at(InputSource::Client, 1, 200);
        m.record_at(InputSource::Client, 1, 150);
        assert_eq!(m.get(InputSource::Client), Counter { last_mono: Some(200), bytes: 2, writes: 2 });
    }

    #[test]
    fn json_names_all_five_sources() {
        let m = InputMeter::new();
        m.record_at(InputSource::Notify, 5, 7);
        m.record_at(InputSource::ClientPointer, 12, 9);
        let j = m.sources_json();
        assert_eq!(j["notify"], json!({"last_mono": 7, "bytes": 5, "writes": 1}));
        assert_eq!(j["client_pointer"], json!({"last_mono": 9, "bytes": 12, "writes": 1}));
        assert_eq!(j["client"]["last_mono"], Value::Null);
        assert_eq!(j.as_object().unwrap().len(), 5);
    }

    #[test]
    fn mono_is_monotonic() {
        let a = mono_ms();
        let b = mono_ms();
        assert!(b >= a && a > 0);
    }
}
