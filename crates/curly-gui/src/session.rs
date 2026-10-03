//! The folder that is open now and its index.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use curly_finder::index::{Index, SharedIndex};

use crate::search::{Found, search};

/// The message shown when no folder is open yet.
pub const NO_FOLDER: &str = "No folder is open. Choose one with the Folder button.";

#[derive(Default)]
pub struct Session {
    current: RwLock<Option<Arc<SharedIndex>>>,
}

impl Session {
    pub fn new() -> Session {
        Session::default()
    }

    /// Scans `dir` and makes it the open folder.
    pub fn open(&self, dir: &Path) -> Result<(), String> {
        let index = Index::scan(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        *self.current.write().unwrap() = Some(Arc::new(SharedIndex::new(index)));
        Ok(())
    }

    /// The open folder, if any.
    pub fn root(&self) -> Option<PathBuf> {
        let current = self.current.read().unwrap();
        current
            .as_ref()
            .map(|shared| shared.snapshot().root.clone())
    }

    /// Scans the open folder again. Searches keep working while it runs.
    pub fn rescan(&self) -> Result<(), String> {
        let shared = self.current.read().unwrap().clone();
        let shared = shared.ok_or(NO_FOLDER)?;
        shared.rescan().map_err(|e| e.to_string())
    }

    /// Searches the open folder. See [`crate::search::search`].
    pub fn search(&self, expr: &str, limit: usize) -> Result<Found, String> {
        let current = self.current.read().unwrap();
        let shared = current.as_ref().ok_or(NO_FOLDER)?;
        search(&shared.snapshot(), expr, limit)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::search::Found;

    fn touch(root: &Path, rel: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"").unwrap();
    }

    fn rels(found: &Found) -> Vec<&str> {
        found.entries.iter().map(|e| e.rel.as_str()).collect()
    }

    mod session {
        use super::{rels, touch};
        use crate::session::{NO_FOLDER, Session};

        #[test]
        fn finds_files_in_the_opened_folder() {
            let dir = tempfile::tempdir().unwrap();
            touch(dir.path(), "game-a/MV{3D}/opening.mp4");
            touch(dir.path(), "game-a/MV{2D}/Blue Sky.mp4");
            let session = Session::new();
            session.open(dir.path()).unwrap();
            let found = session.search("#3D", 100).unwrap();
            assert_eq!(rels(&found), ["game-a/MV{3D}/opening.mp4"]);
        }

        #[test]
        fn tells_which_folder_is_open() {
            let dir = tempfile::tempdir().unwrap();
            let session = Session::new();
            session.open(dir.path()).unwrap();
            assert_eq!(session.root().as_deref(), Some(dir.path()));
        }

        #[test]
        fn opening_another_folder_replaces_the_index() {
            let a = tempfile::tempdir().unwrap();
            let b = tempfile::tempdir().unwrap();
            touch(a.path(), "a.mp4");
            touch(b.path(), "b.mp4");
            let session = Session::new();
            session.open(a.path()).unwrap();
            session.open(b.path()).unwrap();
            assert_eq!(rels(&session.search("", 100).unwrap()), ["b.mp4"]);
            assert_eq!(session.root().as_deref(), Some(b.path()));
        }

        #[test]
        fn rescan_picks_up_new_files() {
            let dir = tempfile::tempdir().unwrap();
            touch(dir.path(), "a.mp4");
            let session = Session::new();
            session.open(dir.path()).unwrap();
            touch(dir.path(), "b.mp4");
            session.rescan().unwrap();
            assert_eq!(rels(&session.search("", 100).unwrap()), ["a.mp4", "b.mp4"]);
        }

        #[test]
        fn search_before_opening_a_folder_is_an_error() {
            let session = Session::new();
            assert_eq!(session.search("", 100).unwrap_err(), NO_FOLDER);
        }

        #[test]
        fn rescan_before_opening_a_folder_is_an_error() {
            let session = Session::new();
            assert_eq!(session.rescan().unwrap_err(), NO_FOLDER);
        }

        #[test]
        fn opening_a_missing_folder_is_an_error_and_keeps_the_open_one() {
            let dir = tempfile::tempdir().unwrap();
            touch(dir.path(), "a.mp4");
            let session = Session::new();
            session.open(dir.path()).unwrap();
            let missing = dir.path().join("missing");
            assert!(session.open(&missing).is_err());
            assert_eq!(session.root().as_deref(), Some(dir.path()));
            assert_eq!(rels(&session.search("", 100).unwrap()), ["a.mp4"]);
        }
    }
}
