//! Searching the index for the result list.

use curly_finder::index::{Entry, Index, TagCount, count_tags};
use serde::Serialize;

/// What the result list shows.
#[derive(Debug, Serialize)]
pub struct Found {
    /// The first `limit` matches, in path order
    pub entries: Vec<Entry>,
    /// Number of all matches
    pub total: usize,
    /// The tags of all matches, most used first
    pub tags: Vec<TagCount>,
}

pub fn search(index: &Index, expr: &str, limit: usize) -> Result<Found, String> {
    let matches = index.query(expr).map_err(|e| e.to_string())?;
    Ok(Found {
        total: matches.len(),
        tags: count_tags(matches.iter().copied()),
        entries: matches.into_iter().take(limit).cloned().collect(),
    })
}

#[cfg(test)]
mod tests {
    use curly_finder::index::Index;

    fn index_of(rels: &[&str]) -> Index {
        Index::from_rels("/root", rels.iter().map(|r| r.to_string()))
    }

    fn rels(found: &super::Found) -> Vec<&str> {
        found.entries.iter().map(|e| e.rel.as_str()).collect()
    }

    fn tag_counts(found: &super::Found) -> Vec<(&str, usize)> {
        found
            .tags
            .iter()
            .map(|t| (t.tag.as_str(), t.count))
            .collect()
    }

    mod search {
        use super::{index_of, rels, tag_counts};
        use crate::search::search;

        #[test]
        fn returns_the_entries_that_match_the_query() {
            let index = index_of(&[
                "game-a/MV{3D}/opening{solo}.mp4",
                "game-a/MV{2D}/Blue Sky.mp4",
                "game-b/MV{3D}/good night!.mp4",
            ]);
            let found = search(&index, "#3D", 100).unwrap();
            assert_eq!(
                rels(&found),
                [
                    "game-a/MV{3D}/opening{solo}.mp4",
                    "game-b/MV{3D}/good night!.mp4"
                ]
            );
            assert_eq!(found.total, 2);
        }

        #[test]
        fn returns_every_entry_for_an_empty_query() {
            let index = index_of(&["b.mp4", "a/c.mp4"]);
            let found = search(&index, "", 100).unwrap();
            assert_eq!(rels(&found), ["a/c.mp4", "b.mp4"]);
            assert_eq!(found.total, 2);
        }

        #[test]
        fn returns_at_most_limit_entries_with_the_total_count() {
            let index = index_of(&["a.mp4", "b.mp4", "c.mp4"]);
            let found = search(&index, "", 2).unwrap();
            assert_eq!(rels(&found), ["a.mp4", "b.mp4"]);
            assert_eq!(found.total, 3);
        }

        #[test]
        fn returns_the_tags_of_the_matches_most_used_first() {
            let index = index_of(&[
                "MV{3D}/songA{solo}.mp4",
                "MV{3D}/songB.mp4",
                "MV{2D}/songC{solo}.mp4",
                "live{3D}/songD{solo}.mp4",
            ]);
            let found = search(&index, "MV", 100).unwrap();
            assert_eq!(tag_counts(&found), [("3D", 2), ("solo", 2), ("2D", 1)]);
        }

        #[test]
        #[ignore = "todo"]
        fn counts_the_tags_of_matches_beyond_the_limit() {}

        #[test]
        fn returns_the_error_message_for_a_broken_query() {
            let index = index_of(&["a.mp4"]);
            assert_eq!(search(&index, "(a", 100).unwrap_err(), "unbalanced ( and )");
        }
    }
}
