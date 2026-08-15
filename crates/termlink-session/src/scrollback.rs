use std::collections::VecDeque;

/// A UTF-8 character is at most 4 bytes, so at most 3 continuation bytes can
/// follow its lead byte. A scan longer than this is not looking at UTF-8.
const MAX_UTF8_CONTINUATION_BYTES: usize = 3;

/// Is this a UTF-8 continuation byte (`0b10xxxxxx`)?
fn is_utf8_continuation(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

/// Advance a **front-cut** offset forward to the next UTF-8 character boundary.
///
/// T-2733. The scrollback is a byte ring by design — it holds raw terminal
/// output including ANSI sequences — but every cut it makes lands on an
/// arbitrary byte offset, so a multi-byte character straddling one gets halved.
/// `handler.rs` then `from_utf8_lossy`es the result, and the operator reads a
/// U+FFFD where a real character was. Nothing errors; the text is just wrong.
///
/// Direction matters, which is why `commands/pty.rs::char_boundary_floor`
/// could not simply be reused: it moves **backward**, correct when truncating a
/// tail, wrong here. Moving backward from a start offset would *include* the
/// orphaned lead byte rather than drop it — turning a cut character into a
/// visible mojibake byte instead of removing it.
///
/// **Bounded on purpose.** After `MAX_UTF8_CONTINUATION_BYTES` steps this
/// returns the caller's offset untouched. Terminal output is not guaranteed to
/// be UTF-8 — a child can emit arbitrary bytes — and silently eating bytes off
/// the front of binary output to satisfy a UTF-8 assumption would be a second
/// corruption bug wearing the first one's clothes.
fn utf8_boundary_ceil(len: usize, start: usize, byte_at: impl Fn(usize) -> u8) -> usize {
    if start == 0 || start >= len {
        return start;
    }
    let mut i = start;
    let mut steps = 0;
    while i < len && steps < MAX_UTF8_CONTINUATION_BYTES && is_utf8_continuation(byte_at(i)) {
        i += 1;
        steps += 1;
    }
    if i < len && is_utf8_continuation(byte_at(i)) {
        start
    } else {
        i
    }
}

/// A byte-oriented ring buffer for terminal output.
///
/// Stores raw terminal output (including ANSI sequences) and provides
/// methods to query recent output by line count or byte count.
pub struct ScrollbackBuffer {
    buffer: VecDeque<u8>,
    max_bytes: usize,
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer with the given maximum size in bytes.
    pub fn new(max_bytes: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_bytes.min(64 * 1024)),
            max_bytes,
        }
    }

    /// Append bytes to the buffer, dropping oldest bytes if capacity is exceeded.
    pub fn append(&mut self, data: &[u8]) {
        // If incoming data is larger than max, only keep the tail
        if data.len() >= self.max_bytes {
            self.buffer.clear();
            let start = data.len() - self.max_bytes;
            // T-2733: keep slightly less than max rather than start mid-character.
            let start = utf8_boundary_ceil(data.len(), start, |i| data[i]);
            self.buffer.extend(&data[start..]);
            return;
        }

        // Make room if needed
        let overflow = (self.buffer.len() + data.len()).saturating_sub(self.max_bytes);
        if overflow > 0 {
            // T-2733: drop a few more bytes if the ring boundary splits a
            // character — the alternative is a U+FFFD at the head of every
            // read once the buffer has wrapped.
            let overflow = utf8_boundary_ceil(self.buffer.len(), overflow, |i| self.buffer[i]);
            self.buffer.drain(..overflow);
        }

        self.buffer.extend(data);
    }

    /// Return the last N bytes of output.
    ///
    /// May return slightly fewer than `n` bytes: T-2733 advances the cut to a
    /// UTF-8 character boundary rather than splitting a character. This is the
    /// path `cmd_interact` polls (`bytes: 131072`), so the split was reaching
    /// operators as a replacement glyph on live output.
    pub fn last_n_bytes(&self, n: usize) -> Vec<u8> {
        let start = self.buffer.len().saturating_sub(n);
        let start = utf8_boundary_ceil(self.buffer.len(), start, |i| self.buffer[i]);
        self.buffer.iter().skip(start).copied().collect()
    }

    /// Return the last N lines of output.
    ///
    /// A "line" is delimited by `\n`. The returned bytes include the newlines.
    /// If the buffer contains fewer than N lines, returns all content.
    ///
    /// Deliberately NOT boundary-adjusted (T-2733): this cuts immediately after
    /// a `\n`, and `\n` is ASCII, so the offset is already a UTF-8 character
    /// boundary by construction. Adding a `utf8_boundary_ceil` call here would
    /// be dead code implying a hazard that does not exist on this path — which
    /// is why the default read path was never the one corrupting output.
    pub fn last_n_lines(&self, n: usize) -> Vec<u8> {
        if n == 0 || self.buffer.is_empty() {
            return Vec::new();
        }

        // Walk backwards counting newlines.
        // A "line" is the content after a newline (or start of buffer).
        // If buffer ends without newline, the trailing content is the last line.
        let mut newline_count = 0;
        let mut start = 0;
        let len = self.buffer.len();

        // Skip trailing newline for counting purposes
        let search_end = if self.buffer[len - 1] == b'\n' {
            len - 1
        } else {
            len
        };

        for i in (0..search_end).rev() {
            if self.buffer[i] == b'\n' {
                newline_count += 1;
                if newline_count == n {
                    start = i + 1;
                    break;
                }
            }
        }

        self.buffer.iter().skip(start).copied().collect()
    }

    /// Total bytes currently stored.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear all stored output.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Default scrollback size: 1 MiB.
impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new(1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === T-2733 LOAD-BEARING: cuts must land on UTF-8 character boundaries ===
    //
    // The buffer is a byte ring holding raw terminal output, and three of its
    // cuts used arbitrary byte offsets. A multi-byte character straddling one
    // was halved, and `from_utf8_lossy` downstream turned the remnant into
    // U+FFFD — a wrong character in output an operator reads to decide what
    // happened, with nothing anywhere reporting a problem.
    //
    // Each of these fails if `utf8_boundary_ceil` is removed from its site
    // (demonstrated by temp-revert), so they pin the fix rather than describe it.

    /// U+00E9 (é) is 2 bytes, U+20AC (€) is 3, U+1F600 (😀) is 4 — one of each
    /// width, so a cut can land inside any of the three multi-byte forms.
    const MIXED: &str = "aéb€c😀d";

    #[test]
    fn last_n_bytes_never_splits_a_character() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(MIXED.as_bytes());

        // Every possible tail length, including all the ones that land inside a
        // character. Not a sampled offset — the bug is offset-specific, so
        // testing one offset would prove nothing about the others.
        for n in 0..=MIXED.len() {
            let got = buf.last_n_bytes(n);
            let s = String::from_utf8(got.clone())
                .unwrap_or_else(|_| panic!("last_n_bytes({n}) returned invalid UTF-8: {got:?}"));
            assert!(
                !s.contains('\u{FFFD}'),
                "last_n_bytes({n}) produced a replacement char: {s:?}"
            );
            // It must still be a genuine suffix — boundary-snapping may return
            // fewer bytes, never different ones.
            assert!(MIXED.ends_with(&s), "last_n_bytes({n}) is not a suffix: {s:?}");
        }
    }

    #[test]
    fn ring_overflow_never_leaves_a_split_character_at_the_head() {
        // Size the ring so the wrap point lands inside a multi-byte character
        // for at least some capacities; sweep them all rather than guess.
        for cap in 4..=MIXED.len() * 2 {
            let mut buf = ScrollbackBuffer::new(cap);
            buf.append(MIXED.as_bytes());
            buf.append(MIXED.as_bytes());
            let got = buf.last_n_bytes(usize::MAX);
            let s = String::from_utf8(got.clone())
                .unwrap_or_else(|_| panic!("cap {cap}: buffer head is not a boundary: {got:?}"));
            assert!(
                !s.contains('\u{FFFD}'),
                "cap {cap}: replacement char after ring overflow: {s:?}"
            );
        }
    }

    #[test]
    fn oversized_single_write_keeps_a_boundary_start() {
        // The `data.len() >= max_bytes` branch: the write is truncated to its
        // own tail, a separate cut site from the overflow drain above.
        for cap in 4..MIXED.len() {
            let mut buf = ScrollbackBuffer::new(cap);
            buf.append(MIXED.as_bytes());
            let got = buf.last_n_bytes(usize::MAX);
            let s = String::from_utf8(got.clone())
                .unwrap_or_else(|_| panic!("cap {cap}: oversized write split a char: {got:?}"));
            assert!(!s.contains('\u{FFFD}'), "cap {cap}: replacement char: {s:?}");
        }
    }

    #[test]
    fn binary_output_is_not_trimmed_by_the_bounded_scan() {
        // A child can emit arbitrary bytes. Advancing past continuation bytes
        // unboundedly would eat the front of binary output to satisfy a UTF-8
        // assumption that does not hold — trading one corruption for another.
        // 0x80..0xBF are all continuation bytes with no lead byte in sight.
        let binary: Vec<u8> = (0x80u8..=0xBF).collect();
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(&binary);

        let got = buf.last_n_bytes(10);
        assert_eq!(
            got.len(),
            10,
            "bounded scan must give up on non-UTF-8 and return the exact tail"
        );
        assert_eq!(got, &binary[binary.len() - 10..]);
    }

    #[test]
    fn boundary_ceil_advances_at_most_three_bytes() {
        // The bound is the whole reason the binary case above is safe, so pin
        // it directly rather than only through its consequence.
        let all_continuations = [0x80u8, 0x80, 0x80, 0x80, 0x80, 0x80];
        let got = utf8_boundary_ceil(all_continuations.len(), 1, |i| all_continuations[i]);
        assert_eq!(got, 1, "unresolvable scan must return the original offset");

        // Three continuations then a lead byte: resolvable, so it advances.
        let four_byte_tail = [b'x', 0x80, 0x80, 0x80, b'y'];
        let got = utf8_boundary_ceil(four_byte_tail.len(), 1, |i| four_byte_tail[i]);
        assert_eq!(got, 4, "must advance to the next non-continuation byte");
    }

    #[test]
    fn append_and_read_bytes() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"hello ");
        buf.append(b"world");
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.last_n_bytes(5), b"world");
        assert_eq!(buf.last_n_bytes(100), b"hello world");
    }

    #[test]
    fn ring_buffer_drops_oldest() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.append(b"12345");
        buf.append(b"67890");
        assert_eq!(buf.len(), 10);

        // Buffer is full, append more
        buf.append(b"abc");
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.last_n_bytes(10), b"4567890abc");
    }

    #[test]
    fn oversized_append_keeps_tail() {
        let mut buf = ScrollbackBuffer::new(5);
        buf.append(b"0123456789");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.last_n_bytes(5), b"56789");
    }

    #[test]
    fn last_n_lines() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"line1\nline2\nline3\nline4\n");

        let last2 = buf.last_n_lines(2);
        assert_eq!(last2, b"line3\nline4\n");

        let last1 = buf.last_n_lines(1);
        assert_eq!(last1, b"line4\n");

        let all = buf.last_n_lines(100);
        assert_eq!(all, b"line1\nline2\nline3\nline4\n");
    }

    #[test]
    fn last_n_lines_no_trailing_newline() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"line1\nline2\nline3");

        let last1 = buf.last_n_lines(1);
        assert_eq!(last1, b"line3");

        let last2 = buf.last_n_lines(2);
        assert_eq!(last2, b"line2\nline3");
    }

    #[test]
    fn last_n_lines_zero() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"data");
        assert!(buf.last_n_lines(0).is_empty());
    }

    #[test]
    fn empty_buffer() {
        let buf = ScrollbackBuffer::new(1024);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert!(buf.last_n_bytes(10).is_empty());
        assert!(buf.last_n_lines(10).is_empty());
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = ScrollbackBuffer::new(1024);
        buf.append(b"data");
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn default_is_1mib() {
        let buf = ScrollbackBuffer::default();
        assert_eq!(buf.max_bytes, 1024 * 1024);
    }
}
