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
#[ignore = "todo"]
fn query_with_json_prints_entries_as_json() {}

#[test]
#[ignore = "todo"]
fn tags_prints_counts() {}

#[test]
#[ignore = "todo"]
fn lint_prints_issues() {}

#[test]
#[ignore = "todo"]
fn a_broken_query_exits_with_an_error() {}
