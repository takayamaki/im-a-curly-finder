//! Searching the index for the result list.

use curly_finder::index::{Entry, Index};
use serde::Serialize;

/// What the result list shows.
#[derive(Debug, Serialize)]
pub struct Found {
    /// The first `limit` matches, in path order
    pub entries: Vec<Entry>,
    /// Number of all matches
    pub total: usize,
}

pub fn search(index: &Index, expr: &str, limit: usize) -> Result<Found, String> {
    let _ = limit;
    let matches = index.query(expr).unwrap();
    Ok(Found {
        total: matches.len(),
        entries: matches.into_iter().cloned().collect(),
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

    mod search {
        use super::{index_of, rels};
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
        #[ignore = "todo"]
        fn returns_at_most_limit_entries_with_the_total_count() {}

        #[test]
        #[ignore = "todo"]
        fn returns_the_error_message_for_a_broken_query() {}
    }
}
