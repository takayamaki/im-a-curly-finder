use std::fs;
use std::path::Path;

use assert_cmd::Command;

fn touch(root: &Path, rel: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"").unwrap();
}

fn vjfind(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("vjfind").unwrap();
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
    vjfind(dir.path())
        .args(["query", "#3D"])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn query_with_json_prints_entries_as_json() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "game-a/MV{3D}/opening{solo}.mp4");

    let output = vjfind(dir.path())
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

    vjfind(dir.path())
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

    vjfind(dir.path())
        .arg("lint")
        .assert()
        .success()
        .stdout("live｛3D｝\tfull-width ｛｝\nlive｛3D｝/songB{so{lo}}.mp4\tnested {}\n");
}

#[test]
fn a_broken_query_exits_with_an_error() {
    let dir = tempfile::tempdir().unwrap();
    touch(dir.path(), "song.mp4");

    vjfind(dir.path())
        .args(["query", "(#3D"])
        .assert()
        .failure()
        .stdout("")
        .stderr("vjfind: query: unbalanced ( and )\n");
}
