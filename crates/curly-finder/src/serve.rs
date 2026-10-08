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
    let mut response = match answer(index, &request) {
        Ok(body) => body,
        Err(error) => json!({"ok": false, "error": error}),
    };
    response["id"] = request.id;
    response.to_string()
}

/// The response to `request` without the `id`.
fn answer(index: &SharedIndex, request: &Request) -> Result<Value, String> {
    Ok(match request.op.as_str() {
        "ping" => json!({"ok": true}),
        "tags" => json!({"ok": true, "tags": index.snapshot().tags()}),
        "rescan" => {
            index.rescan().expect("a rescan");
            json!({"ok": true, "total": index.snapshot().entries.len()})
        }
        _ => {
            let index = index.snapshot();
            let q = request.q.as_deref().ok_or("query: missing q")?;
            let found = index.query(q).map_err(|e| e.to_string())?;
            let total = found.len();
            let entries = &found[..request.limit.unwrap_or(total).min(total)];
            json!({"ok": true, "total": total, "entries": entries})
        }
    })
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
    fn rescan_picks_up_a_file_added_since_the_start() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("songA.mp4"), b"").unwrap();
        let index = SharedIndex::new(Index::scan(dir.path()).unwrap());
        std::fs::write(dir.path().join("songB.mp4"), b"").unwrap();

        let response = ask(&index, json!({"id": 3, "op": "rescan"}));
        assert_eq!(response, json!({"id": 3, "ok": true, "total": 2}));
        let response = ask(&index, json!({"id": 4, "op": "query", "q": "song"}));
        assert_eq!(rels(&response), ["songA.mp4", "songB.mp4"]);
    }

    #[test]
    fn ping_answers_ok() {
        let index = shared_of(&["opening.mp4"]);
        let response = ask(&index, json!({"id": 5, "op": "ping"}));
        assert_eq!(response, json!({"id": 5, "ok": true}));
    }

    #[test]
    fn a_broken_query_answers_the_parse_error() {
        let index = shared_of(&["opening.mp4"]);
        let response = ask(&index, json!({"id": 6, "op": "query", "q": "(#3D"}));
        assert_eq!(
            response,
            json!({"id": 6, "ok": false, "error": "unbalanced ( and )"})
        );
    }

    #[test]
    fn a_query_without_q_answers_an_error() {
        let index = shared_of(&["opening.mp4"]);
        let response = ask(&index, json!({"id": 7, "op": "query"}));
        assert_eq!(
            response,
            json!({"id": 7, "ok": false, "error": "query: missing q"})
        );
    }

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
