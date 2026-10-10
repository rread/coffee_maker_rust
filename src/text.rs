//! Paragraph word-wrapping.

/// The original's line length. Lines hold up to 80 bytes.
pub const COLUMNS: usize = 80;

/// Break `s` into lines of at most `columns` bytes.
///
/// With `width = columns - 1`, each line ends just before the last run of
/// spaces within `width` bytes of its start. If no space falls in the back
/// half of that window, the line is cut mid-word at `width / 2 + 1` bytes.
/// Leading spaces are kept on the first line (to allow indenting) and
/// dropped from the others.
///
/// Works on bytes so that arbitrary player input can't split a `char`.
pub fn wrap(s: &str, columns: usize) -> Vec<&[u8]> {
    let width = columns.max(2) - 1;
    let s = s.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    while start < s.len() {
        let stop = find_stop(s, start, width);
        lines.push(&s[start..=stop]);
        start = stop + 1;
        while start < s.len() && s[start] == b' ' {
            start += 1;
        }
    }
    lines
}

/// Index of the last byte of the line beginning at `start`.
fn find_stop(s: &[u8], start: usize, width: usize) -> usize {
    let last = s.len() - 1;
    if last <= start + width {
        return last;
    }
    let mut stop = start + width;
    while stop > start + width / 2 && s[stop] != b' ' {
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
        wrap(s, COLUMNS)
            .into_iter()
            .map(|l| String::from_utf8(l.to_vec()).unwrap())
            .collect()
    }

    #[test]
    fn empty() {
        assert!(wrap("", COLUMNS).is_empty());
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
    fn narrower_columns_use_the_same_rule() {
        let s = "You see an electric kettle, a pour-over funnel that needs cleaning.";
        let narrow: Vec<_> = wrap(s, 30)
            .into_iter()
            .map(|l| String::from_utf8(l.to_vec()).unwrap())
            .collect();
        assert_eq!(
            narrow,
            [
                "You see an electric kettle, a",
                "pour-over funnel that needs",
                "cleaning."
            ]
        );
        assert!(wrap(&"x".repeat(100), 30).iter().all(|l| l.len() <= 30));
    }

    #[test]
    fn non_ascii_never_panics() {
        let s = "é".repeat(100);
        let total: usize = wrap(&s, COLUMNS).iter().map(|l| l.len()).sum();
        assert_eq!(total, s.len());
    }
}
