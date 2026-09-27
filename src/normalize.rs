//! Normalization for matching. Treats full-width and half-width, hiragana and katakana,
//! and upper and lower case as the same.

pub fn normalize(s: &str) -> String {
    s.chars()
        .map(hiragana_to_katakana)
        .collect::<String>()
        .to_lowercase()
}

/// Shifts ぁ(U+3041)..ゖ(U+3096) to ァ(U+30A1)..ヶ(U+30F6).
fn hiragana_to_katakana(c: char) -> char {
    match c {
        '\u{3041}'..='\u{3096}' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    mod normalize {
        use crate::normalize::normalize;

        #[test]
        fn lowercases_ascii_letters() {
            assert_eq!(normalize("CinderellaGirls"), "cinderellagirls");
        }

        #[test]
        fn turns_hiragana_into_katakana() {
            assert_eq!(normalize("あいう！アイウ"), "アイウ！アイウ");
        }

        #[test]
        #[ignore = "todo"]
        fn turns_full_width_ascii_into_half_width() {}

        #[test]
        #[ignore = "todo"]
        fn turns_half_width_katakana_into_full_width() {}
    }
}
