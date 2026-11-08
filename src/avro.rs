//! Avro Phonetic-compatible transliteration.

use crate::{base_letter_map, convert_with, is_bengali_consonant, Scheme, NUKTA};

/// Spelling choices that produce alternate but plausible Roman forms.
#[derive(Clone, Copy)]
struct Style {
    /// Emit the implicit `o` that Avro typists insert between consonants
    /// (e.g. `kortobbo` for কর্তব্য).
    inherent_o: bool,
    /// Prefer aspirated/common alternate keys: `চ` → `ch`, and treat the
    /// `্য` glide as silent (`ব্য` → `b`).
    alternate: bool,
}

impl Scheme for Style {
    fn spelling(&self, letter: char) -> Option<&'static str> {
        if self.alternate && letter == 'চ' {
            Some("ch")
        } else {
            base_letter_map(letter)
        }
    }

    fn insert_inherent_o(&self, chars: &[char], next: usize) -> bool {
        self.inherent_o
            && chars
                .get(next)
                .is_some_and(|c| is_bengali_consonant(*c) && *c != NUKTA)
    }

    fn suppress_z_fola(&self) -> bool {
        self.alternate
    }
}

pub(crate) fn convert(input: &str) -> String {
    convert_with(
        input,
        &Style {
            inherent_o: false,
            alternate: false,
        },
    )
}

pub(crate) fn variants(input: &str) -> Vec<String> {
    const STYLES: [Style; 4] = [
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

    STYLES
        .iter()
        .map(|style| convert_with(input, style))
        .filter(|form| !form.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{convert_to_phonetic, convert_to_phonetic_variants, Mode};

    #[test]
    fn avro_mode() {
        assert_eq!(convert_to_phonetic("মাঁ", Mode::Avro), "ma");
        assert_eq!(convert_to_phonetic("ৎ", Mode::Avro), "t");
        assert_eq!(convert_to_phonetic_variants("ক্স", Mode::Avro), vec!["ks"]);
    }

    #[test]
    fn avro_variant_forms_cover_common_typings() {
        let forms = convert_to_phonetic_variants("চিনো", Mode::Avro);
        assert!(forms.contains(&"chino".to_string()), "{:?}", forms);
        assert!(forms.contains(&"cino".to_string()), "{:?}", forms);

        let forms = convert_to_phonetic_variants("কিংকর্তব্যবিমূঢ়", Mode::Avro);
        assert!(
            forms.contains(&"kingkortobobimur".to_string()),
            "{:?}",
            forms
        );

        assert_eq!(
            convert_to_phonetic_variants("পৃথিবীর", Mode::Avro),
            vec!["prithibir"]
        );
    }
}
