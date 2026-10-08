use std::fs;
use std::path::Path;

use assert_cmd::Command;

fn touch(root: &Path, rel: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"").unwrap();
}

fn curly(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("curly").unwrap();
    cmd.arg("--root").arg(root);
    cmd
}

#[test]
fn query_prints_matching_paths_one_per_line() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "game-a/MV{3D}/opening.mp4");
    touch(dir.path(), "game-a/MV{2D}/Blue Sky.mp4");
    touch(dir.path(), "game-b/MV{3D}/good night!.mp4");

    let expected = format!(
        "{}\n{}\n",
        dir.path().join("game-a/MV{3D}/opening.mp4").display(),
        dir.path().join("game-b/MV{3D}/good night!.mp4").display(),
    );
    curly(dir.path())
        .args(["query", "#3D"])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn query_with_json_prints_entries_as_json() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "game-a/MV{3D}/opening{solo}.mp4");

    let output = curly(dir.path())
        .args(["--json", "query", "#solo"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json,
        serde_json::json!([{
            "path": dir.path().join("game-a/MV{3D}/opening{solo}.mp4"),
            "rel": "game-a/MV{3D}/opening{solo}.mp4",
            "dir": "game-a/MV{3D}",
            "name": "opening{solo}.mp4",
            "stem": "opening{solo}",
            "ext": "mp4",
            "tags": ["3D", "solo"],
        }])
    );
}

#[test]
fn tags_prints_counts() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "MV{3D}/songA{solo}.mp4");
    touch(dir.path(), "MV{3D}/songB.mp4");
    touch(dir.path(), "MV{３Ｄ}/songC.mp4");

    curly(dir.path())
        .arg("tags")
        .assert()
        .success()
        .stdout("3\t3D\t(3D ×2, ３Ｄ ×1)\n1\tsolo\n");
}

#[test]
fn lint_prints_issues() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "live｛3D｝/songA.mp4");
    touch(dir.path(), "live｛3D｝/songB{so{lo}}.mp4");
    touch(dir.path(), "songC.mp4");

    curly(dir.path())
        .arg("lint")
        .assert()
        .success()
        .stdout("live｛3D｝\tfull-width ｛｝\nlive｛3D｝/songB{so{lo}}.mp4\tnested {}\n");
}

#[test]
fn a_broken_query_exits_with_an_error() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "song.mp4");

    curly(dir.path())
        .args(["query", "(#3D"])
        .assert()
        .failure()
        .stdout("")
        .stderr("curly: query: unbalanced ( and )\n");
}

#[test]
#[cfg(unix)]
fn folders_that_cannot_be_read_are_reported_on_stderr() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "clip.mp4");
    touch(dir.path(), "locked/secret.mp4");
    let locked = dir.path().join("locked");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let assert = curly(dir.path()).args(["query", ""]).assert();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    assert
        .success()
        .stdout(format!("{}\n", dir.path().join("clip.mp4").display()))
        .stderr("curly: could not read: locked\n");
}

/// Sends `line` to a running `curly serve` and reads one response line.
fn ask(
    stdin: &mut impl std::io::Write,
    stdout: &mut impl std::io::BufRead,
    line: &str,
) -> serde_json::Value {
    writeln!(stdin, "{line}").unwrap();
    stdin.flush().unwrap();
    let mut response = String::new();
    stdout.read_line(&mut response).unwrap();
    serde_json::from_str(&response).unwrap_or_else(|e| panic!("not a JSON line: {response:?}: {e}"))
}

#[test]
fn serve_answers_each_request_line_with_a_response_line() {
    use std::io::BufReader;
    use std::process::Stdio;

    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "game-a/MV{3D}/opening.mp4");
    touch(dir.path(), "game-a/MV{2D}/Blue Sky.mp4");

    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("curly"))
        .arg("--root")
        .arg(dir.path())
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    let response = ask(
        &mut stdin,
        &mut stdout,
        r##"{"id": 1, "op": "query", "q": "#3D"}"##,
    );
    assert_eq!(response["id"], 1);
    assert_eq!(response["total"], 1);
    assert_eq!(response["entries"][0]["rel"], "game-a/MV{3D}/opening.mp4");
    let response = ask(&mut stdin, &mut stdout, r#"{"id": 2, "op": "ping"}"#);
    assert_eq!(response, serde_json::json!({"id": 2, "ok": true}));

    drop(stdin);
    assert!(child.wait().unwrap().success());
}

#[test]
#[ignore]
fn serve_keeps_serving_after_a_broken_request() {}

#[test]
#[ignore]
fn serve_exits_successfully_when_stdin_is_closed() {}

#[test]
#[ignore]
fn serve_does_not_list_macos_resource_fork_files() {}
