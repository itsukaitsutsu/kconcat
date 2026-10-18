//! E2: the Indonesian filler-word lists and the matcher that finds them in
//! a caption's heard words.
//!
//! Owner decision A7 (16 Oct 2026): two tiers, both opt-in - nothing is
//! removed unless its tier is ticked, because a weak-tier word such as
//! "terus" can be meaningful. This file is the one fact for the lists;
//! `03-manuals/C-filler-words.md` records them for the owner.

use crate::captionmap::Word;

/// The strong tier: sounds that carry no meaning of their own.
pub const FILLER_STRONG: &[&str] = &["eee", "anu", "ehm"];

/// The weak tier: real words that are often - not always - filler. The
/// owner ticks this tier knowingly.
pub const FILLER_WEAK: &[&str] = &["kayak", "gitu", "terus", "apa ya"];

/// A word's filler form: lowercased, with edge punctuation off. Matching is
/// whole-word, so "begitu" is never "gitu".
fn normalized(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// The indices of the words the ticked tiers' fillers cover, ascending. A
/// multi-word entry ("apa ya") covers every word of a consecutive run, and
/// only a consecutive run: "apa" alone and "ya" alone are not filler.
pub fn filler_indices(words: &[Word], strong: bool, weak: bool) -> Vec<usize> {
    let mut entries: Vec<Vec<String>> = Vec::new();
    for (ticked, list) in [(strong, FILLER_STRONG), (weak, FILLER_WEAK)] {
        if ticked {
            entries.extend(
                list.iter()
                    .map(|entry| entry.split_whitespace().map(normalized).collect()),
            );
        }
    }
    // Longer entries first, so the "apa ya" run wins over any single-word
    // entry sharing its first word.
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.len()));
    let heard: Vec<String> = words.iter().map(|word| normalized(&word.text)).collect();
    let mut hits = Vec::new();
    let mut at = 0;
    while at < heard.len() {
        match entries.iter().find(|entry| {
            entry.len() <= heard.len() - at && heard[at..at + entry.len()] == entry[..]
        }) {
            Some(entry) => {
                hits.extend(at..at + entry.len());
                at += entry.len();
            }
            None => at += 1,
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str) -> Word {
        Word {
            text: text.to_owned(),
            start: 0.0,
            end: 0.1,
        }
    }

    fn words(texts: &[&str]) -> Vec<Word> {
        texts.iter().map(|t| word(t)).collect()
    }

    #[test]
    fn the_strong_tier_matches_only_its_own_words() {
        let w = words(&["eee", "kita", "pergi"]);
        assert_eq!(filler_indices(&w, true, false), vec![0]);
        // "anu" is strong; "terus" is weak and stays when weak is unticked.
        let w = words(&["anu", "terus", "selesai"]);
        assert_eq!(filler_indices(&w, true, false), vec![0]);
    }

    #[test]
    fn the_weak_tier_matches_only_when_ticked() {
        let w = words(&["terus", "selesai"]);
        assert_eq!(filler_indices(&w, false, true), vec![0]);
        assert_eq!(filler_indices(&w, false, false), Vec::<usize>::new());
    }

    #[test]
    fn case_and_edge_punctuation_do_not_hide_a_filler() {
        let w = words(&["Eee,", "kita"]);
        assert_eq!(filler_indices(&w, true, false), vec![0]);
        let w = words(&["Terus."]);
        assert_eq!(filler_indices(&w, false, true), vec![0]);
    }

    #[test]
    fn matching_is_whole_word() {
        // The negative case the roadmap asks for: a meaningful lookalike.
        let w = words(&["begitu", "indah"]);
        assert_eq!(filler_indices(&w, true, true), Vec::<usize>::new());
        let w = words(&["kayaknya", "bagus"]);
        assert_eq!(filler_indices(&w, true, true), Vec::<usize>::new());
    }

    #[test]
    fn a_multi_word_filler_needs_its_consecutive_run() {
        let w = words(&["apa", "ya", "bagus"]);
        assert_eq!(filler_indices(&w, false, true), vec![0, 1]);
        // Broken up, neither half is filler on its own.
        let w = words(&["apa", "bagus", "ya"]);
        assert_eq!(filler_indices(&w, true, true), Vec::<usize>::new());
    }
}
