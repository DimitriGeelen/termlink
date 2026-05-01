/// Strip ANSI escape sequences and carriage returns from a string.
///
/// Handles CSI sequences (\x1b[...), OSC sequences (\x1b]...\x07 or \x1b]...\x1b\\),
/// and bare escape sequences.
pub(crate) fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() || ch == 'K' || ch == 'J' || ch == 'H' {
                            break;
                        }
                    }
                }
                Some(']') => {
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
