//! Helpers shared by the transliteration modules.

/// Combining nukta sign.
pub(crate) const NUKTA: char = '\u{09BC}';

/// `ড়`, `ঢ়` and `য়` are composition-excluded, so text may spell them as a
/// base letter plus a nukta.
fn precomposed_nukta(base: char) -> Option<char> {
    match base {
        'ড' => Some('\u{09DC}'), // ড়
        'ঢ' => Some('\u{09DD}'), // ঢ়
        'য' => Some('\u{09DF}'), // য়
        _ => None,
    }
}

/// Normalizes decomposed nukta letters (`ড` + nukta) to their precomposed
/// form so the character tables alone decide the spelling.
pub(crate) fn normalize_nukta(input: &str) -> Vec<char> {
    let mut normalized = Vec::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(current) = chars.next() {
        if chars.peek() == Some(&NUKTA) {
            if let Some(precomposed) = precomposed_nukta(current) {
                normalized.push(precomposed);
                chars.next();
                continue;
            }
        }
        normalized.push(current);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_decomposed_nukta() {
        assert_eq!(normalize_nukta("\u{09A1}\u{09BC}"), vec!['\u{09DC}']);
        assert_eq!(normalize_nukta("\u{09A2}\u{09BC}"), vec!['\u{09DD}']);
        assert_eq!(normalize_nukta("\u{09AF}\u{09BC}"), vec!['\u{09DF}']);
    }

    #[test]
    fn leaves_other_text_alone() {
        assert_eq!(normalize_nukta("ক\u{09BC}"), vec!['ক', NUKTA]);
        assert_eq!(normalize_nukta("\u{09DC}"), vec!['\u{09DC}']);
        assert_eq!(normalize_nukta("আমি"), vec!['আ', 'ম', 'ি']);
    }
}
