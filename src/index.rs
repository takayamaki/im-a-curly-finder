//! The index. Built in memory by scanning the directory at startup.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::normalize::normalize;
use crate::query::{ParseError, Query, Subject};
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
    /// `rel`, normalized for matching
    #[serde(skip)]
    text_key: String,
    /// `tags`, normalized for matching
    #[serde(skip)]
    tag_keys: Vec<String>,
}

impl Entry {
    fn new(root: &Path, rel: String) -> Entry {
        let path = rel.split('/').fold(root.to_path_buf(), |p, c| p.join(c));
        let tags = path_tags(&rel);
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
            text_key: normalize(&rel),
            tag_keys: tags.iter().map(|t| normalize(t)).collect(),
            tags,
            rel,
        }
    }
}

#[derive(Debug)]
pub struct Index {
    pub root: PathBuf,
    pub entries: Vec<Entry>,
}

/// One row of [`Index::tags`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TagCount {
    pub tag: String,
    /// Number of files with this tag
    pub count: usize,
}

impl Index {
    /// Builds an index from a list of relative paths, without scanning.
    pub fn from_rels(root: impl AsRef<Path>, rels: impl IntoIterator<Item = String>) -> Index {
        let root = root.as_ref();
        let mut entries: Vec<Entry> = rels.into_iter().map(|rel| Entry::new(root, rel)).collect();
        entries.sort_by(|a, b| a.rel.cmp(&b.rel));
        Index {
            root: root.to_path_buf(),
            entries,
        }
    }

    /// Lists the tags, most used first. Meant for spotting typos by eye.
    pub fn tags(&self) -> Vec<TagCount> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for tag in self.entries.iter().flat_map(|e| &e.tags) {
            *counts.entry(tag).or_default() += 1;
        }
        let mut tags: Vec<TagCount> = counts
            .into_iter()
            .map(|(tag, count)| TagCount {
                tag: tag.to_string(),
                count,
            })
            .collect();
        tags.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.tag.cmp(&b.tag)));
        tags
    }

    pub fn query(&self, input: &str) -> Result<Vec<&Entry>, ParseError> {
        let query = Query::parse(input)?;
        Ok(self
            .entries
            .iter()
            .filter(|e| {
                query.matches(&Subject {
                    text: &e.text_key,
                    tags: &e.tag_keys,
                })
            })
            .collect())
    }

    /// Scans the files under `root` recursively. Directories themselves are not indexed.
    pub fn scan(root: impl AsRef<Path>) -> io::Result<Index> {
        let root = root.as_ref();
        let mut rels = Vec::new();
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
            rels.push(rel);
        }
        Ok(Index::from_rels(root, rels))
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

    fn index_of(rels: &[&str]) -> crate::index::Index {
        crate::index::Index::from_rels("/vj", rels.iter().map(|r| r.to_string()))
    }

    fn rels(entries: &[&crate::index::Entry]) -> Vec<String> {
        entries.iter().map(|e| e.rel.clone()).collect()
    }

    mod query {
        use super::{index_of, rels};

        #[test]
        fn returns_the_matching_entries_in_path_order() {
            let index = index_of(&[
                "game-a/MV{3D}/opening.mp4",
                "game-a/MV{3D}/Blue Sky.mp4",
                "game-b/MV{3D}/good night!.mp4",
            ]);
            let found = index.query("GAME-A #3d").unwrap();
            assert_eq!(
                rels(&found),
                ["game-a/MV{3D}/Blue Sky.mp4", "game-a/MV{3D}/opening.mp4"]
            );
        }

        #[test]
        fn returns_the_parse_error_for_a_broken_query() {
            let index = index_of(&["song.mp4"]);
            assert_eq!(
                index.query("(#3D").unwrap_err(),
                crate::query::ParseError::UnbalancedParen
            );
        }
    }

    mod tags {
        use super::index_of;

        #[test]
        fn counts_files_per_tag_most_used_first() {
            let index = index_of(&[
                "MV{3D}/songA{solo}.mp4",
                "MV{3D}/songB.mp4",
                "MV{2D}/songC{solo}.mp4",
                "MV{3D}/songD.mp4",
            ]);
            let counts: Vec<(String, usize)> =
                index.tags().into_iter().map(|t| (t.tag, t.count)).collect();
            assert_eq!(
                counts,
                [
                    ("3D".to_string(), 3),
                    ("solo".to_string(), 2),
                    ("2D".to_string(), 1)
                ]
            );
        }

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
