//! Per-source PTY input counters (arc-011 B-27, AL-2; ARC4 6.14.1 S1, 6.14.2).
//!
//! Every write into the PTY's input is tagged with where it came from and
//! counted under exactly one source:
//!
//! | source | writes |
//! |---|---|
//! | `client` | an attached client's input stream (keystrokes, paste); also any write whose source cannot be classified (FCW-10: someone may be typing) |
//! | `client_auto` | a `client` write that is exactly one terminal reply (cursor-position report, focus in/out, device-attribute reply): the terminal answering, not a person |
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
}

impl InputSource {
    pub const ALL: [InputSource; 4] = [InputSource::Client, InputSource::ClientAuto, InputSource::Notify, InputSource::Inject];

    pub fn name(self) -> &'static str {
        match self {
            InputSource::Client => "client",
            InputSource::ClientAuto => "client_auto",
            InputSource::Notify => "notify",
            InputSource::Inject => "inject",
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

/// The four counters of one PTY.
#[derive(Debug, Default)]
pub struct InputMeter {
    ctr: Mutex<[Counter; 4]>,
}

impl InputMeter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify a write: a `client` write that is exactly one terminal reply is
    /// `client_auto` (AL-2 step 2). Other sources are taken as given.
    pub fn classify(src: InputSource, data: &[u8]) -> InputSource {
        if src == InputSource::Client && is_terminal_reply(data) { InputSource::ClientAuto } else { src }
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

    /// `{client, client_auto, notify, inject}: {last_mono|null, bytes, writes}`.
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
    let dir = std::path::Path::new("/proc").join(pgid.to_string());
    let exe = std::fs::read_link(dir.join("exe")).ok().map(|p| p.to_string_lossy().into_owned());
    let comm = std::fs::read_to_string(dir.join("comm")).ok().map(|s| s.trim().to_string());
    let script = script_path(&dir);
    json!({"pgid": pgid, "leader_pid": pgid, "exe_realpath": exe, "script_path": script, "comm": comm})
}

/// B-16 (04 17.4.a): the script an interpreter runs — the first non-option
/// argument after argv[0], resolved against the process's cwd. `None` for a
/// native binary run without one, or when it cannot be read.
pub fn script_path(proc_dir: &std::path::Path) -> Option<String> {
    let raw = std::fs::read(proc_dir.join("cmdline")).ok()?;
    let arg = raw.split(|b| *b == 0).skip(1).map(|a| String::from_utf8_lossy(a).into_owned()).find(|a| !a.is_empty() && !a.starts_with('-'))?;
    let p = std::path::Path::new(&arg);
    let full = if p.is_absolute() { p.to_path_buf() } else { std::fs::read_link(proc_dir.join("cwd")).ok()?.join(p) };
    std::fs::canonicalize(full).ok().map(|p| p.to_string_lossy().into_owned())
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
    fn counters_only_grow() {
        let m = InputMeter::new();
        m.record_at(InputSource::Client, 1, 200);
        m.record_at(InputSource::Client, 1, 150);
        assert_eq!(m.get(InputSource::Client), Counter { last_mono: Some(200), bytes: 2, writes: 2 });
    }

    #[test]
    fn json_names_all_four_sources() {
        let m = InputMeter::new();
        m.record_at(InputSource::Notify, 5, 7);
        let j = m.sources_json();
        assert_eq!(j["notify"], json!({"last_mono": 7, "bytes": 5, "writes": 1}));
        assert_eq!(j["client"]["last_mono"], Value::Null);
        assert_eq!(j.as_object().unwrap().len(), 4);
    }

    #[test]
    fn mono_is_monotonic() {
        let a = mono_ms();
        let b = mono_ms();
        assert!(b >= a && a > 0);
    }
}
