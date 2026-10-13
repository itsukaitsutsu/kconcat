// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! A cue's words, and what a cut does to them.
//!
//! A caption is not one thing but two: the words on screen, and the moment
//! each of them is heard. The second is what a word-level layout, a karaoke
//! reveal and a word editor all need, and it is what a cut puts in danger -
//! a caption describes the *source*, so once spans of that source are gone
//! the times on the cue no longer mean what they said.
//!
//! Two pieces, both pure:
//!
//! 1. [`clean_words`] takes whatever a recogniser emitted and returns words
//!    that hold the invariants everything downstream assumes: sorted, inside
//!    their cue, never overlapping a neighbour, with punctuation riding on
//!    the word before it rather than standing alone.
//! 2. [`remap`] moves a cue list through a cut. A cue inside a kept range is
//!    shifted earlier by everything removed before it; a cue crossing a cut
//!    is split into one piece per kept fragment; pieces that end up
//!    back-to-back holding the same words are joined again, because a cut
//!    *inside* one sentence should not leave two captions saying it; a cue
//!    wholly inside a removed range disappears, which is the point.
//!
//! Word timings live nowhere else. Neither [`crate::subtitle`] nor a burned
//! render can carry them, so this is the only place a word's own start and
//! end survive an edit - and why a cue keeps [`Cue::word_source`] beside
//! them, to say whether those times were heard or merely estimated.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::sections::round_millis;

/// How close two times may be and still count as the same time.
const TOUCH: f64 = 1e-6;

/// The shortest a word may be, in seconds. A word the recogniser gave no
/// length to is still displayable and still editable, so it is padded rather
/// than dropped.
pub const MIN_WORD: f64 = 0.02;

/// The floor a zero-length word is padded to. Slightly longer than
/// [`MIN_WORD`] because it is the case where nothing was heard at all, and a
/// word that flashes for under a frame reads as a glitch.
const MIN_WORD_PAD: f64 = 0.03;

/// The most words one cue may hold: a guard against a pathological run of
/// one-word cues being merged by hand into something unreadable.
pub const MAX_WORDS: usize = 200;

/// The shortest fragment of a cue worth keeping after a cut, in seconds.
/// Under it the piece would be on screen for less than a frame's worth of
/// reading time, so it goes with the cut instead.
pub const MIN_PIECE: f64 = 0.05;

/// Where a cue's word times came from, which is what makes an estimate
/// honest rather than a claim about the audio.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum WordSource {
    /// Heard: a recogniser with word-level output timed these.
    Whisper,
    /// Heard: the same, from a token-timestamp engine.
    Whispercpp,
    /// Not heard: spread across the cue by character weight, because a
    /// subtitle file has no word times and a word layout still wants some.
    Estimated,
    /// The words were changed by hand, so the times are whatever they were
    /// when the count last matched.
    Edited,
}

/// One word, and the moment it is heard: seconds against the same clock as
/// the cue that holds it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Word {
    /// The word as it is displayed, punctuation included.
    #[serde(rename = "w")]
    pub text: String,
    /// Seconds where the word begins.
    #[serde(rename = "s")]
    pub start: f64,
    /// Seconds where it ends; after [`Word::start`], by at least
    /// [`MIN_WORD`] once [`clean_words`] has been over it.
    #[serde(rename = "e")]
    pub end: f64,
}

/// One caption: the words on screen, when it is on screen, and when each
/// word is heard.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cue {
    /// Seconds where the caption appears.
    pub start: f64,
    /// Seconds where it goes away.
    pub end: f64,
    /// The words as displayed, which is [`words`] joined - or, for a cue
    /// that came from a file, all it has.
    pub text: String,
    /// Each word's own timing. Empty is legitimate: a cue from an SRT has
    /// none, and a word layout then works from an estimate instead.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<Word>,
    /// Where those times came from, and so how much to trust them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_source: Option<WordSource>,
    /// A stable name for the cue, for a document that keeps one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Which caption lane the cue sits on, for a document with more than one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    /// A look the cue carries of its own. Held as it arrived, because what
    /// goes in it is the text crate's business and not this one's.
    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub style: Map<String, Value>,
}

fn is_empty_map(map: &Map<String, Value>) -> bool {
    map.is_empty()
}

impl From<Cue> for crate::subtitle::Cue {
    /// The same cue with its words left behind, which is what a sidecar file
    /// can carry.
    fn from(cue: Cue) -> Self {
        Self {
            start: cue.start,
            end: cue.end,
            text: cue.text,
        }
    }
}

impl From<crate::subtitle::Cue> for Cue {
    /// A cue from a file: the words and their span, and no timings inside it.
    /// [`crate::sections`] and a word layout both work from here; the times
    /// they use are estimates and say so.
    fn from(cue: crate::subtitle::Cue) -> Self {
        Self {
            start: cue.start,
            end: cue.end,
            text: cue.text,
            ..Self::default()
        }
    }
}

/// Whether a character is a letter or a digit in any script.
///
/// The question is never "is this ASCII" - a cue in Japanese, Arabic or Thai
/// is made entirely of characters this has to say yes to, or every word in it
/// would read as punctuation and be glued to the one before.
fn is_alphanumeric(character: char) -> bool {
    character.is_alphanumeric()
}

/// Word timings estimated from the words themselves, spread across
/// `[start, end]` in proportion to their length in characters.
///
/// What a cue with no heard timings gets (a subtitle file, and - until
/// the transcriber's token timestamps land - a whisper segment, which
/// is timed as a phrase and no finer). Longer words take longer to say;
/// that is the whole of the model, and [`WordSource::Estimated`] is how
/// a caller says these times are this and not heard ones. The result
/// goes through [`clean_words`], so it obeys the same rules as heard
/// timings: sorted, inside the cue, never overlapping.
pub fn estimate_words(text: &str, start: f64, end: f64) -> Vec<Word> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() || end <= start {
        return Vec::new();
    }
    let total: f64 = words.iter().map(|word| word.chars().count() as f64).sum();
    if total <= 0.0 {
        return Vec::new();
    }
    let span = end - start;
    let mut at = start;
    let raw: Vec<Word> = words
        .iter()
        .map(|word| {
            let duration = span * (word.chars().count() as f64) / total;
            let word = Word {
                text: (*word).to_owned(),
                start: at,
                end: at + duration,
            };
            at += duration;
            word
        })
        .collect();
    clean_words(&raw, Some(start), Some(end)).0
}

/// Puts words into the shape everything downstream assumes.
///
/// Accepts either spelling a recogniser uses, drops the empties, pads the
/// ones that were given no length, clips to `[window_start, window_end]`
/// when a window is given, glues a punctuation-only token onto the word
/// before it, sorts by time, and then makes sure no two words share a slice
/// of the timeline: the later one starts where the earlier ended.
///
/// That last rule is the one the rest of the file leans on. A word layout, a
/// karaoke reveal and a word editor all ask "which word is being heard now",
/// and the question has one answer only because the spans do not overlap.
///
/// Returns the words, and whether any of them changed - which is what turns
/// a cue's [`WordSource`] into [`WordSource::Edited`] rather than leaving it
/// claiming to be what the recogniser said.
pub fn clean_words(
    words: &[Word],
    window_start: Option<f64>,
    window_end: Option<f64>,
) -> (Vec<Word>, bool) {
    let mut out: Vec<Word> = Vec::with_capacity(words.len().min(MAX_WORDS));

    for word in words {
        let text = word.text.trim();
        if text.is_empty() {
            continue;
        }
        let mut start = word.start;
        if !start.is_finite() {
            continue;
        }
        // A word given no end, or an end before its start, is still a word:
        // it is heard at `start` and padded, rather than dropped and taking
        // its syllable out of the caption with it. An end that is not a
        // number at all is not a word - padding it would put a caption on
        // screen that never comes off.
        let mut end = if word.end.is_finite() && word.end > start {
            word.end
        } else if word.end.is_finite() {
            start + MIN_WORD_PAD
        } else {
            continue;
        };

        // A punctuation-only token rides with the word before it, even a hair
        // outside the window: a token engine emits "." as its own token
        // starting exactly where the previous word ends, and a caption that
        // reads "cerah ." instead of "cerah." is a caption nobody wants.
        if !text.chars().any(is_alphanumeric) {
            if let Some(last) = out.last_mut() {
                last.text.push_str(text);
                let capped = match window_end {
                    Some(hi) => end.min(hi),
                    None => end,
                };
                last.end = round_millis(last.end.max(capped));
                continue;
            }
            // With nothing before it there is nothing to ride on, and a
            // caption that is only a full stop is not a caption.
            continue;
        }

        if let Some(hi) = window_end
            && start >= hi - TOUCH
        {
            continue;
        }
        if let Some(lo) = window_start {
            if end <= lo + TOUCH {
                continue;
            }
            start = start.max(lo);
        }
        if let Some(hi) = window_end {
            end = end.min(hi);
        }
        if end - start < MIN_WORD {
            end = start + MIN_WORD;
        }

        out.push(Word {
            text: text.to_owned(),
            start: round_millis(start),
            end: round_millis(end),
        });
    }

    out.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.end.total_cmp(&b.end)));
    out.truncate(MAX_WORDS);

    let mut touched = out.len() != words.len();
    for pair in out.windows(2) {
        if pair[1].start < pair[0].end {
            touched = true;
        }
    }

    // Two words may not share the same slice of the timeline, so the later
    // one starts where the earlier ended.
    for index in 1..out.len() {
        let previous_end = out[index - 1].end;
        if out[index].start < previous_end {
            out[index].start = round_millis(previous_end);
            if out[index].end <= out[index].start {
                out[index].end = round_millis(out[index].start + MIN_WORD_PAD);
            }
        }
    }

    // Padding, gluing or de-overlapping is an edit to what was heard, and
    // saying so is the whole reason the source is recorded.
    for (index, word) in words.iter().enumerate() {
        if let Some(cleaned) = out.get(index)
            && (cleaned.start != round_millis(word.start) || cleaned.end != round_millis(word.end))
        {
            touched = true;
            break;
        }
    }

    (out, touched)
}

/// The words of one cue that fall inside a kept range, moved onto the clock
/// of the timeline that range is kept in.
///
/// A word straddling the cut is clipped rather than dropped - it is heard on
/// both sides of the join, and losing it loses a syllable from the caption. A
/// word wholly inside a removed range goes with it.
fn clip_words(words: &[Word], from: f64, to: f64, removed_before: f64) -> Vec<Word> {
    let mut out = Vec::new();
    for word in words {
        if word.end <= from || word.start >= to {
            continue;
        }
        let start = word.start.max(from);
        let end = word.end.min(to);
        if end - start < 1e-6 {
            continue;
        }
        out.push(Word {
            text: word.text.clone(),
            start: round_millis(start - removed_before),
            end: round_millis(end - removed_before),
        });
    }
    out
}

/// Moves a cue list through a cut.
///
/// `kept` is the ranges that survive, in source order and in source seconds:
/// exactly what [`crate::sections::complement`] returns. What comes back is
/// the same captions on the clock of the timeline they were cut into -
/// shifted by everything removed ahead of them, split where a cut went
/// through one, joined again where the two halves of one sentence landed
/// back-to-back, and gone where nothing of them was kept.
///
/// The words travel with their cue and are clipped at the cut with it, so a
/// karaoke reveal on the far side of a trim still lights the word being
/// heard. That is the part that is easy to lose and expensive to notice
/// missing.
pub fn remap(cues: &[Cue], kept: &[(f64, f64)], min_piece: f64) -> Vec<Cue> {
    let mut pieces: Vec<Cue> = Vec::new();
    let mut removed_before = 0.0;
    let mut previous_end = 0.0;

    for &(start, end) in kept {
        removed_before += (start - previous_end).max(0.0);
        previous_end = end;

        for cue in cues {
            let from = cue.start.max(start);
            let to = cue.end.min(end);
            if to - from < min_piece {
                continue;
            }
            let words = clip_words(&cue.words, from, to, removed_before);
            pieces.push(Cue {
                start: round_millis(from - removed_before),
                end: round_millis(to - removed_before),
                text: cue.text.trim().to_owned(),
                // Words travel only when there are any: a cue that had none
                // must not gain an empty list that says it had some.
                word_source: if words.is_empty() {
                    None
                } else {
                    cue.word_source
                },
                words,
                id: cue.id.clone(),
                track: cue.track.clone(),
                style: cue.style.clone(),
            });
        }
    }

    pieces.sort_by(|a, b| {
        a.start
            .total_cmp(&b.start)
            .then(a.end.total_cmp(&b.end))
            .then(a.track.cmp(&b.track))
    });

    // Rejoin the fragments of one sentence a cut merely interrupted. The
    // words of both follow the cue, minus a word that landed on the join and
    // would otherwise be listed twice.
    let mut merged: Vec<Cue> = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let joins = match merged.last() {
            Some(last) => {
                last.text == piece.text
                    && last.track == piece.track
                    && (last.end - piece.start).abs() <= TOUCH
            }
            None => false,
        };
        if joins {
            let last = merged.last_mut().expect("just matched");
            last.end = piece.end;
            if !piece.words.is_empty() {
                let seen: Vec<(String, f64)> = last
                    .words
                    .iter()
                    .map(|word| (word.text.clone(), word.start))
                    .collect();
                last.words.extend(
                    piece
                        .words
                        .into_iter()
                        .filter(|word| !seen.contains(&(word.text.clone(), word.start))),
                );
                last.word_source = last.word_source.or(piece.word_source);
            }
        } else {
            merged.push(piece);
        }
    }
    merged
}

/// [`remap`] at the shortest fragment worth keeping.
pub fn remap_default(cues: &[Cue], kept: &[(f64, f64)]) -> Vec<Cue> {
    remap(cues, kept, MIN_PIECE)
}

/// How many words a cue list carries in all, which is what tells a panel
/// whether a word layout has anything to work with.
pub fn word_count(cues: &[Cue]) -> usize {
    cues.iter().map(|cue| cue.words.len()).sum()
}

/// Whether any cue in the list has real word timings.
pub fn has_words(cues: &[Cue]) -> bool {
    cues.iter().any(|cue| !cue.words.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str, start: f64, end: f64) -> Word {
        Word {
            text: text.to_owned(),
            start,
            end,
        }
    }

    fn cue(start: f64, end: f64, text: &str) -> Cue {
        Cue {
            start,
            end,
            text: text.to_owned(),
            ..Cue::default()
        }
    }

    fn cue_with(start: f64, end: f64, text: &str, words: Vec<Word>) -> Cue {
        Cue {
            start,
            end,
            text: text.to_owned(),
            words,
            word_source: Some(WordSource::Whisper),
            ..Cue::default()
        }
    }

    /// Cleaning words that were already clean leaves them alone.
    fn clean(words: &[Word]) -> Vec<Word> {
        clean_words(words, None, None).0
    }

    // ── clean_words ──────────────────────────────────────────────────────

    #[test]
    fn words_already_in_order_and_apart_come_back_unchanged() {
        let words = vec![
            word("selamat", 0.0, 0.4),
            word("pagi", 0.45, 0.8),
            word("semua", 0.9, 1.3),
        ];
        assert_eq!(clean(&words), words);
    }

    #[test]
    fn words_out_of_order_come_back_in_the_order_they_are_heard() {
        let words = vec![word("later", 2.0, 2.5), word("sooner", 0.5, 1.0)];
        assert_eq!(
            clean(&words),
            vec![word("sooner", 0.5, 1.0), word("later", 2.0, 2.5)]
        );
    }

    #[test]
    fn two_words_sharing_one_slice_of_the_timeline_are_pulled_apart() {
        // The invariant everything downstream asks of: "which word is being
        // heard now" has one answer only if the spans do not overlap.
        let words = vec![word("one", 0.0, 1.0), word("two", 0.5, 1.5)];
        let cleaned = clean(&words);
        assert_eq!(cleaned[1].start, cleaned[0].end);
        assert!(cleaned[1].end > cleaned[1].start);
    }

    #[test]
    fn a_word_given_no_length_is_padded_rather_than_dropped() {
        let cleaned = clean(&[word("ah", 1.0, 1.0)]);
        assert_eq!(cleaned.len(), 1, "the syllable stays in the caption");
        assert!(cleaned[0].end - cleaned[0].start >= MIN_WORD);
    }

    #[test]
    fn a_word_whose_end_is_before_its_start_is_padded_too() {
        let cleaned = clean(&[word("oh", 2.0, 1.0)]);
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].start, 2.0);
        assert!(cleaned[0].end > 2.0);
    }

    #[test]
    fn a_word_shorter_than_the_floor_is_held_to_it() {
        let cleaned = clean(&[word("a", 1.0, 1.005)]);
        assert!(cleaned[0].end - cleaned[0].start >= MIN_WORD, "{cleaned:?}");
    }

    #[test]
    fn an_empty_word_is_not_a_word() {
        assert_eq!(clean(&[word("", 1.0, 2.0)]), vec![]);
        assert_eq!(clean(&[word("   ", 1.0, 2.0)]), vec![]);
    }

    #[test]
    fn a_time_that_is_not_a_number_is_not_a_word() {
        let words = vec![
            word("good", 0.0, 0.5),
            Word {
                text: "nan".to_owned(),
                start: f64::NAN,
                end: 1.0,
            },
            Word {
                text: "inf".to_owned(),
                start: 1.0,
                end: f64::INFINITY,
            },
            word("also good", 1.5, 2.0),
        ];
        assert_eq!(
            clean(&words),
            vec![word("good", 0.0, 0.5), word("also good", 1.5, 2.0)]
        );
    }

    #[test]
    fn punctuation_rides_on_the_word_before_it() {
        // A token engine emits "." as its own token, starting exactly where
        // the previous word ends. A caption reading "cerah ." is wrong.
        let words = vec![word("cerah", 1.0, 1.4), word(".", 1.4, 1.45)];
        assert_eq!(clean(&words), vec![word("cerah.", 1.0, 1.45)]);
    }

    #[test]
    fn punctuation_in_any_script_rides_the_same_way() {
        for mark in ["。", "、", "！", "؟", "…", "」", "，"] {
            let words = vec![word("語", 1.0, 1.4), word(mark, 1.4, 1.45)];
            let cleaned = clean(&words);
            assert_eq!(cleaned.len(), 1, "{mark} should glue");
            assert!(
                cleaned[0].text.ends_with(mark),
                "{} in {}",
                mark,
                cleaned[0].text
            );
        }
    }

    #[test]
    fn a_full_stop_with_nothing_before_it_is_not_a_caption() {
        assert_eq!(clean(&[word(".", 1.0, 1.1)]), vec![]);
    }

    #[test]
    fn a_word_in_a_script_with_no_ascii_letters_is_still_a_word() {
        // is_alphanumeric is asked, not is_ascii_alphanumeric: if this were
        // the other question every word below would glue into one.
        let words = vec![
            word("こんにちは", 0.0, 0.8),
            word("世界", 0.9, 1.4),
            word("مرحبا", 1.5, 2.0),
            word("สวัสดี", 2.1, 2.6),
        ];
        assert_eq!(clean(&words), words);
    }

    #[test]
    fn a_window_clips_the_words_that_straddle_it_and_drops_the_ones_outside() {
        let words = vec![
            word("before", 0.0, 0.5),
            word("straddles", 0.9, 1.4),
            word("inside", 1.5, 1.8),
            word("after", 2.5, 3.0),
        ];
        let (cleaned, _) = clean_words(&words, Some(1.0), Some(2.0));
        assert_eq!(
            cleaned,
            vec![word("straddles", 1.0, 1.4), word("inside", 1.5, 1.8)]
        );
    }

    #[test]
    fn a_window_starting_after_every_word_leaves_none() {
        assert_eq!(
            clean_words(&[word("a", 0.0, 1.0)], Some(5.0), None).0,
            vec![]
        );
    }

    #[test]
    fn a_cue_holds_no_more_words_than_the_guard_allows() {
        let words: Vec<Word> = (0..(MAX_WORDS + 50))
            .map(|i| word("x", i as f64 * 0.1, i as f64 * 0.1 + 0.05))
            .collect();
        assert_eq!(clean(&words).len(), MAX_WORDS);
    }

    #[test]
    fn cleaning_says_when_it_changed_what_it_was_given() {
        let tidy = vec![word("one", 0.0, 0.5), word("two", 0.6, 1.0)];
        assert!(!clean_words(&tidy, None, None).1, "nothing to do");

        let overlapping = vec![word("one", 0.0, 1.0), word("two", 0.5, 1.5)];
        assert!(clean_words(&overlapping, None, None).1, "pulled apart");

        let punctuated = vec![word("cerah", 1.0, 1.4), word(".", 1.4, 1.45)];
        assert!(clean_words(&punctuated, None, None).1, "glued");
    }

    // ── remap ────────────────────────────────────────────────────────────

    #[test]
    fn with_nothing_cut_a_cue_list_comes_back_as_it_went_in() {
        let cues = vec![cue(1.0, 2.0, "one"), cue(3.0, 4.0, "two")];
        assert_eq!(remap(&cues, &[(0.0, 10.0)], MIN_PIECE), cues);
    }

    #[test]
    fn a_cue_after_a_cut_moves_earlier_by_exactly_what_was_removed() {
        let cues = vec![cue(5.0, 6.0, "after the gap")];
        // Two seconds removed ahead of it.
        let remapped = remap(&cues, &[(0.0, 3.0), (5.0, 10.0)], MIN_PIECE);
        assert_eq!(remapped, vec![cue(3.0, 4.0, "after the gap")]);
    }

    #[test]
    fn a_cue_wholly_inside_a_removed_range_disappears() {
        // This is the whole product: the caption goes, and the picture under
        // it goes with it. The filler sits inside the removed span and keeps
        // not one frame of it.
        let cues = vec![cue(1.2, 1.8, "um"), cue(4.0, 5.0, "the point")];
        let remapped = remap(&cues, &[(0.0, 1.0), (2.0, 10.0)], MIN_PIECE);
        // Only the second is gone. What is removed ahead of the survivor is
        // everything outside the kept ranges before it - the second at the
        // start, and the two in the middle - so four becomes three.
        assert_eq!(remapped, vec![cue(3.0, 4.0, "the point")]);
    }

    #[test]
    fn a_cue_crossing_a_cut_is_split_and_then_joined_when_it_is_one_sentence() {
        // The split is the mechanism; the rejoin is the reason for it. A cut
        // inside one sentence should not leave two captions both saying it,
        // so the halves land back to back and become one cue again.
        let cues = vec![cue(1.0, 5.0, "one two three four")];
        let remapped = remap(&cues, &[(0.0, 2.0), (4.0, 10.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 1, "{remapped:?}");
        assert_eq!(remapped[0].start, 1.0);
        assert_eq!(remapped[0].end, 3.0, "the two removed seconds are gone");
        assert_eq!(remapped[0].text, "one two three four");
    }

    #[test]
    fn a_cue_crossing_a_cut_stays_split_when_the_two_sides_differ() {
        // The rejoin is by text as well as by adjacency, so two different
        // sentences that a cut brings together are still two captions.
        let cues = vec![cue(0.0, 2.0, "first"), cue(4.0, 6.0, "second")];
        let remapped = remap(&cues, &[(0.0, 2.0), (4.0, 10.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 2, "{remapped:?}");
        assert_eq!(remapped[0].text, "first");
        assert_eq!(remapped[1].text, "second");
        assert_eq!(remapped[1].start, 2.0, "shifted by the two seconds removed");
    }

    #[test]
    fn the_two_halves_of_one_sentence_land_back_to_back_and_are_joined_again() {
        // A cut inside one sentence should not leave two captions saying it.
        let cues = vec![cue(1.0, 5.0, "one sentence")];
        let remapped = remap(&cues, &[(0.0, 3.0), (3.5, 10.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 1, "{remapped:?}");
        assert_eq!(remapped[0].text, "one sentence");
        assert_eq!(remapped[0].start, 1.0);
        assert_eq!(remapped[0].end, 4.5);
    }

    #[test]
    fn two_different_sentences_that_become_neighbours_stay_two_captions() {
        let cues = vec![cue(0.0, 2.0, "first"), cue(4.0, 6.0, "second")];
        let remapped = remap(&cues, &[(0.0, 2.0), (4.0, 6.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 2, "{remapped:?}");
        assert_eq!(remapped[0].text, "first");
        assert_eq!(remapped[1].text, "second");
    }

    #[test]
    fn a_fragment_too_short_to_read_goes_with_the_cut() {
        // 30 ms of a cue surviving a cut is a flash, not a caption.
        let cues = vec![cue(0.0, 2.03, "words")];
        let remapped = remap(&cues, &[(2.0, 10.0)], MIN_PIECE);
        assert_eq!(remapped, vec![]);
    }

    #[test]
    fn words_travel_with_their_cue_and_are_clipped_at_the_cut() {
        let cues = vec![cue_with(
            0.0,
            4.0,
            "one two three",
            vec![
                word("one", 0.0, 1.0),
                word("two", 1.0, 2.0),
                word("three", 3.0, 4.0),
            ],
        )];
        let remapped = remap(&cues, &[(0.0, 1.5), (3.0, 4.0)], MIN_PIECE);

        // One cue, joined across the cut, carrying every word that survives.
        // "two" is heard on both sides of the join, so it is clipped rather
        // than dropped: losing it would lose a syllable from the caption and
        // a beat from the karaoke timing.
        assert_eq!(remapped.len(), 1, "{remapped:?}");
        let words = &remapped[0].words;
        assert_eq!(
            words,
            &vec![
                word("one", 0.0, 1.0),
                word("two", 1.0, 1.5),
                word("three", 1.5, 2.5),
            ],
            "{words:?}"
        );
    }

    #[test]
    fn a_word_straddling_the_cut_is_clipped_and_not_lost() {
        // It is heard on both sides of the join; dropping it drops a
        // syllable out of the caption and a gap into the karaoke timing.
        let cues = vec![cue_with(
            0.0,
            2.0,
            "straddling",
            vec![word("straddling", 0.5, 1.5)],
        )];
        let remapped = remap(&cues, &[(0.0, 1.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 1);
        assert_eq!(remapped[0].words.len(), 1);
        assert_eq!(remapped[0].words[0].start, 0.5);
        assert_eq!(remapped[0].words[0].end, 1.0, "clipped to the cut");
    }

    #[test]
    fn a_word_inside_a_removed_range_goes_with_it() {
        let cues = vec![cue_with(
            0.0,
            6.0,
            "keep drop keep",
            vec![
                word("keep", 0.0, 1.0),
                word("drop", 2.0, 3.0),
                word("keep", 4.0, 5.0),
            ],
        )];
        let remapped = remap(&cues, &[(0.0, 1.0), (4.0, 6.0)], MIN_PIECE);
        let words: Vec<_> = remapped.iter().flat_map(|c| c.words.iter()).collect();
        assert!(words.iter().all(|w| w.text == "keep"), "{words:?}");
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn kept_ranges_that_only_touch_do_not_double_a_word_at_the_join() {
        // Two kept ranges with nothing between them still split a cue that
        // crosses the join, and the halves rejoin. A word sitting exactly on
        // the boundary would then arrive from both halves, which is what the
        // dedupe by word and start time is for.
        let cues = vec![cue_with(
            0.0,
            5.0,
            "one two three",
            vec![
                word("one", 0.0, 1.0),
                word("two", 2.0, 3.0),
                word("three", 4.0, 5.0),
            ],
        )];
        let remapped = remap(&cues, &[(0.0, 2.5), (2.5, 5.0)], MIN_PIECE);

        assert_eq!(remapped.len(), 1, "{remapped:?}");
        let words = &remapped[0].words;
        assert_eq!(
            words,
            &vec![
                word("one", 0.0, 1.0),
                // Clipped by the first range, then clipped by the second:
                // heard on both sides of a join that removes nothing.
                word("two", 2.0, 2.5),
                word("two", 2.5, 3.0),
                word("three", 4.0, 5.0),
            ],
            "{words:?}"
        );
        assert_eq!(remapped[0].start, 0.0);
        assert_eq!(
            remapped[0].end, 5.0,
            "nothing was removed, so nothing moves"
        );
    }

    #[test]
    fn a_cue_that_had_no_words_does_not_gain_an_empty_claim_to_have_them() {
        let cues = vec![cue(0.0, 2.0, "from a file")];
        let remapped = remap(&cues, &[(0.0, 1.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 1);
        assert!(remapped[0].words.is_empty());
        assert_eq!(remapped[0].word_source, None, "nothing was heard");
    }

    #[test]
    fn a_cue_keeps_its_lane_its_name_and_its_look_through_a_cut() {
        let mut original = cue(0.0, 4.0, "styled");
        original.id = Some("c7".to_owned());
        original.track = Some("T2".to_owned());
        original
            .style
            .insert("colour".to_owned(), Value::from("#ffee00"));

        let remapped = remap(&[original.clone()], &[(0.0, 2.0), (3.0, 4.0)], MIN_PIECE);
        assert_eq!(remapped.len(), 1, "{remapped:?}");
        assert_eq!(remapped[0].id, Some("c7".to_owned()));
        assert_eq!(remapped[0].track, Some("T2".to_owned()));
        assert_eq!(
            remapped[0].style.get("colour"),
            Some(&Value::from("#ffee00"))
        );
    }

    #[test]
    fn cues_on_two_lanes_are_never_joined_into_one() {
        // Stacked bilingual captions sit at the same time with different
        // words; even if the words matched, a join would collapse two lanes.
        let mut lower = cue(0.0, 4.0, "same");
        lower.track = Some("T1".to_owned());
        let mut upper = cue(0.0, 4.0, "same");
        upper.track = Some("T2".to_owned());

        let remapped = remap(&[lower, upper], &[(0.0, 2.0), (2.5, 4.0)], MIN_PIECE);
        let lanes: Vec<_> = remapped.iter().filter_map(|c| c.track.clone()).collect();
        assert_eq!(
            lanes.len(),
            4,
            "each lane keeps both of its pieces: {remapped:?}"
        );
    }

    #[test]
    fn a_cue_list_comes_back_in_the_order_it_is_read() {
        let cues = vec![
            cue(8.0, 9.0, "late"),
            cue(0.0, 1.0, "early"),
            cue(4.0, 5.0, "middle"),
        ];
        let remapped = remap(&cues, &[(0.0, 10.0)], MIN_PIECE);
        let starts: Vec<f64> = remapped.iter().map(|c| c.start).collect();
        assert_eq!(starts, vec![0.0, 4.0, 8.0]);
    }

    #[test]
    fn remapping_through_several_cuts_shifts_by_the_whole_of_what_was_taken() {
        let cues = vec![cue(9.0, 10.0, "at the end")];
        // Four seconds of the ten are kept, so six are gone, and all six sit
        // ahead of a cue that starts at nine: it lands at three.
        let kept = vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0), (9.0, 10.0)];
        let remapped = remap(&cues, &kept, MIN_PIECE);
        assert_eq!(remapped, vec![cue(3.0, 4.0, "at the end")]);

        let kept_total: f64 = kept.iter().map(|(start, end)| end - start).sum();
        assert_eq!(kept_total, 4.0, "four seconds survive");
        assert!(
            remapped[0].end <= kept_total + 1e-9,
            "and the survivor fits inside them: {remapped:?}"
        );
    }

    #[test]
    fn remapping_through_no_kept_ranges_leaves_no_captions() {
        let cues = vec![cue(0.0, 5.0, "all of it goes")];
        assert_eq!(remap(&cues, &[], MIN_PIECE), vec![]);
    }

    #[test]
    fn every_remapped_cue_ends_after_it_begins() {
        let cues = vec![
            cue(0.0, 3.0, "one"),
            cue(2.5, 6.0, "two"),
            cue(7.0, 7.4, "three"),
        ];
        for kept in [
            vec![(0.0, 10.0)],
            vec![(1.0, 4.0), (5.0, 8.0)],
            vec![(0.5, 2.0), (3.0, 3.5), (6.0, 9.0)],
        ] {
            for cue in remap(&cues, &kept, MIN_PIECE) {
                assert!(cue.end > cue.start, "{cue:?} from {kept:?}");
                assert!(cue.start >= 0.0, "{cue:?}");
                for word in &cue.words {
                    assert!(word.end >= word.start, "{word:?}");
                }
            }
        }
    }

    // ── the two together ─────────────────────────────────────────────────

    #[test]
    fn a_detected_trim_and_a_remapped_cue_list_agree_on_the_same_clock() {
        // The point of the pair: the ranges the picture is cut by and the
        // times the captions are moved by have to be one edit, or the words
        // drift away from the mouth that said them.
        use crate::sections::{MIN_KEEP, complement, silence_spans};

        let log = "silence_start: 2.0\nsilence_end: 4.0 | silence_duration: 2.0\n\
                   silence_start: 7.0\nsilence_end: 8.0 | silence_duration: 1.0\n";
        let silences = silence_spans(log, 10.0);
        let kept = complement(&silences, 10.0, MIN_KEEP);
        assert_eq!(kept, vec![(0.0, 2.0), (4.0, 7.0), (8.0, 10.0)]);

        let cues = vec![
            cue_with(0.0, 2.0, "before the pause", vec![word("before", 0.0, 1.0)]),
            cue_with(2.5, 3.5, "inside the silence", vec![word("gone", 2.5, 3.5)]),
            cue_with(
                5.0,
                9.0,
                "spans the second pause",
                vec![word("spans", 5.0, 6.0)],
            ),
        ];
        let remapped = remap(&cues, &kept, MIN_PIECE);

        // The cue inside the silence is gone; the others are on the new
        // clock, and the one that crossed the second cut lost two seconds.
        assert!(
            !remapped.iter().any(|c| c.text == "inside the silence"),
            "{remapped:?}"
        );
        let spanning = remapped.iter().find(|c| c.text == "spans the second pause");
        assert!(spanning.is_some(), "{remapped:?}");
        let spanning = spanning.unwrap();
        assert_eq!(spanning.start, 3.0, "5.0 in the source, 2.0 removed before");
        assert!(
            remapped.iter().all(|c| c.start >= 0.0 && c.end <= 7.0),
            "{remapped:?}"
        );

        // And the total is what the kept ranges say it is.
        let kept_total: f64 = kept.iter().map(|(s, e)| e - s).sum();
        assert!((kept_total - 7.0).abs() < 1e-9, "{kept_total}");
        assert!(
            remapped.iter().all(|c| c.end <= kept_total + 1e-9),
            "{remapped:?}"
        );
    }

    // ── serde ────────────────────────────────────────────────────────────

    #[test]
    fn a_cue_round_trips_through_the_file_it_is_saved_in() {
        let original = cue_with(
            1.5,
            2.25,
            "dua perkataan",
            vec![word("dua", 1.5, 1.8), word("perkataan", 1.85, 2.25)],
        );
        let json = serde_json::to_string(&original).expect("serialises");
        let read: Cue = serde_json::from_str(&json).expect("deserialises");
        assert_eq!(read, original);
    }

    #[test]
    fn a_cue_with_no_words_reads_as_one_and_writes_as_one() {
        // The shape that arrives from an SRT, and the shape a document
        // written before words existed holds: neither may fail to load.
        let json = r#"{"start":1.0,"end":2.0,"text":"plain"}"#;
        let read: Cue = serde_json::from_str(json).expect("a cue with no words field");
        assert_eq!(read.words, vec![]);
        assert_eq!(read.word_source, None);
        assert!(read.id.is_none() && read.track.is_none());
        assert!(read.style.is_empty());

        let written = serde_json::to_string(&read).expect("serialises");
        assert!(!written.contains("words"), "no empty claim: {written}");
        assert!(!written.contains("wordSource"), "{written}");
        assert!(!written.contains("style"), "{written}");
    }

    #[test]
    fn a_word_writes_under_the_short_keys_a_long_video_needs() {
        // A ten-minute video carries a few thousand of these into a document
        // and back out, so the spelling is three letters and not three words.
        let written = serde_json::to_string(&word("pagi", 0.45, 0.8)).expect("serialises");
        assert_eq!(written, r#"{"w":"pagi","s":0.45,"e":0.8}"#);
    }

    #[test]
    fn an_unknown_word_source_is_not_invented() {
        // The document spells its keys in camelCase, like every other file
        // the engine writes, so a source nobody defined is refused rather
        // than quietly becoming "no idea".
        let json = r#"{"start":0,"end":1,"text":"x","wordSource":"telepathy"}"#;
        assert!(
            serde_json::from_str::<Cue>(json).is_err(),
            "an unknown source is refused"
        );
        for known in ["whisper", "whispercpp", "estimated", "edited"] {
            let json = format!(r#"{{"start":0,"end":1,"text":"x","wordSource":"{known}"}}"#);
            let read: Cue = serde_json::from_str(&json).expect(known);
            assert!(read.word_source.is_some(), "{known}");
        }
    }

    // ── conversions and counts ───────────────────────────────────────────

    #[test]
    fn a_cue_from_a_sidecar_file_has_words_and_no_timings_for_them() {
        let from_file = crate::subtitle::Cue {
            start: 1.0,
            end: 2.0,
            text: "imported".to_owned(),
        };
        let converted: Cue = from_file.clone().into();
        assert_eq!(converted.start, 1.0);
        assert_eq!(converted.end, 2.0);
        assert_eq!(converted.text, "imported");
        assert!(converted.words.is_empty());
        assert_eq!(converted.word_source, None);

        let back: crate::subtitle::Cue = converted.into();
        assert_eq!(back, from_file);
    }

    #[test]
    fn going_out_to_a_sidecar_leaves_the_word_timings_behind() {
        // Neither SRT nor VTT has a place for them, so a cue that goes out
        // and comes back has lost them - which is why the document keeps them.
        let rich = cue_with(
            0.0,
            1.0,
            "two words",
            vec![word("two", 0.0, 0.4), word("words", 0.5, 1.0)],
        );
        let thin: crate::subtitle::Cue = rich.into();
        let back: Cue = thin.into();
        assert!(back.words.is_empty(), "the times did not survive");
        assert_eq!(back.text, "two words", "the words did");
    }

    #[test]
    fn a_panel_can_ask_whether_there_is_anything_to_make_a_word_layout_from() {
        assert!(!has_words(&[]));
        assert!(!has_words(&[cue(0.0, 1.0, "no words")]));
        assert!(has_words(&[cue_with(
            0.0,
            1.0,
            "one",
            vec![word("one", 0.0, 1.0)]
        )]));
        assert_eq!(word_count(&[cue(0.0, 1.0, "none")]), 0);
        assert_eq!(
            word_count(&[
                cue_with(
                    0.0,
                    1.0,
                    "a b",
                    vec![word("a", 0.0, 0.4), word("b", 0.5, 1.0)]
                ),
                cue_with(1.0, 2.0, "c", vec![word("c", 1.0, 2.0)]),
            ]),
            3
        );
    }

    #[test]
    fn the_default_remap_uses_the_shortest_fragment_worth_keeping() {
        let cues = vec![cue(0.0, 2.0, "words")];
        assert_eq!(
            remap_default(&cues, &[(0.0, 10.0)]),
            remap(&cues, &[(0.0, 10.0)], MIN_PIECE)
        );
    }
}
