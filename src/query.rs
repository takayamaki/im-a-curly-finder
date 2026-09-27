//! Search queries.
//!
//! - `word` matches any part of the path
//! - `#tag` matches a whole tag
//! - Spaces mean AND, `|` means OR (binds tighter than AND), a leading `-` means NOT
//! - `( )` groups terms. Put a word with spaces in `"..."`

/// What a query is matched against. Both fields must already be [`crate::normalize::normalize`]d.
pub struct Subject<'a> {
    /// Path relative to the root
    pub text: &'a str,
    pub tags: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    word: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {}

impl Query {
    pub fn parse(input: &str) -> Result<Query, ParseError> {
        Ok(Query {
            word: input.to_string(),
        })
    }

    pub fn matches(&self, subject: &Subject) -> bool {
        subject.text.contains(&self.word)
    }
}

#[cfg(test)]
mod tests {
    use super::{Query, Subject};
    use crate::normalize::normalize;
    use crate::tags::path_tags;

    fn hits(query: &str, rel: &str) -> bool {
        let text = normalize(rel);
        let tags: Vec<String> = path_tags(rel).iter().map(|t| normalize(t)).collect();
        Query::parse(query).unwrap().matches(&Subject {
            text: &text,
            tags: &tags,
        })
    }

    mod matches {
        use super::hits;

        #[test]
        fn a_word_matches_part_of_the_path() {
            assert!(hits("morning", "game-a/good morning.mp4"));
            assert!(!hits("morning", "game-b/good night!.mp4"));
        }

        #[test]
        #[ignore = "todo"]
        fn a_word_is_compared_after_normalization() {}

        #[test]
        #[ignore = "todo"]
        fn words_separated_by_spaces_must_all_match() {}

        #[test]
        #[ignore = "todo"]
        fn a_hash_matches_a_whole_tag() {}

        #[test]
        #[ignore = "todo"]
        fn a_hash_does_not_match_part_of_a_tag() {}

        #[test]
        #[ignore = "todo"]
        fn a_minus_excludes_matches() {}

        #[test]
        #[ignore = "todo"]
        fn a_bar_matches_either_side() {}

        #[test]
        #[ignore = "todo"]
        fn a_bar_binds_tighter_than_a_space() {}

        #[test]
        #[ignore = "todo"]
        fn parentheses_group_terms() {}

        #[test]
        #[ignore = "todo"]
        fn quotes_keep_spaces_in_a_word() {}

        #[test]
        #[ignore = "todo"]
        fn quotes_keep_spaces_in_a_tag() {}

        #[test]
        #[ignore = "todo"]
        fn an_empty_query_matches_everything() {}
    }

    mod parse_errors {
        #[test]
        #[ignore = "todo"]
        fn an_unclosed_parenthesis_is_an_error() {}

        #[test]
        #[ignore = "todo"]
        fn an_unclosed_quote_is_an_error() {}

        #[test]
        #[ignore = "todo"]
        fn a_dangling_bar_is_an_error() {}

        #[test]
        #[ignore = "todo"]
        fn a_lone_minus_or_hash_is_an_error() {}
    }
}
