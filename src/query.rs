//! Search queries.
//!
//! - `word` matches any part of the path
//! - `#tag` matches a whole tag
//! - Spaces mean AND, `|` means OR (binds tighter than AND), a leading `-` means NOT
//! - `( )` groups terms. Put a word with spaces in `"..."`

use crate::normalize::normalize;

/// What a query is matched against. Both fields must already be [`crate::normalize::normalize`]d.
pub struct Subject<'a> {
    /// Path relative to the root
    pub text: &'a str,
    pub tags: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Text(String),
    Tag(String),
    Not(Box<Expr>),
    And(Vec<Expr>),
}

fn term(word: &str) -> Expr {
    if let Some(rest) = word.strip_prefix('-') {
        return Expr::Not(Box::new(term(rest)));
    }
    match word.strip_prefix('#') {
        Some(tag) => Expr::Tag(normalize(tag)),
        None => Expr::Text(normalize(word)),
    }
}

impl Expr {
    fn matches(&self, subject: &Subject) -> bool {
        match self {
            Expr::Text(word) => subject.text.contains(word.as_str()),
            Expr::Tag(tag) => subject.tags.contains(tag),
            Expr::Not(expr) => !expr.matches(subject),
            Expr::And(exprs) => exprs.iter().all(|e| e.matches(subject)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {}

impl Query {
    pub fn parse(input: &str) -> Result<Query, ParseError> {
        let words = input.split_whitespace().map(term).collect();
        Ok(Query {
            expr: Expr::And(words),
        })
    }

    pub fn matches(&self, subject: &Subject) -> bool {
        self.expr.matches(subject)
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
        fn a_word_is_compared_after_normalization() {
            assert!(hits("あいう", "game-a/アイウ.mp4"));
            assert!(hits("ｍｖ", "game-a/MV/song.mp4"));
        }

        #[test]
        fn words_separated_by_spaces_must_all_match() {
            assert!(hits("game-a morning", "game-a/good morning.mp4"));
            assert!(!hits("game-b morning", "game-a/good morning.mp4"));
        }

        #[test]
        fn a_hash_matches_a_whole_tag() {
            assert!(hits("#solo", "game-a/opening{solo}.mp4"));
            assert!(hits("#がぎぐ", "ガギグ/opening.mp4"));
            assert!(!hits("#solo", "game-a/opening.mp4"));
        }

        #[test]
        fn a_hash_does_not_match_part_of_a_tag() {
            assert!(!hits("#game-a", "game-a-tour/song.mp4"));
        }

        #[test]
        fn a_minus_excludes_matches() {
            assert!(!hits("game-a -#solo", "game-a/song{solo}.mp4"));
            assert!(hits("game-a -#solo", "game-a/song.mp4"));
            assert!(!hits("-song", "game-a/song.mp4"));
        }

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
