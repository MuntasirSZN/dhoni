//! Khipro layout transliteration.
//!
//! This module provides transliteration according to the Khipro layout convention.

use crate::utils::normalize_nukta;

/// Consonant conjunct special cases: (first, second after virama) → spelling.
const SPECIAL_CONJUNCTS: &[(char, char, &str)] = &[
    ('ক', 'ষ', "kkh"), // ক্ষ
];

fn special_conjunct(first: char, second: char) -> Option<&'static str> {
    for &(a, b, out) in SPECIAL_CONJUNCTS {
        if a == first && b == second {
            return Some(out);
        }
    }
    None
}

fn letter_map(letter: char) -> Option<&'static str> {
    match letter {
        // Numbers
        '০' => Some("0"),
        '১' => Some("1"),
        '২' => Some("2"),
        '৩' => Some("3"),
        '৪' => Some("4"),
        '৫' => Some("5"),
        '৬' => Some("6"),
        '৭' => Some("7"),
        '৮' => Some("8"),
        '৯' => Some("9"),
        // Vowels
        'অ' => Some("o"),
        'আ' => Some("a"),
        'ই' => Some("i"),
        'ঈ' => Some("i"),
        'উ' => Some("u"),
        'ঊ' => Some("u"),
        'ঋ' => Some("ri"),
        'এ' => Some("e"),
        'ঐ' => Some("oi"),
        'ও' => Some("o"),
        'ঔ' => Some("ou"),
        // Kars
        'া' => Some("a"),
        'ি' => Some("i"),
        'ী' => Some("i"),
        'ু' => Some("u"),
        'ূ' => Some("u"),
        'ৃ' => Some("ri"),
        'ে' => Some("e"),
        'ৈ' => Some("oi"),
        'ো' => Some("o"),
        'ৌ' => Some("ou"),
        // Consonants
        'ক' => Some("k"),
        'খ' => Some("kh"),
        'গ' => Some("g"),
        'ঘ' => Some("gh"),
        'ঙ' => Some("ng"),
        'চ' => Some("c"),
        'ছ' => Some("ch"),
        'জ' => Some("j"),
        'ঝ' => Some("jh"),
        'ঞ' => Some("ng"),
        'ট' => Some("t"),
        'ঠ' => Some("th"),
        'ড' => Some("d"),
        'ঢ' => Some("dh"),
        'ণ' => Some("n"),
        'ত' => Some("t"),
        'থ' => Some("th"),
        'দ' => Some("d"),
        'ধ' => Some("dh"),
        'ন' => Some("n"),
        'প' => Some("p"),
        'ফ' => Some("f"),
        'ব' => Some("b"),
        'ভ' => Some("bh"),
        'ম' => Some("m"),
        'য' => Some("z"),
        'র' => Some("r"),
        'ল' => Some("l"),
        'শ' => Some("s"),
        'ষ' => Some("x"),
        'স' => Some("s"),
        'হ' => Some("h"),
        'ড়' => Some("r"),
        'ঢ়' => Some("r"),
        'য়' => Some("y"),
        'ৎ' => Some("t/"),
        'ং' => Some("ng"),
        'ঃ' => Some(""),
        'ঁ' => Some("/"),
        _ => None,
    }
}

fn convert_with(input: &str) -> String {
    let mut converted = String::with_capacity(input.len());
    let chars = normalize_nukta(input);

    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];

        if chars.get(index + 1) == Some(&'্') {
            if let Some(after) = chars.get(index + 2) {
                if let Some(out) = special_conjunct(current, *after) {
                    converted.push_str(out);
                    index += 3;
                    continue;
                }
            }
        }

        if current == '্' {
            match chars.get(index + 1) {
                Some('ব') => {
                    converted.push('w');
                    index += 2;
                    continue;
                }
                Some('য') => {
                    converted.push('y');
                    index += 2;
                    continue;
                }
                _ => {
                    index += 1;
                    continue;
                }
            }
        }

        let spelling = letter_map(current).unwrap_or("");
        converted.push_str(spelling);
        if spelling.is_empty() && (current.is_ascii_whitespace() || current.is_ascii_punctuation())
        {
            converted.push(current);
        }

        index += 1;
    }

    converted
}

/// Converts input text into its Khipro phonetic counterpart.
pub fn convert(input: &str) -> String {
    convert_with(input)
}

/// Returns plausible phonetic forms for one Bangla word using Khipro styles.
pub fn convert_variants(input: &str) -> Vec<String> {
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

    forms.sort();
    forms.dedup();
    forms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(convert("০১২৩৪৫৬৭৮৯"), "0123456789");
    }

    #[test]
    fn word() {
        assert_eq!(convert("আমি"), "ami");
        assert_eq!(convert("ঋতু"), "ritu");
        assert_eq!(convert("ওষুধ"), "oxudh");
        assert_eq!(convert("সংখ্যা"), "sngkhya");
        assert_eq!(convert("ক্ষুধা"), "kkhudha");
        assert_eq!(convert("বিশ্ব"), "bisw");
        assert_eq!(convert("পদ্ম"), "pdm");
        assert_eq!(convert("জিতেন্দ্র"), "jitendr");
        assert_eq!(convert("বিদ্বান"), "bidwan");
        assert_eq!(convert("চাঁদ"), "ca/d");
    }

    #[test]
    fn decomposed_nukta_matches_precomposed() {
        // ড/ঢ/য + nukta normalize to ড়/ঢ়/য় before spelling.
        let pairs = [
            ("\u{09A1}\u{09BC}", "\u{09DC}"), // ড + nukta, ড়
            ("\u{09A2}\u{09BC}", "\u{09DD}"), // ঢ + nukta, ঢ়
            ("\u{09AF}\u{09BC}", "\u{09DF}"), // য + nukta, য়
        ];
        for (decomposed, precomposed) in pairs {
            assert_eq!(convert(decomposed), convert(precomposed), "{decomposed:?}");
        }
        assert_eq!(convert("\u{09A1}\u{09BC}"), "r");
        assert_eq!(convert("\u{09A2}\u{09BC}"), "r");
        assert_eq!(convert("\u{09AF}\u{09BC}"), "y");
    }

    #[test]
    fn sentence() {
        assert_eq!(
            convert("আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ"),
            "amader bhalobasa hye gel ghas, kheye gel gru ar diye gel ba/s"
        );
    }

    #[test]
    fn khipro_mode() {
        assert_eq!(convert("মাঁ"), "ma/");
        assert_eq!(convert("ৎ"), "t/");
        assert_eq!(convert("কঁ"), "k/");
        assert_eq!(convert("ক্ষ"), "kkh");
        assert_eq!(convert("মা"), "ma");
        assert_eq!(convert("চাঁদ"), "ca/d");
    }

    #[test]
    fn khipro_virama_variants() {
        assert_eq!(convert_variants("ক্স"), vec!["ks", "ks/"]);
        assert_eq!(convert_variants("কস"), vec!["ks"]);
        assert_eq!(convert_variants("মা"), vec!["ma"]);
    }

    #[test]
    fn conjuncts_are_concatenated_letters() {
        let cases = [
            ("স্ত", "st"),
            ("ক্ত", "kt"),
            ("ঞ্জ", "ngj"),
            ("ম্প", "mp"),
            ("ল্ল", "ll"),
            ("স্য", "sy"),
            ("দ্ধ", "ddh"),
            ("ক্ষ", "kkh"),
            ("ত্র", "tr"),
        ];
        for (bangla, roman) in cases {
            assert_eq!(convert(bangla), roman, "conjunct {bangla}");
        }
    }

    #[test]
    fn canonical_forms_are_ascii_lowercase() {
        let letters = "অআইঈউঊঋএঐওঔকখগঘঙচছজঝঞটঠডঢণতথদধানপফবভমযরলশষসহড়ঢ়য়ৎংঃঁ";
        for letter in letters.chars() {
            for form in convert_variants(&letter.to_string()) {
                assert!(
                    form.bytes()
                        .all(|b| b.is_ascii_lowercase() || b == b'/' || b.is_ascii_digit()),
                    "{}",
                    "non-canonical form {form:?} for {letter:?}"
                );
            }
        }
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
            let primary = convert(output);
            if primary == input {
                exact += 1;
            } else if convert_variants(output).iter().any(|f| f == input) {
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
