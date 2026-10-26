//! The extension drop-down: groups of extensions with a check box each, and what is searched.

use serde::Serialize;

/// A group of extensions in the drop-down (lower case, without the `.`).
#[derive(Debug, Serialize)]
pub struct ExtGroup {
    pub name: &'static str,
    pub exts: &'static [&'static str],
}

/// The groups, in the order shown.
pub const EXT_GROUPS: &[ExtGroup] = &[
    ExtGroup {
        name: "Video",
        exts: &["mp4", "mov", "flv", "mkv", "webm", "avi", "wmv", "m4v"],
    },
    ExtGroup {
        name: "Images",
        exts: &["png", "jpg", "jpeg", "gif", "bmp", "webp"],
    },
    ExtGroup {
        name: "Audio",
        exts: &["m4a", "mp3", "wav", "flac", "aac", "ogg"],
    },
];

/// The extensions to search for: those of the groups and `others` that are not `unchecked`.
pub fn request_exts(unchecked: &[String], others: &[String]) -> Vec<String> {
    EXT_GROUPS
        .iter()
        .flat_map(|g| g.exts.iter().map(|e| e.to_string()))
        .chain(others.iter().cloned())
        .filter(|e| !unchecked.contains(e))
        .collect()
}

#[cfg(test)]
mod tests {
    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    mod ext_groups {
        use crate::exts::EXT_GROUPS;

        #[test]
        fn are_videos_images_and_audio() {
            let names: Vec<&str> = EXT_GROUPS.iter().map(|g| g.name).collect();
            assert_eq!(names, ["Video", "Images", "Audio"]);
            assert_eq!(
                EXT_GROUPS[0].exts,
                ["mp4", "mov", "flv", "mkv", "webm", "avi", "wmv", "m4v"]
            );
        }
    }

    // Others: the extensions of the open folder in no group, and the unchecked ones kept instead of
    // the checked ones (so that an extension new to the folder starts checked).
    mod request_exts_with_others {
        use super::strings;
        use crate::exts::request_exts;

        #[test]
        fn sends_the_extensions_left_checked() {
            let sent = request_exts(&strings(&["mov", "png", "jpg"]), &strings(&["txt"]));
            assert_eq!(
                sent,
                [
                    "mp4", "flv", "mkv", "webm", "avi", "wmv", "m4v", "jpeg", "gif", "bmp", "webp",
                    "m4a", "mp3", "wav", "flac", "aac", "ogg", "txt"
                ]
            );
        }

        #[test]
        #[ignore]
        fn sends_nothing_when_nothing_is_unchecked() {}

        #[test]
        #[ignore]
        fn sends_nothing_when_everything_is_unchecked() {}

        #[test]
        #[ignore]
        fn sends_the_other_extensions_left_checked() {}

        #[test]
        #[ignore]
        fn ignores_unchecked_extensions_that_are_not_offered() {}
    }

    mod other_exts {
        #[test]
        #[ignore]
        fn are_the_extensions_of_the_folder_in_no_group() {}

        #[test]
        #[ignore]
        fn put_files_without_an_extension_last() {}
    }
}
