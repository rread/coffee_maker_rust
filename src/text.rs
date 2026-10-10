//! Paragraph word-wrapping.

/// Lines are broken at or before this offset from the start of the line.
const WIDTH: usize = 79;

/// Break `s` into lines.
///
/// Each line ends just before the last run of spaces within `WIDTH` bytes
/// of its start, so a line holds at most `WIDTH + 1` bytes. If no space
/// falls in the back half of that window, the line is cut mid-word at
/// `WIDTH / 2 + 1` bytes. Leading spaces are kept on the first line (to
/// allow indenting) and dropped from the others.
///
/// Works on bytes so that arbitrary player input can't split a `char`.
pub fn wrap(s: &str) -> Vec<&[u8]> {
    let s = s.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    while start < s.len() {
        let stop = find_stop(s, start);
        lines.push(&s[start..=stop]);
        start = stop + 1;
        while start < s.len() && s[start] == b' ' {
            start += 1;
        }
    }
    lines
}

/// Index of the last byte of the line beginning at `start`.
fn find_stop(s: &[u8], start: usize) -> usize {
    let last = s.len() - 1;
    if last <= start + WIDTH {
        return last;
    }
    let mut stop = start + WIDTH;
    while stop > start + WIDTH / 2 && s[stop] != b' ' {
        stop -= 1;
    }
    while stop > start && s[stop] == b' ' {
        stop -= 1;
    }
    stop
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        wrap(s)
            .into_iter()
            .map(|l| String::from_utf8(l.to_vec()).unwrap())
            .collect()
    }

    #[test]
    fn empty() {
        assert!(wrap("").is_empty());
    }

    #[test]
    fn up_to_80_bytes_fits_on_one_line() {
        let s = "x".repeat(80);
        assert_eq!(lines(&s), [s]);
    }

    #[test]
    fn breaks_at_last_space_in_window() {
        // A space at offset 79 is the latest possible break.
        let s = format!("{} {}", "a".repeat(79), "b".repeat(5));
        assert_eq!(lines(&s), ["a".repeat(79), "b".repeat(5)]);
        // The 81st byte is never considered.
        let s = format!("{} {}", "a".repeat(80), "b".repeat(5));
        assert_eq!(
            lines(&s),
            [
                "a".repeat(40),
                format!("{} {}", "a".repeat(40), "b".repeat(5))
            ]
        );
    }

    #[test]
    fn double_space_at_break_is_dropped() {
        let s = format!("{}.  {}", "a".repeat(70), "b".repeat(20));
        assert_eq!(lines(&s), [format!("{}.", "a".repeat(70)), "b".repeat(20)]);
    }

    #[test]
    fn no_space_cuts_at_40_bytes() {
        let s = "x".repeat(100);
        assert_eq!(lines(&s), ["x".repeat(40), "x".repeat(60)]);
    }

    #[test]
    fn space_only_in_front_half_still_cuts_at_40() {
        let s = format!("{} {}", "a".repeat(10), "b".repeat(90));
        assert_eq!(
            lines(&s),
            [
                format!("{} {}", "a".repeat(10), "b".repeat(29)),
                "b".repeat(61)
            ]
        );
    }

    #[test]
    fn leading_indent_kept_on_first_line_only() {
        let s = format!("  {} {}", "a".repeat(70), "b".repeat(20));
        assert_eq!(lines(&s), [format!("  {}", "a".repeat(70)), "b".repeat(20)]);
    }

    #[test]
    fn non_ascii_never_panics() {
        let s = "é".repeat(100);
        let total: usize = wrap(&s).iter().map(|l| l.len()).sum();
        assert_eq!(total, s.len());
    }
}
