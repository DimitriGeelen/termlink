use std::collections::VecDeque;

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
            self.buffer.extend(&data[start..]);
            return;
        }

        // Make room if needed
        let overflow = (self.buffer.len() + data.len()).saturating_sub(self.max_bytes);
        if overflow > 0 {
            self.buffer.drain(..overflow);
        }

        self.buffer.extend(data);
    }

    /// Return the last N bytes of output.
    pub fn last_n_bytes(&self, n: usize) -> Vec<u8> {
        let start = self.buffer.len().saturating_sub(n);
        self.buffer.iter().skip(start).copied().collect()
    }

    /// Return the last N lines of output.
    ///
    /// A "line" is delimited by `\n`. The returned bytes include the newlines.
    /// If the buffer contains fewer than N lines, returns all content.
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
