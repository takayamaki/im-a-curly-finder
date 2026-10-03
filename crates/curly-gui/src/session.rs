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
        use crate::session::Session;

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
        #[ignore = "todo"]
        fn opening_another_folder_replaces_the_index() {}

        #[test]
        #[ignore = "todo"]
        fn rescan_picks_up_new_files() {}

        #[test]
        #[ignore = "todo"]
        fn search_before_opening_a_folder_is_an_error() {}

        #[test]
        #[ignore = "todo"]
        fn rescan_before_opening_a_folder_is_an_error() {}

        #[test]
        #[ignore = "todo"]
        fn opening_a_missing_folder_is_an_error_and_keeps_the_open_one() {}
    }
}
