//! Search queries.
//!
//! - `word` matches any part of the path outside tags
//! - `#tag` matches a whole tag
//! - Spaces mean AND, `|` means OR (binds tighter than AND), a leading `-` means NOT
//! - `( )` groups terms. Put a word with spaces in `"..."`

use crate::normalize::normalize;

/// What a query is matched against. Both fields must already be [`crate::normalize::normalize`]d.
pub struct Subject<'a> {
    /// Path relative to the root, with the tags removed
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
    /// A `-` at the start of a word
    Minus,
    /// A `#` at the start of a word
    Hash,
}

/// Inside `"..."`, symbols and spaces are part of the word.
/// `-` and `#` must be followed by a word (`# song` is an error).
fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    for c in input.chars() {
        if quoted {
            if c == '"' {
                quoted = false;
            } else {
                word.push(c);
            }
            continue;
        }
        let symbol = match c {
            '|' => Some(Token::Bar),
            '(' => Some(Token::Open),
            ')' => Some(Token::Close),
            _ => None,
        };
        if c.is_whitespace() || symbol.is_some() {
            if word.is_empty() && matches!(tokens.last(), Some(Token::Minus | Token::Hash)) {
                return Err(ParseError::MissingTerm);
            }
            if !word.is_empty() {
                tokens.push(Token::Word(std::mem::take(&mut word)));
            }
            tokens.extend(symbol);
        } else if c == '"' {
            quoted = true;
        } else if word.is_empty() && c == '-' {
            tokens.push(Token::Minus);
        } else if word.is_empty() && c == '#' {
            tokens.push(Token::Hash);
        } else {
            word.push(c);
        }
    }
    if quoted {
        return Err(ParseError::UnclosedQuote);
    }
    if !word.is_empty() {
        tokens.push(Token::Word(word));
    }
    Ok(tokens)
}

/// and   := or*
/// or    := unary ('|' unary)*
/// unary := '-' unary | '(' and ')' | '#' word | word
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
        let mut exprs = vec![self.unary()?];
        while self.tokens.next_if_eq(&Token::Bar).is_some() {
            exprs.push(self.unary()?);
        }
        Ok(if exprs.len() == 1 {
            exprs.remove(0)
        } else {
            Expr::Or(exprs)
        })
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.tokens.next() {
            Some(Token::Minus) => Ok(Expr::Not(Box::new(self.unary()?))),
            Some(Token::Open) => {
                let expr = self.and()?;
                match self.tokens.next() {
                    Some(Token::Close) => Ok(expr),
                    _ => Err(ParseError::UnbalancedParen),
                }
            }
            Some(Token::Hash) => match self.tokens.next() {
                Some(Token::Word(tag)) => Ok(Expr::Tag(normalize(&tag))),
                _ => Err(ParseError::MissingTerm),
            },
            Some(Token::Word(word)) => Ok(Expr::Text(normalize(&word))),
            _ => Err(ParseError::MissingTerm),
        }
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
    /// An unclosed `"`
    UnclosedQuote,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ParseError::MissingTerm => "missing a word after |, - or #",
            ParseError::UnbalancedParen => "unbalanced ( and )",
            ParseError::UnclosedQuote => "unclosed \"",
        })
    }
}

impl std::error::Error for ParseError {}

impl Query {
    pub fn parse(input: &str) -> Result<Query, ParseError> {
        let mut parser = Parser {
            tokens: tokenize(input)?.into_iter().peekable(),
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

    /// This query AND every one of `tags`, as if each were written `#tag`.
    pub fn and_tags<'a>(self, tags: impl IntoIterator<Item = &'a str>) -> Query {
        let mut exprs: Vec<Expr> = tags.into_iter().map(|t| Expr::Tag(normalize(t))).collect();
        exprs.push(self.expr);
        Query {
            expr: Expr::And(exprs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Query, Subject};
    use crate::normalize::normalize;
    use crate::tags::path_tags;

    fn hits(query: &str, rel: &str) -> bool {
        query_hits(&Query::parse(query).unwrap(), rel)
    }

    fn query_hits(query: &Query, rel: &str) -> bool {
        let text = normalize(rel);
        let tags: Vec<String> = path_tags(rel).iter().map(|t| normalize(t)).collect();
        query.matches(&Subject {
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
            assert!(hits("#がぎぐ", "{ガギグ}/opening.mp4"));
            assert!(!hits("#solo", "game-a/opening.mp4"));
        }

        #[test]
        fn a_hash_does_not_match_part_of_a_tag() {
            assert!(!hits("#game-a", "{game-a-tour}/song.mp4"));
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
        fn quotes_keep_spaces_in_a_word() {
            assert!(hits(r#""Blue Sky""#, "mv/Blue Sky.mp4"));
            assert!(!hits(r#""Blue Sky""#, "mv/Sky Blue.mp4"));
            assert!(hits(r#""-(REMIX)""#, "song-(REMIX).mp4"));
        }

        #[test]
        fn quotes_keep_spaces_in_a_tag() {
            assert!(hits(r#"#"Blue Sky""#, "{Blue Sky}/song.mp4"));
        }

        #[test]
        fn an_empty_query_matches_everything() {
            assert!(hits("", "song.mp4"));
            assert!(hits("　 ", "song.mp4"));
        }
    }

    mod and_tags {
        use super::query_hits;
        use crate::query::Query;

        fn hits_with(query: &str, tags: &[&str], rel: &str) -> bool {
            query_hits(
                &Query::parse(query).unwrap().and_tags(tags.iter().copied()),
                rel,
            )
        }

        #[test]
        fn the_tags_narrow_down_the_query() {
            assert!(hits_with("song", &["solo"], "MV/song{solo}.mp4"));
            assert!(!hits_with("song", &["solo"], "MV/song.mp4"));
        }

        #[test]
        fn every_tag_must_match() {
            assert!(hits_with("", &["3D", "solo"], "MV{3D}/song{solo}.mp4"));
            assert!(!hits_with("", &["3D", "solo"], "MV{3D}/song.mp4"));
        }

        #[test]
        fn the_tags_alone_narrow_down_an_empty_query() {
            assert!(hits_with("", &["solo"], "song{solo}.mp4"));
            assert!(!hits_with("", &["solo"], "song.mp4"));
        }

        #[test]
        fn a_tag_is_compared_after_normalization() {
            assert!(hits_with("", &["がぎぐ"], "{ガギグ}/song.mp4"));
            assert!(hits_with("", &["ＭＶ"], "{mv}/song.mp4"));
        }

        #[test]
        fn a_bar_or_a_minus_in_the_query_does_not_reach_the_tags() {
            // (#2D | song) AND #solo, not #2D | (song AND #solo)
            assert!(!hits_with("#2D|song", &["solo"], "MV{2D}/a.mp4"));
            // (-song) AND #solo, not -(song AND #solo)
            assert!(!hits_with("-song", &["solo"], "a.mp4"));
            assert!(hits_with("-song", &["solo"], "a{solo}.mp4"));
        }
    }

    mod parse_errors {
        use crate::query::{ParseError, Query};

        #[test]
        fn an_unclosed_parenthesis_is_an_error() {
            assert_eq!(Query::parse("(#3D #2D"), Err(ParseError::UnbalancedParen));
            assert_eq!(Query::parse("#3D)"), Err(ParseError::UnbalancedParen));
        }

        #[test]
        fn an_unclosed_quote_is_an_error() {
            assert_eq!(Query::parse(r#""Blue Sky"#), Err(ParseError::UnclosedQuote));
        }

        #[test]
        fn a_dangling_bar_is_an_error() {
            assert_eq!(Query::parse("#3D |"), Err(ParseError::MissingTerm));
            assert_eq!(Query::parse("| #3D"), Err(ParseError::MissingTerm));
        }

        #[test]
        fn a_lone_minus_or_hash_is_an_error() {
            assert_eq!(Query::parse("song -"), Err(ParseError::MissingTerm));
            assert_eq!(Query::parse("# song"), Err(ParseError::MissingTerm));
        }
    }
}
