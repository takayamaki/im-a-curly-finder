//! `curly serve`: answers one JSON request per line, so the index is scanned only once.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::index::SharedIndex;

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    id: Value,
    op: String,
    q: Option<String>,
    limit: Option<usize>,
}

/// Answers one request line with one response line (without the newline).
pub fn handle(index: &SharedIndex, line: &str) -> String {
    let request: Request = serde_json::from_str(line).expect("a request");
    let index = index.snapshot();
    match request.op.as_str() {
        "tags" => json!({"id": request.id, "ok": true, "tags": index.tags()}).to_string(),
        _ => {
            let found = index
                .query(request.q.as_deref().expect("q"))
                .expect("a query");
            let total = found.len();
            let entries = &found[..request.limit.unwrap_or(total).min(total)];
            json!({"id": request.id, "ok": true, "total": total, "entries": entries}).to_string()
        }
    }
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
    fn query_echoes_the_id() {
        let index = shared_of(&["opening.mp4"]);
        let response = ask(&index, json!({"id": 12, "op": "query", "q": ""}));
        assert_eq!(response["id"], 12);
        let response = ask(&index, json!({"id": "a-1", "op": "query", "q": ""}));
        assert_eq!(response["id"], "a-1");
    }

    fn rels(response: &Value) -> Vec<&str> {
        response["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["rel"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn query_cuts_the_entries_at_the_limit_but_counts_them_all() {
        let index = shared_of(&["songC.mp4", "songA.mp4", "songB.mp4"]);
        let response = ask(
            &index,
            json!({"id": 1, "op": "query", "q": "song", "limit": 2}),
        );
        assert_eq!(response["total"], 3);
        assert_eq!(rels(&response), ["songA.mp4", "songB.mp4"]);
    }

    #[test]
    fn tags_returns_the_tag_counts() {
        let index = shared_of(&["MV{3D}/songA{solo}.mp4", "MV{３Ｄ}/songB.mp4"]);
        let response = ask(&index, json!({"id": 2, "op": "tags"}));
        assert_eq!(
            response,
            json!({
                "id": 2,
                "ok": true,
                "tags": [
                    {"tag": "3D", "count": 2, "spellings": [
                        {"text": "3D", "count": 1},
                        {"text": "３Ｄ", "count": 1},
                    ]},
                    {"tag": "solo", "count": 1, "spellings": [{"text": "solo", "count": 1}]},
                ],
            })
        );
    }

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
