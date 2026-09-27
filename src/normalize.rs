//! Normalization for matching. Treats full-width and half-width, hiragana and katakana,
//! and upper and lower case as the same.

#[cfg(test)]
mod tests {
    mod normalize {
        #[test]
        #[ignore = "todo"]
        fn lowercases_ascii_letters() {}

        #[test]
        #[ignore = "todo"]
        fn turns_hiragana_into_katakana() {}

        #[test]
        #[ignore = "todo"]
        fn turns_full_width_ascii_into_half_width() {}

        #[test]
        #[ignore = "todo"]
        fn turns_half_width_katakana_into_full_width() {}

        #[test]
        #[ignore = "todo"]
        fn turns_hiragana_vu_into_katakana_vu() {}
    }
}
