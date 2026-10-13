// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Sidecar subtitles: SubRip and WebVTT, in and out.
//!
//! A caption file is the one caption shape every player, every uploader and
//! every other editor agrees on, so it is both how words arrive from
//! somewhere else and how they leave. What is here is text and nothing else:
//! no files, no FFmpeg, no engine type, so a cue list can be parsed, cut,
//! remapped and written back without a project being open.
//!
//! Both formats are read by the same pass, because they differ in their
//! header and their decimal mark and in almost nothing else: a block, a
//! timing line holding two stamps either side of `-->`, and the words under
//! it. WebVTT's extras - the `WEBVTT` header, `NOTE` and `STYLE` and
//! `REGION` blocks, cue settings, and the `<v Roger>` tags - are recognised
//! and dropped rather than parsed, since a subtitle file's job here is to
//! say what was heard and when.
//!
//! Word-level timings do not survive this format: neither SRT nor VTT has a
//! place for them. A cue that arrives from a file therefore carries its
//! words and no timings for them, and [`crate::captionmap`] is where a cue's
//! words are carried through a cut. See also [`crate::sections`] for the
//! word grouping a transcription's own timings allow.

/// One cue: what was said, and when, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    /// Seconds from the start of the media the words are heard over.
    pub start: f64,
    /// Seconds; always after [`Cue::start`], since a cue that ends where it
    /// began is a cue nothing can read.
    pub end: f64,
    /// The words, one line's worth of them joined by single spaces.
    pub text: String,
}

/// How close two times may be and still count as the same time. Float
/// seconds that should meet exactly sometimes miss by a hair.
const TOUCH: f64 = 1e-6;

/// The most decimal digits a stamp's fraction carries. SRT writes three and
/// tolerates fewer on the way in; a fourth digit is read and then rounded
/// away, the way a player rounds it.
const MILLIS: u32 = 3;

/// Reads SubRip or WebVTT text into cues, in the order they are heard.
///
/// Tolerant on purpose: a byte-order mark, CRLF line endings, a lone CR,
/// cue indices, a `WEBVTT` header with anything after it on the line,
/// `NOTE` / `STYLE` / `REGION` blocks, cue settings that leaked onto their
/// own line, inline tags, and a `MM:SS.mmm` stamp with the hours left off
/// all arrive in real files. A block with no timing line in it is not a cue
/// and is skipped, which is how a header or a note is skipped without
/// anything naming it.
pub fn parse(text: &str) -> Vec<Cue> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut cues = Vec::new();

    // A block is what sits between two blank lines. `split` on a predicate
    // rather than on "\n\n" because a file may separate its blocks with a
    // line of spaces, and may end without a trailing newline at all.
    for block in split_blocks(text) {
        let lines: Vec<&str> = block
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();

        // The timing line is what makes a block a cue. Everything above it
        // is an index or a header; everything below it is words.
        let Some(at) = lines.iter().position(|line| timing(line).is_some()) else {
            continue;
        };
        let Some((start, end)) = timing(lines[at]) else {
            continue;
        };
        if end <= start + TOUCH {
            continue;
        }

        let text = lines[at + 1..]
            .iter()
            // A VTT cue setting that ended up on its own line is not words.
            .filter(|line| !is_block_keyword(line))
            .map(|line| strip_tags(line))
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        cues.push(Cue {
            start: round_millis(start),
            end: round_millis(end),
            text,
        });
    }

    // A file out of order is still a file; the timeline wants them as heard.
    cues.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.end.total_cmp(&b.end)));
    cues
}

/// The two ends of a timing line, or None when the line is not one.
///
/// Reads `HH:MM:SS,mmm --> HH:MM:SS.mmm` and every relaxed form of it: the
/// hours may be left off, the fraction may be one to three digits, and the
/// mark between the seconds and it may be a comma or a point. What follows
/// the second stamp is a WebVTT cue setting and is not part of it.
fn timing(line: &str) -> Option<(f64, f64)> {
    let arrow = line.find("-->")?;
    let start = stamp(&line[..arrow])?;
    let end = stamp(&line[arrow + 3..])?;
    Some((start, end))
}

/// One timestamp as seconds, read from the front of a slice that may carry
/// anything after it.
///
/// Reading forwards rather than taking the whole slice is what lets a VTT
/// timing line keep its cue settings: in `00:00:02.000 line:90%`, the point
/// in `line:90%` is not a decimal mark, and a reader that looked for the
/// last one on the line would try to parse `90% position:50%` as a fraction
/// of a second and lose the cue.
fn stamp(slice: &str) -> Option<f64> {
    let slice = slice.trim_start();
    let end = slice
        .find(|c: char| !c.is_ascii_digit() && c != ':' && c != ',' && c != '.')
        .unwrap_or(slice.len());
    let slice = slice[..end].trim_end();

    let at = slice.rfind([',', '.'])?;
    let (whole, fraction) = (&slice[..at], &slice[at + 1..]);
    if fraction.is_empty() || fraction.len() > MILLIS as usize || !all_digits(fraction) {
        return None;
    }

    // One to three digits, read as the leading digits of a millisecond
    // count: `.5` is half a second, not five milliseconds.
    let mut millis = 0u32;
    for (place, digit) in fraction.bytes().enumerate() {
        millis += (digit - b'0') as u32 * 10u32.pow(2 - place as u32);
    }

    let mut clock = whole.split(':');
    let first = clock.next()?.trim();
    if !all_digits(first) {
        return None;
    }
    let (hours, minutes, seconds) = match (clock.next(), clock.next(), clock.next()) {
        // HH:MM:SS
        (Some(m), Some(s), None) => {
            if !all_digits(m.trim()) || !all_digits(s.trim()) {
                return None;
            }
            (
                first.parse::<u32>().ok()?,
                m.trim().parse::<u32>().ok()?,
                s.trim().parse::<u32>().ok()?,
            )
        }
        // MM:SS, which WebVTT permits when the hours are zero
        (Some(s), None, None) => {
            if !all_digits(s.trim()) {
                return None;
            }
            (0, first.parse::<u32>().ok()?, s.trim().parse::<u32>().ok()?)
        }
        // Anything longer is not a clock
        _ => return None,
    };
    if minutes > 59 || seconds > 59 {
        return None;
    }

    Some(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + millis as f64 / 1000.0)
}

/// Whether every byte is an ASCII digit, and there is at least one.
fn all_digits(slice: &str) -> bool {
    !slice.is_empty() && slice.bytes().all(|byte| byte.is_ascii_digit())
}

/// Drops inline tags: `<v Roger>`, `<b>`, `<00:00:01.000>`, and the rest.
///
/// A tag's contents go with it, which is right for a voice or a class and
/// wrong for nothing that carries words: every tag VTT defines wraps either
/// styling or a name.
fn strip_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        rest = match rest[open + 1..].find('>') {
            Some(close) => &rest[open + 1 + close + 1..],
            // An unclosed '<' is a character, not a tag: the rest is words.
            None => {
                out.push_str(&rest[open..]);
                return out;
            }
        };
    }
    out.push_str(rest);
    out
}

/// Whether a line opens one of WebVTT's block kinds, which hold no words.
fn is_block_keyword(line: &str) -> bool {
    ["WEBVTT", "NOTE", "STYLE", "REGION"]
        .iter()
        .any(|keyword| line == *keyword || line.starts_with(&format!("{keyword} ")))
}

/// Splits on blank lines, where a blank line may carry spaces or tabs.
fn split_blocks(text: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = 0;
    let mut index = 0;

    for line in text.split_inclusive('\n') {
        let length = line.len();
        if line.trim().is_empty() {
            if index > start {
                blocks.push(&text[start..index]);
            }
            start = index + length;
        }
        index += length;
    }
    if start < text.len() {
        blocks.push(&text[start..]);
    }
    blocks
}

/// Seconds rounded to what a subtitle file can write, so a cue read back is
/// the cue that was written.
fn round_millis(seconds: f64) -> f64 {
    (seconds * 1000.0).round() / 1000.0
}

/// Seconds as an SRT stamp: `HH:MM:SS,mmm`, the comma being SubRip's.
pub fn srt_stamp(seconds: f64) -> String {
    clock(seconds).replace('.', ",")
}

/// Seconds as a WebVTT stamp: `HH:MM:SS.mmm`.
pub fn vtt_stamp(seconds: f64) -> String {
    clock(seconds)
}

/// The `HH:MM:SS.mmm` both formats share, from a count of seconds that is
/// held to zero rather than allowed to go negative and print a sign.
fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    let whole = seconds.floor() as u64;
    let millis = ((seconds - whole as f64) * 1000.0).round() as u64;
    // A fraction that rounds up to a whole second carries into it, the way
    // a player's own formatter would.
    let (whole, millis) = if millis >= 1000 {
        (whole + 1, millis - 1000)
    } else {
        (whole, millis)
    };
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        whole / 3600,
        whole / 60 % 60,
        whole % 60,
        millis
    )
}

/// The cues as an SRT file's text: numbered, comma-separated stamps, a
/// blank line between.
pub fn to_srt(cues: &[Cue]) -> String {
    let mut out = String::new();
    for (index, cue) in cues.iter().enumerate() {
        out.push_str(&(index + 1).to_string());
        out.push('\n');
        out.push_str(&srt_stamp(cue.start));
        out.push_str(" --> ");
        out.push_str(&srt_stamp(cue.end));
        out.push('\n');
        out.push_str(cue.text.trim());
        out.push_str("\n\n");
    }
    out
}

/// The cues as a WebVTT file's text: the header, then unnumbered cues.
pub fn to_vtt(cues: &[Cue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for cue in cues {
        out.push_str(&vtt_stamp(cue.start));
        out.push_str(" --> ");
        out.push_str(&vtt_stamp(cue.end));
        out.push('\n');
        out.push_str(cue.text.trim());
        out.push_str("\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(start: f64, end: f64, text: &str) -> Cue {
        Cue {
            start,
            end,
            text: text.to_owned(),
        }
    }

    #[test]
    fn an_srt_file_reads_as_the_cues_it_holds() {
        let text = "1\n00:00:01,000 --> 00:00:03,500\nHello there\n\n\
                    2\n00:00:04,000 --> 00:00:05,250\nSecond line\n\n";
        assert_eq!(
            parse(text),
            vec![cue(1.0, 3.5, "Hello there"), cue(4.0, 5.25, "Second line")]
        );
    }

    #[test]
    fn a_webvtt_file_reads_as_the_same_cues() {
        let text = "WEBVTT\n\n00:00:01.000 --> 00:00:03.500\nHello there\n\n\
                    00:00:04.000 --> 00:00:05.250\nSecond line\n";
        assert_eq!(
            parse(text),
            vec![cue(1.0, 3.5, "Hello there"), cue(4.0, 5.25, "Second line")]
        );
    }

    #[test]
    fn a_file_written_by_the_one_format_is_read_by_the_other() {
        // The only thing that differs is the decimal mark, and a reader
        // that accepts both means a cue list survives any round trip.
        let cues = vec![cue(0.5, 1.25, "one"), cue(2.0, 3.0, "two")];
        assert_eq!(parse(&to_srt(&cues)), cues);
        assert_eq!(parse(&to_vtt(&cues)), cues);
    }

    #[test]
    fn the_hours_may_be_left_off_a_webvtt_stamp() {
        assert_eq!(
            parse("WEBVTT\n\n01:02.500 --> 01:05.000\nShort form\n"),
            vec![cue(62.5, 65.0, "Short form")]
        );
    }

    #[test]
    fn a_byte_order_mark_and_windows_line_endings_are_read_through() {
        let text = "\u{feff}1\r\n00:00:01,000 --> 00:00:02,000\r\nMarked\r\n\r\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Marked")]);
    }

    #[test]
    fn a_fraction_of_one_or_two_digits_is_read_as_a_fraction_of_a_second() {
        // `.5` is half a second, not five milliseconds: the digits are the
        // leading digits of a millisecond count.
        assert_eq!(stamp("00:00:01.5"), Some(1.5));
        assert_eq!(stamp("00:00:01.25"), Some(1.25));
        assert_eq!(stamp("00:00:01.250"), Some(1.25));
    }

    #[test]
    fn a_fourth_decimal_digit_is_read_and_then_rounded_away() {
        assert_eq!(
            stamp("00:00:01.2500"),
            None,
            "too many digits is not a stamp"
        );
        assert_eq!(parse("1\n00:00:01,2504 --> 00:00:02,000\nx\n"), vec![]);
    }

    #[test]
    fn voice_and_class_tags_go_and_their_words_stay() {
        let text = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Roger>Good <b>morning</b>\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Good morning")]);
    }

    #[test]
    fn an_unclosed_angle_bracket_is_a_character_and_not_a_tag() {
        let text = "1\n00:00:01,000 --> 00:00:02,000\na < b\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "a < b")]);
    }

    #[test]
    fn a_cue_setting_trailing_the_timing_line_is_not_words() {
        // The settings belong to the timing line, so they are above the body
        // and never reach it - which is why a line:90% position survives as a
        // position and does not become the first word of the caption.
        let text = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000 line:90% position:50%\nWords here\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Words here")]);
    }

    #[test]
    fn a_note_block_of_its_own_holds_no_cues_and_leaks_no_words() {
        let text = "WEBVTT\n\nNOTE\na comment on two lines\n\n\
                    00:00:01.000 --> 00:00:02.000\nWords here\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Words here")]);
    }

    #[test]
    fn a_note_block_and_a_style_block_hold_no_cues() {
        let text = "WEBVTT - a title is allowed here\n\nNOTE\nThis is a comment\n\n\
                    STYLE\n::cue { color: white }\n\n\
                    00:00:01.000 --> 00:00:02.000\nOnly this\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Only this")]);
    }

    #[test]
    fn the_cue_index_is_optional_and_ignored_when_present() {
        let with = parse("7\n00:00:01,000 --> 00:00:02,000\nWords\n");
        let without = parse("00:00:01,000 --> 00:00:02,000\nWords\n");
        assert_eq!(with, without);
        assert_eq!(with, vec![cue(1.0, 2.0, "Words")]);
    }

    #[test]
    fn a_cue_that_ends_where_it_begins_is_not_a_cue() {
        assert_eq!(parse("1\n00:00:01,000 --> 00:00:01,000\nNothing\n"), vec![]);
        assert_eq!(
            parse("1\n00:00:02,000 --> 00:00:01,000\nBackwards\n"),
            vec![]
        );
    }

    #[test]
    fn two_lines_of_one_cue_become_one_line_of_words() {
        let text = "1\n00:00:01,000 --> 00:00:03,000\nFirst line\nSecond line\n";
        assert_eq!(parse(text), vec![cue(1.0, 3.0, "First line Second line")]);
    }

    #[test]
    fn cues_out_of_order_come_back_in_the_order_they_are_heard() {
        let text = "1\n00:00:05,000 --> 00:00:06,000\nLater\n\n\
                    2\n00:00:01,000 --> 00:00:02,000\nSooner\n";
        assert_eq!(
            parse(text),
            vec![cue(1.0, 2.0, "Sooner"), cue(5.0, 6.0, "Later")]
        );
    }

    #[test]
    fn a_block_with_no_timing_line_in_it_is_skipped() {
        let text = "WEBVTT\n\nKind: captions\nLanguage: en\n\n\
                    00:00:01.000 --> 00:00:02.000\nReal cue\n";
        assert_eq!(parse(text), vec![cue(1.0, 2.0, "Real cue")]);
    }

    #[test]
    fn an_empty_file_and_a_file_of_nothing_but_a_header_have_no_cues() {
        assert_eq!(parse(""), vec![]);
        assert_eq!(parse("WEBVTT\n\n"), vec![]);
        assert_eq!(parse("\n\n\n"), vec![]);
    }

    #[test]
    fn words_in_any_script_survive_the_round_trip() {
        // The point of a local editor is that it is nobody's business what
        // language the words are in, so the writer escapes nothing.
        let cues = vec![
            cue(0.0, 1.5, "Selamat pagi"),
            cue(1.5, 3.0, "こんにちは世界"),
            cue(3.0, 4.5, "مرحبا بالعالم"),
        ];
        assert_eq!(parse(&to_srt(&cues)), cues);
        assert_eq!(parse(&to_vtt(&cues)), cues);
    }

    #[test]
    fn a_stamp_writes_the_hours_it_was_given() {
        assert_eq!(srt_stamp(3661.5), "01:01:01,500");
        assert_eq!(vtt_stamp(3661.5), "01:01:01.500");
        assert_eq!(srt_stamp(0.0), "00:00:00,000");
    }

    #[test]
    fn a_negative_time_writes_as_zero_rather_than_as_a_sign() {
        assert_eq!(srt_stamp(-1.0), "00:00:00,000");
        assert_eq!(vtt_stamp(-0.5), "00:00:00.000");
    }

    #[test]
    fn a_fraction_that_rounds_up_to_a_second_carries_into_it() {
        assert_eq!(vtt_stamp(1.9999), "00:00:02.000");
        assert_eq!(vtt_stamp(59.9999), "00:01:00.000");
    }

    #[test]
    fn the_srt_numbers_its_cues_from_one_and_the_vtt_does_not_number_them() {
        let cues = vec![cue(0.0, 1.0, "a"), cue(1.0, 2.0, "b")];
        assert!(to_srt(&cues).starts_with("1\n00:00:00,000"));
        assert!(to_srt(&cues).contains("\n\n2\n00:00:01,000"));
        assert!(to_vtt(&cues).starts_with("WEBVTT\n\n00:00:00.000"));
        assert!(!to_vtt(&cues).contains("\n1\n"));
    }

    #[test]
    fn a_cue_read_back_is_the_cue_that_was_written() {
        // The writer rounds to milliseconds because the format cannot hold
        // more, so a cue whose times are already on that grid is exact.
        let cues = vec![
            cue(0.0, 0.04, "a"),
            cue(12.34, 12.71, "b"),
            cue(3599.999, 3600.001, "c"),
        ];
        assert_eq!(parse(&to_srt(&cues)), cues);
        assert_eq!(parse(&to_vtt(&cues)), cues);
    }
}
