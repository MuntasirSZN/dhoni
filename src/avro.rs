//! Avro Phonetic-compatible transliteration.
//!
//! This module provides transliteration according to the Avro Phonetic convention.

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

/// A consonant that can carry an implicit vowel.
fn is_bengali_consonant(c: char) -> bool {
    matches!(c, '\u{0995}'..='\u{09B9}' | 'ড়' | 'ঢ়' | 'য়')
}

const NUKTA: char = '\u{09BC}';

/// Precomposed nukta letters are composition-excluded, so text may carry
/// `ড` + nukta instead of `ড়`. Fold that pair onto its phonetic spelling.
fn nukta_letter(base: char) -> Option<&'static str> {
    match base {
        'ড' | 'ঢ' => Some("r"),
        'য' => Some("y"),
        _ => None,
    }
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
        'ৎ' => Some("t"),
        'ং' => Some("ng"),
        'ঃ' => Some(""),
        'ঁ' => Some(""),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Style {
    inherent_o: bool,
    alternate: bool,
}

fn push_inherent_o(out: &mut String, chars: &[char], next: usize, style: Style) {
    if style.inherent_o
        && chars
            .get(next)
            .is_some_and(|c| is_bengali_consonant(*c) && *c != NUKTA)
    {
        out.push('o');
    }
}

fn convert_with(input: &str, style: Style) -> String {
    let mut converted = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();

    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];

        if chars.get(index + 1) == Some(&'্') {
            if let Some(after) = chars.get(index + 2) {
                if let Some(out) = special_conjunct(current, *after) {
                    converted.push_str(out);
                    index += 3;
                    push_inherent_o(&mut converted, &chars, index, style);
                    continue;
                }
            }
        }

        if chars.get(index + 1) == Some(&NUKTA) {
            if let Some(out) = nukta_letter(current) {
                converted.push_str(out);
                index += 2;
                push_inherent_o(&mut converted, &chars, index, style);
                continue;
            }
        }

        if current == '্' {
            match chars.get(index + 1) {
                Some('ব') => {
                    converted.push('w');
                    index += 2;
                    push_inherent_o(&mut converted, &chars, index, style);
                    continue;
                }
                Some('য') => {
                    if !style.alternate {
                        converted.push('y');
                    }
                    index += 2;
                    push_inherent_o(&mut converted, &chars, index, style);
                    continue;
                }
                _ => {
                    index += 1;
                    continue;
                }
            }
        }

        let spelling = if style.alternate && current == 'চ' {
            "ch"
        } else {
            letter_map(current).unwrap_or("")
        };
        converted.push_str(spelling);
        if is_bengali_consonant(current) {
            push_inherent_o(&mut converted, &chars, index + 1, style);
        }

        if spelling.is_empty() && (current.is_ascii_whitespace() || current.is_ascii_punctuation())
        {
            converted.push(current);
        }

        index += 1;
    }

    converted
}

/// Converts input text into its Avro phonetic counterpart.
pub fn convert(input: &str) -> String {
    convert_with(
        input,
        Style {
            inherent_o: false,
            alternate: false,
        },
    )
}

/// Returns plausible phonetic forms for one Bangla word using Avro styles.
pub fn convert_variants(input: &str) -> Vec<String> {
    let styles = [
        Style {
            inherent_o: false,
            alternate: false,
        },
        Style {
            inherent_o: true,
            alternate: false,
        },
        Style {
            inherent_o: true,
            alternate: true,
        },
        Style {
            inherent_o: false,
            alternate: true,
        },
    ];

    let mut forms: Vec<String> = styles
        .iter()
        .map(|style| convert_with(input, *style))
        .filter(|form| !form.is_empty())
        .collect();

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
        assert_eq!(convert("চাঁদ"), "cad");
    }

    #[test]
    fn sentence() {
        assert_eq!(
            convert("আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ"),
            "amader bhalobasa hye gel ghas, kheye gel gru ar diye gel bas"
        );
    }

    #[test]
    fn avro_mode() {
        assert_eq!(convert("মাঁ"), "ma");
        assert_eq!(convert("ৎ"), "t");
        assert_eq!(convert_variants("ক্স"), vec!["ks"]);
    }

    #[test]
    fn avro_variant_forms_cover_common_typings() {
        let forms = convert_variants("চিনো");
        assert!(forms.contains(&"chino".to_string()));
        assert!(forms.contains(&"cino".to_string()));

        let forms = convert_variants("কিংকর্তব্যবিমূঢ়");
        assert!(forms.contains(&"kingkortobobimur".to_string()));

        assert_eq!(convert_variants("পৃথিবীর"), vec!["prithibir"]);
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
}
