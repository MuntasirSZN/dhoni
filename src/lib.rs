//! **dhoni** converts Bengali text into a phonetic Roman counterpart.
//!
//! It supports multiple transliteration conventions via [`Mode`]. The output
//! is ASCII lowercase and aims to be compatible with common phonetic schemes
//! (Avro, Khipro) rather than a lossless round-trip.
//!
//! # Examples
//! ```rust
//! use dhoni::{convert_to_phonetic, convert_to_phonetic_variants, Mode};
//!
//! assert_eq!(convert_to_phonetic("আমি", Mode::Avro), "ami");
//! assert_eq!(convert_to_phonetic("চাঁদ", Mode::Khipro), "ca/d");
//! ```

mod avro;
mod khipro;

/// Transliteration convention selector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Avro Phonetic-compatible spelling.
    Avro,
    /// Khipro layout spelling (see khipro-m17n): candrabindu `ঁ` → `/`,
    /// visarga-like `ৎ` → `t/`, and virama joins are preserved as-is.
    Khipro,
}

const NUKTA: char = '\u{09BC}';

/// A consonant that can carry an implicit vowel.
fn is_bengali_consonant(c: char) -> bool {
    matches!(c, '\u{0995}'..='\u{09B9}' | 'ড়' | 'ঢ়' | 'য়')
}

/// Precomposed nukta letters are composition-excluded, so text may carry
/// `ড` + nukta instead of `ড়`. Fold that pair onto its phonetic spelling.
fn nukta_letter(base: char) -> Option<&'static str> {
    match base {
        'ড' | 'ঢ' => Some("r"),
        'য' => Some("y"),
        _ => None,
    }
}

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

/// The shared Bengali letter inventory.
fn base_letter_map(letter: char) -> Option<&'static str> {
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

/// The knobs a transliteration scheme can tune
pub(crate) trait Scheme {
    fn spelling(&self, letter: char) -> Option<&'static str>;

    /// Insert the implicit `o` an Avro typist writes between consonants.
    fn insert_inherent_o(&self, _chars: &[char], _next: usize) -> bool {
        false
    }

    /// Avro's alternate style treats the `্য` glide as silent.
    fn suppress_z_fola(&self) -> bool {
        false
    }
}

fn emit_inherent_o(out: &mut String, chars: &[char], next: usize, scheme: &impl Scheme) {
    if scheme.insert_inherent_o(chars, next) {
        out.push('o');
    }
}

/// Shared conversion engine
pub(crate) fn convert_with(input: &str, scheme: &impl Scheme) -> String {
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
                    emit_inherent_o(&mut converted, &chars, index, scheme);
                    continue;
                }
            }
        }

        if chars.get(index + 1) == Some(&NUKTA) {
            if let Some(out) = nukta_letter(current) {
                converted.push_str(out);
                index += 2;
                emit_inherent_o(&mut converted, &chars, index, scheme);
                continue;
            }
        }

        if current == '্' {
            match chars.get(index + 1) {
                Some('ব') => {
                    converted.push('w');
                    index += 2;
                    emit_inherent_o(&mut converted, &chars, index, scheme);
                    continue;
                }
                Some('য') => {
                    if !scheme.suppress_z_fola() {
                        converted.push('y');
                    }
                    index += 2;
                    emit_inherent_o(&mut converted, &chars, index, scheme);
                    continue;
                }
                // Both schemes drop the virama mark and let the following
                // consonant flow through; Khipro's join behaviour is simply
                // the absence of an implicit-vowel insertion.
                _ => {
                    index += 1;
                    continue;
                }
            }
        }

        let spelling = scheme.spelling(current).unwrap_or("");
        converted.push_str(spelling);
        if is_bengali_consonant(current) {
            emit_inherent_o(&mut converted, &chars, index + 1, scheme);
        }

        if spelling.is_empty() && (current.is_ascii_whitespace() || current.is_ascii_punctuation())
        {
            // Includes the Khipro `/` layout marker.
            converted.push(current);
        }

        index += 1;
    }

    converted
}

/// Converts input text using the given transliteration [`Mode`].
///
/// This is the canonical spelling: no optional implicit vowels are inserted.
pub fn convert_to_phonetic(input: &str, mode: Mode) -> String {
    match mode {
        Mode::Avro => avro::convert(input),
        Mode::Khipro => khipro::convert(input),
    }
}

/// Returns plausible phonetic forms for one Bangla word.
///
/// Avro mode returns the canonical spelling plus alternates that cover common
/// Avro-style typings: the implicit `o` many typists insert between
/// consonants (`kortobbo` for কর্তব্য) and aspirated/silent-glide spellings
/// (`chino` for চিনো, `kortobbo` for কর্তব্য). Khipro mode additionally
/// returns a trailing-`/` variant for words containing a virama (`্`),
/// because Khipro typists can suppress a virama with `/` (e.g. `ks/` → কস
/// vs `ks` → ক্স).
pub fn convert_to_phonetic_variants(input: &str, mode: Mode) -> Vec<String> {
    let mut forms = match mode {
        Mode::Avro => avro::variants(input),
        Mode::Khipro => khipro::variants(input),
    };
    forms.sort();
    forms.dedup();
    forms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(convert_to_phonetic("০১২৩৪৫৬৭৮৯", Mode::Avro), "0123456789");
    }

    #[test]
    fn word() {
        assert_eq!(convert_to_phonetic("আমি", Mode::Avro), "ami");
        assert_eq!(convert_to_phonetic("ঋতু", Mode::Avro), "ritu");
        assert_eq!(convert_to_phonetic("ওষুধ", Mode::Avro), "oxudh");
        assert_eq!(convert_to_phonetic("সংখ্যা", Mode::Avro), "sngkhya");
        assert_eq!(convert_to_phonetic("ক্ষুধা", Mode::Avro), "kkhudha");
        assert_eq!(convert_to_phonetic("বিশ্ব", Mode::Avro), "bisw");
        assert_eq!(convert_to_phonetic("পদ্ম", Mode::Avro), "pdm");
        assert_eq!(convert_to_phonetic("জিতেন্দ্র", Mode::Avro), "jitendr");
        assert_eq!(convert_to_phonetic("বিদ্বান", Mode::Avro), "bidwan");
        assert_eq!(convert_to_phonetic("চাঁদ", Mode::Avro), "cad");
    }

    #[test]
    fn sentence() {
        assert_eq!(
            convert_to_phonetic(
                "আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ",
                Mode::Avro
            ),
            "amader bhalobasa hye gel ghas, kheye gel gru ar diye gel bas"
        );
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
            assert_eq!(
                convert_to_phonetic(bangla, Mode::Avro),
                roman,
                "conjunct {bangla}"
            );
        }
    }

    #[test]
    fn canonical_forms_are_ascii_lowercase() {
        let letters = "অআইঈউঊঋএঐওঔকখগঘঙচছজঝঞটঠডঢণতথদধানপফবভমযরলশষসহড়ঢ়য়ৎংঃঁ";
        for mode in [Mode::Avro, Mode::Khipro] {
            for letter in letters.chars() {
                for form in convert_to_phonetic_variants(&letter.to_string(), mode) {
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
}
