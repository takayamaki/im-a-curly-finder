//! Settings kept between runs, such as the last folder opened from the GUI.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The folder last opened from the GUI
    pub root: Option<PathBuf>,
}

impl Settings {
    /// Reads the settings from `path`.
    pub fn load(path: &Path) -> Settings {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Settings::default();
        };
        serde_json::from_str(&text).unwrap()
    }

    /// Writes the settings to `path`.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, text)
    }
}

#[cfg(test)]
mod tests {
    mod settings {
        use std::path::PathBuf;

        use crate::settings::Settings;

        #[test]
        fn loads_the_root_that_was_saved() {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("settings.json");
            let settings = Settings {
                root: Some(PathBuf::from(r"D:\clips")),
            };
            settings.save(&file).unwrap();
            assert_eq!(Settings::load(&file), settings);
        }

        #[test]
        fn loads_no_root_when_the_file_does_not_exist() {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("settings.json");
            assert_eq!(Settings::load(&file), Settings::default());
        }

        #[test]
        fn creates_the_folder_of_the_file_when_saving() {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("config/curly/settings.json");
            let settings = Settings {
                root: Some(PathBuf::from(r"D:\clips")),
            };
            settings.save(&file).unwrap();
            assert_eq!(Settings::load(&file), settings);
        }

        #[test]
        #[ignore = "todo"]
        fn loads_no_root_when_the_file_is_broken() {}
    }
}
