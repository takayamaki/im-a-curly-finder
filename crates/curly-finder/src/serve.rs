//! `curly serve`: answers one JSON request per line, so the index is scanned only once.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::index::SharedIndex;

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    id: Value,
    q: String,
}

/// Answers one request line with one response line (without the newline).
pub fn handle(index: &SharedIndex, line: &str) -> String {
    let request: Request = serde_json::from_str(line).expect("a request");
    let index = index.snapshot();
    let found = index.query(&request.q).expect("a query");
    json!({"id": request.id, "ok": true, "total": found.len(), "entries": found}).to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::handle;
    use crate::index::{Index, SharedIndex};

    fn shared_of(rels: &[&str]) -> SharedIndex {
        SharedIndex::new(Index::from_rels("/vj", rels.iter().map(|r| r.to_string())))
    }

    fn ask(index: &SharedIndex, request: Value) -> Value {
        serde_json::from_str(&handle(index, &request.to_string())).unwrap()
    }

    #[test]
    fn query_returns_the_matching_entries_and_the_total() {
        let index = shared_of(&["MV{3D}/opening{solo}.mp4", "MV{2D}/ending.mp4"]);
        let response = ask(&index, json!({"id": 1, "op": "query", "q": "#solo"}));
        assert_eq!(
            response,
            json!({
                "id": 1,
                "ok": true,
                "total": 1,
                "entries": [{
                    "path": "/vj/MV{3D}/opening{solo}.mp4",
                    "rel": "MV{3D}/opening{solo}.mp4",
                    "dir": "MV{3D}",
                    "name": "opening{solo}.mp4",
                    "stem": "opening{solo}",
                    "ext": "mp4",
                    "tags": ["3D", "solo"],
                }],
            })
        );
    }

    #[test]
    #[ignore]
    fn query_echoes_the_id() {}

    #[test]
    #[ignore]
    fn query_cuts_the_entries_at_the_limit_but_counts_them_all() {}

    #[test]
    #[ignore]
    fn tags_returns_the_tag_counts() {}

    #[test]
    #[ignore]
    fn rescan_picks_up_a_file_added_since_the_start() {}

    #[test]
    #[ignore]
    fn ping_answers_ok() {}

    #[test]
    #[ignore]
    fn a_broken_query_answers_the_parse_error() {}

    #[test]
    #[ignore]
    fn a_query_without_q_answers_an_error() {}

    #[test]
    #[ignore]
    fn an_unknown_op_answers_an_error_with_the_id() {}

    #[test]
    #[ignore]
    fn invalid_json_answers_an_error_with_a_null_id() {}

    #[test]
    #[ignore]
    fn a_limit_of_zero_returns_no_entries_but_the_total() {}

    #[test]
    #[ignore]
    fn a_rescan_that_fails_answers_an_error_and_keeps_the_old_index() {}
}
