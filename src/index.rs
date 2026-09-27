//! The index. Built in memory by scanning the directory at startup.

use std::io;
use std::path::{Path, PathBuf};

use crate::tags::path_tags;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Entry {
    /// Absolute path (used when a GUI drops the file into another app)
    pub path: PathBuf,
    /// Path relative to the root (`/`-separated)
    pub rel: String,
    /// The directory part of `rel`. Empty for files directly under the root
    pub dir: String,
    pub name: String,
    pub stem: String,
    /// The extension without the `.`. Empty if there is none
    pub ext: String,
    pub tags: Vec<String>,
}

impl Entry {
    fn new(path: PathBuf, rel: String) -> Entry {
        let (dir, name) = rel.rsplit_once('/').unwrap_or(("", &rel));
        // A leading `.` is not an extension separator (`.DS_Store` etc.)
        let (stem, ext) = match name.rfind('.') {
            Some(i) if i > 0 => (&name[..i], &name[i + 1..]),
            _ => (name, ""),
        };
        Entry {
            path,
            dir: dir.to_string(),
            name: name.to_string(),
            stem: stem.to_string(),
            ext: ext.to_string(),
            tags: path_tags(&rel),
            rel,
        }
    }
}

#[derive(Debug)]
pub struct Index {
    pub root: PathBuf,
    pub entries: Vec<Entry>,
}

impl Index {
    /// Scans the files under `root` recursively. Directories themselves are not indexed.
    pub fn scan(root: impl AsRef<Path>) -> io::Result<Index> {
        let root = root.as_ref();
        let mut entries = Vec::new();
        for item in walkdir::WalkDir::new(root).min_depth(1) {
            let item = item?;
            if !item.file_type().is_file() {
                continue;
            }
            let rel = item.path().strip_prefix(root).unwrap_or(item.path());
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            entries.push(Entry::new(item.path().to_path_buf(), rel));
        }
        entries.sort_by(|a, b| a.rel.cmp(&b.rel));
        Ok(Index {
            root: root.to_path_buf(),
            entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn touch(root: &Path, rel: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"").unwrap();
    }

    mod scan {
        use super::touch;
        use crate::index::Index;

        #[test]
        fn lists_every_file_under_the_root() {
            let dir = tempfile::tempdir().unwrap();
            touch(dir.path(), "game-a/MV{3D}/opening.mp4");
            touch(dir.path(), "clip.mp4");
            fs_mkdir(dir.path(), "empty-folder");

            let index = Index::scan(dir.path()).unwrap();
            let rels: Vec<&str> = index.entries.iter().map(|e| e.rel.as_str()).collect();
            assert_eq!(rels, ["clip.mp4", "game-a/MV{3D}/opening.mp4"]);
        }

        fn fs_mkdir(root: &std::path::Path, rel: &str) {
            std::fs::create_dir_all(root.join(rel)).unwrap();
        }

        #[test]
        fn splits_each_name_into_dir_stem_and_extension() {
            let dir = tempfile::tempdir().unwrap();
            touch(dir.path(), "game-a/MV{3D}/opening{solo}.mp4");

            let index = Index::scan(dir.path()).unwrap();
            let entry = &index.entries[0];
            assert_eq!(
                entry.path,
                dir.path().join("game-a/MV{3D}/opening{solo}.mp4")
            );
            assert_eq!(entry.dir, "game-a/MV{3D}");
            assert_eq!(entry.name, "opening{solo}.mp4");
            assert_eq!(entry.stem, "opening{solo}");
            assert_eq!(entry.ext, "mp4");
            assert_eq!(entry.tags, ["3D", "solo"]);
        }
    }

    mod query {
        #[test]
        #[ignore = "todo"]
        fn returns_the_matching_entries_in_path_order() {}

        #[test]
        #[ignore = "todo"]
        fn returns_the_parse_error_for_a_broken_query() {}
    }

    mod tags {
        #[test]
        #[ignore = "todo"]
        fn counts_files_per_tag_most_used_first() {}

        #[test]
        #[ignore = "todo"]
        fn groups_spellings_that_normalize_to_the_same_tag() {}
    }

    mod lint {
        #[test]
        #[ignore = "todo"]
        fn lists_names_with_issues() {}

        #[test]
        #[ignore = "todo"]
        fn reports_a_folder_once_not_per_file() {}
    }

    mod shared_index {
        #[test]
        #[ignore = "todo"]
        fn rescan_replaces_the_index() {}

        #[test]
        #[ignore = "todo"]
        fn a_snapshot_taken_before_rescan_stays_usable() {}
    }
}
