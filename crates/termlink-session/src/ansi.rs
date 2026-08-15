/// True if `ch` is an ECMA-48 CSI *final byte* (`0x40..=0x7E`), which ends a
/// control sequence.
///
/// T-2728: this deliberately is NOT `is_ascii_alphabetic()`. A CSI sequence is
/// `ESC [`, then parameter bytes (`0x30..=0x3F`), then intermediate bytes
/// (`0x20..=0x2F`), then ONE final byte in `0x40..=0x7E` — a range that
/// includes `~ @ [ \ ] ^ _ ` { | } ` as well as the letters. Terminating only
/// on letters meant `"\x1b[3~hello"` never ended at the `~`, so the scan
/// swallowed the `h` and returned `"ello"`. Bracketed paste (`ESC [ 200~` /
/// `ESC [ 201~`) hits this on every paste.
fn is_csi_final(ch: char) -> bool {
    ('\x40'..='\x7e').contains(&ch)
}

/// Strip ANSI escape sequences and carriage returns from a string.
///
/// Handles CSI sequences (`ESC [ … final`), OSC sequences
/// (`ESC ] … BEL` or `ESC ] … ESC \`), the DCS/SOS/PM/APC string sequences
/// (`ESC P` / `ESC X` / `ESC ^` / `ESC _`, each terminated by ST = `ESC \`),
/// and bare two-character escape sequences.
///
/// This is the single implementation for the workspace (T-2728). It previously
/// existed as two near-identical copies here and in `termlink-cli/src/util.rs`,
/// which is how the two defects above survived: a reader fixing one copy would
/// leave the other wrong. `termlink-cli` now re-exports this function.
pub fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if is_csi_final(ch) {
                            break;
                        }
                    }
                }
                // OSC and the string sequences share a terminator discipline:
                // run to ST (`ESC \`), with BEL also accepted for OSC by long
                // convention. Grouping them is what stops a DCS/APC payload —
                // e.g. a kitty-graphics blob — being emitted as visible text
                // by the old catch-all "skip one character" arm (T-2728).
                Some(']') | Some('P') | Some('X') | Some('^') | Some('_') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c == '\r' {
            continue;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-2728: a CSI sequence ends at any ECMA-48 final byte (0x40..=0x7E),
    /// not only at an ASCII letter. `~` is the canonical counter-example.
    ///
    /// Load-bearing: with the old `ch.is_ascii_alphabetic()` terminator the
    /// scan runs past the `~` and eats the following character, so
    /// `"\x1b[3~hello"` returned `"ello"` — a silent wrong answer on an API
    /// whose entire contract is fidelity.
    #[test]
    fn strip_ansi_csi_non_alphabetic_final_byte() {
        assert_eq!(strip_ansi_codes("\x1b[3~hello"), "hello");
        assert_eq!(strip_ansi_codes("\x1b[2~world"), "world");
        // Bracketed paste wraps real user input on every paste.
        assert_eq!(
            strip_ansi_codes("\x1b[200~pasted text\x1b[201~"),
            "pasted text"
        );
        // The rest of the 0x40..=0x7E range that is not a letter.
        assert_eq!(strip_ansi_codes("\x1b[0@at"), "at");
        assert_eq!(strip_ansi_codes("\x1b[1`tick"), "tick");
        assert_eq!(strip_ansi_codes("\x1b[1{brace"), "brace");
        assert_eq!(strip_ansi_codes("\x1b[1|pipe"), "pipe");
    }

    /// T-2728: DCS / SOS / PM / APC are *string* sequences terminated by ST
    /// (`ESC \`). The old `_ =>` arm skipped a single character, so the whole
    /// payload was emitted as visible text.
    #[test]
    fn strip_ansi_string_sequences_consume_payload() {
        // DCS
        assert_eq!(strip_ansi_codes("a\x1bPq some payload \x1b\\b"), "ab");
        // APC — what a terminal-image or kitty-graphics payload looks like.
        assert_eq!(strip_ansi_codes("a\x1b_Gf=100,s=1;AAAA\x1b\\b"), "ab");
        // PM
        assert_eq!(strip_ansi_codes("a\x1b^private\x1b\\b"), "ab");
        // SOS
        assert_eq!(strip_ansi_codes("a\x1bXstring\x1b\\b"), "ab");
        // Unterminated payload must not leak either.
        assert_eq!(strip_ansi_codes("a\x1bPunterminated"), "a");
    }

    #[test]
    fn strip_ansi_plain_text_passthrough() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
        assert_eq!(strip_ansi_codes(""), "");
        assert_eq!(strip_ansi_codes("line1\nline2\n"), "line1\nline2\n");
    }

    #[test]
    fn strip_ansi_csi_color_codes() {
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi_codes("\x1b[1;32mbold green\x1b[0m"), "bold green");
        assert_eq!(
            strip_ansi_codes("\x1b[38;5;196mextended\x1b[0m"),
            "extended"
        );
    }

    #[test]
    fn strip_ansi_csi_cursor_movement() {
        assert_eq!(strip_ansi_codes("\x1b[2Aup two"), "up two");
        assert_eq!(strip_ansi_codes("\x1b[10Bdown ten"), "down ten");
        assert_eq!(strip_ansi_codes("before\x1b[Kafter"), "beforeafter");
        assert_eq!(strip_ansi_codes("before\x1b[2Jafter"), "beforeafter");
        assert_eq!(strip_ansi_codes("\x1b[5;10Hpositioned"), "positioned");
    }

    #[test]
    fn strip_ansi_osc_title_setting() {
        assert_eq!(
            strip_ansi_codes("\x1b]0;My Terminal Title\x07rest"),
            "rest"
        );
        assert_eq!(
            strip_ansi_codes("\x1b]0;Title\x1b\\rest"),
            "rest"
        );
    }

    #[test]
    fn strip_ansi_carriage_return() {
        assert_eq!(strip_ansi_codes("line\r\n"), "line\n");
        assert_eq!(strip_ansi_codes("overwrite\rvisible"), "overwritevisible");
        assert_eq!(strip_ansi_codes("\r"), "");
    }

    #[test]
    fn strip_ansi_mixed_content() {
        let input = "\x1b[1;34m$ \x1b[0mecho \x1b[32m\"hello\"\x1b[0m\r\nhello\r\n";
        let expected = "$ echo \"hello\"\nhello\n";
        assert_eq!(strip_ansi_codes(input), expected);
    }

    #[test]
    fn strip_ansi_bare_escape_consumed() {
        assert_eq!(strip_ansi_codes("\x1bXrest"), "rest");
    }

    #[test]
    fn strip_plain_text() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
    }

    #[test]
    fn strip_csi_color() {
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strip_osc_title() {
        assert_eq!(
            strip_ansi_codes("\x1b]0;My Title\x07text"),
            "text"
        );
    }

    #[test]
    fn strip_carriage_return() {
        assert_eq!(strip_ansi_codes("line\r\n"), "line\n");
    }
}
