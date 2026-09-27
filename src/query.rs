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
    Or(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Bar,
    Open,
    Close,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    for c in input.chars() {
        let symbol = match c {
            '|' => Some(Token::Bar),
            '(' => Some(Token::Open),
            ')' => Some(Token::Close),
            _ => None,
        };
        if c.is_whitespace() || symbol.is_some() {
            if !word.is_empty() {
                tokens.push(Token::Word(std::mem::take(&mut word)));
            }
            tokens.extend(symbol);
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        tokens.push(Token::Word(word));
    }
    tokens
}

/// and  := or*
/// or   := term ('|' term)*
/// term := '(' and ')' | word
struct Parser {
    tokens: std::iter::Peekable<std::vec::IntoIter<Token>>,
}

impl Parser {
    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut exprs = Vec::new();
        while self.tokens.peek().is_some_and(|t| *t != Token::Close) {
            exprs.push(self.or()?);
        }
        Ok(Expr::And(exprs))
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut exprs = vec![self.term()?];
        while self.tokens.next_if_eq(&Token::Bar).is_some() {
            exprs.push(self.term()?);
        }
        Ok(if exprs.len() == 1 {
            exprs.remove(0)
        } else {
            Expr::Or(exprs)
        })
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        match self.tokens.next() {
            Some(Token::Word(word)) => Ok(term(&word)),
            Some(Token::Open) => {
                let expr = self.and()?;
                match self.tokens.next() {
                    Some(Token::Close) => Ok(expr),
                    _ => Err(ParseError::UnbalancedParen),
                }
            }
            _ => Err(ParseError::MissingTerm),
        }
    }
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
            Expr::Or(exprs) => exprs.iter().any(|e| e.matches(subject)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// A word or tag is missing (`a |` etc.)
    MissingTerm,
    /// The numbers of `(` and `)` do not match
    UnbalancedParen,
}

impl Query {
    pub fn parse(input: &str) -> Result<Query, ParseError> {
        let mut parser = Parser {
            tokens: tokenize(input).into_iter().peekable(),
        };
        let expr = parser.and()?;
        if parser.tokens.next().is_some() {
            return Err(ParseError::UnbalancedParen);
        }
        Ok(Query { expr })
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
        fn a_bar_matches_either_side() {
            assert!(hits("#3D|#2D", "MV{3D}/song.mp4"));
            assert!(hits("#3D | #2D", "MV{2D}/song.mp4"));
            assert!(!hits("#3D|#2D", "MV/song.mp4"));
        }

        #[test]
        fn a_bar_binds_tighter_than_a_space() {
            // #solo AND (#3D OR #2D)
            assert!(hits("#solo #3D|#2D", "MV{2D}/song{solo}.mp4"));
            assert!(!hits("#solo #3D|#2D", "MV{3D}/song.mp4"));
        }

        #[test]
        fn parentheses_group_terms() {
            let query = "(#3D #solo)|#self-shot";
            assert!(hits(query, "MV{3D}/song{solo}.mp4"));
            assert!(hits(query, "{self-shot}/song.mp4"));
            assert!(!hits(query, "MV{3D}/song.mp4"));
        }

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
