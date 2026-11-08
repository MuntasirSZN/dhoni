//! Khipro layout transliteration.

use crate::{base_letter_map, convert_with, Scheme};

struct Khipro;

/// Khipro differs from the shared baseline only in these two letters.
fn letter_map(letter: char) -> Option<&'static str> {
    match letter {
        'ৎ' => Some("t/"),
        'ঁ' => Some("/"),
        _ => base_letter_map(letter),
    }
}

impl Scheme for Khipro {
    fn spelling(&self, letter: char) -> Option<&'static str> {
        letter_map(letter)
    }
}

pub(crate) fn convert(input: &str) -> String {
    convert_with(input, &Khipro)
}

pub(crate) fn variants(input: &str) -> Vec<String> {
    let mut forms: Vec<String> = Vec::new();
    let primary = convert(input);
    if !primary.is_empty() {
        forms.push(primary);
    }

    // Typists can suppress a virama with `/`, so both spellings are plausible.
    if input.contains('্') {
        forms.extend(
            forms
                .clone()
                .into_iter()
                .filter(|form| !form.ends_with('/'))
                .map(|form| format!("{form}/")),
        );
    }

    forms
}

#[cfg(test)]
mod tests {
    use crate::{convert_to_phonetic, convert_to_phonetic_variants, Mode};

    #[test]
    fn khipro_mode_uses_layout_markers() {
        assert_eq!(convert_to_phonetic("মাঁ", Mode::Khipro), "ma/");
        assert_eq!(convert_to_phonetic("ৎ", Mode::Khipro), "t/");
        assert_eq!(convert_to_phonetic("কঁ", Mode::Khipro), "k/");
        assert_eq!(convert_to_phonetic("ক্ষ", Mode::Khipro), "kkh");
        assert_eq!(convert_to_phonetic("মা", Mode::Khipro), "ma");
        assert_eq!(convert_to_phonetic("চাঁদ", Mode::Khipro), "ca/d");
    }

    #[test]
    fn khipro_virama_variants() {
        assert_eq!(
            convert_to_phonetic_variants("ক্স", Mode::Khipro),
            vec!["ks", "ks/"]
        );
        assert_eq!(convert_to_phonetic_variants("কস", Mode::Khipro), vec!["ks"]);
        assert_eq!(convert_to_phonetic_variants("মা", Mode::Khipro), vec!["ma"]);
    }

    #[test]
    fn tsv_has_modifier_rows() {
        let csv = include_str!("../data/khipro-m17n/khipro-testcases.tsv");
        let mut checked = 0;
        for line in csv.lines().skip(1) {
            let mut cols = line.split('\t');
            let input = cols.next().unwrap_or("");
            let output = cols.next().unwrap_or("");
            if output.contains('ঁ') && !input.ends_with("//") {
                checked += 1;
            }
        }
        assert!(
            checked >= 3,
            "{}",
            "expected candrabindu rows in TSV, got {checked}"
        );
    }

    #[test]
    fn khipro_tsv_conformance() {
        // Bangla -> Roman inverse checked against the khipro-m17n corpus:
        // exact = primary matches, variant = any emitted variant matches
        // (e.g. trailing '/' virama suppression), lossy = printed (layout
        // mechanics like conjunct typing/sh-s digraphs/f-suppressor that the
        // inverse intentionally does not encode).
        let csv = include_str!("../data/khipro-m17n/khipro-testcases.tsv");
        let is_bangla = |s: &str| {
            s.chars().all(|c| {
                ('\u{0980}'..='\u{09ff}').contains(&c)
                    || c == '\u{200c}'
                    || c == '\u{200d}'
                    || c == ' '
            })
        };
        let mut exact = 0;
        let mut variant = 0;
        let mut lossy: Vec<&str> = Vec::new();
        let mut non_roman = 0;
        let mut mapped = 0;
        for line in csv.lines().skip(1) {
            if line.is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let input = cols.next().unwrap_or("");
            let output = cols.next().unwrap_or("");
            if !is_bangla(output) {
                non_roman += 1;
                continue;
            }
            mapped += 1;
            let primary = convert_to_phonetic(output, Mode::Khipro);
            if primary == input {
                exact += 1;
            } else if convert_to_phonetic_variants(output, Mode::Khipro)
                .iter()
                .any(|f| f == input)
            {
                variant += 1;
            } else {
                lossy.push(input);
            }
        }

        println!(
            "conformance: exact={exact} variant={variant} mapped={mapped} lossy={lossy:?} non_roman={non_roman}"
        );
        assert!(
            exact + variant >= 20,
            "{}",
            "invertible core shrank: exact={exact} variant={variant} (lossy: {lossy:?})"
        );
        assert!(mapped > 50, "{}", "suspiciously few mapped rows: {mapped}");
    }
}
