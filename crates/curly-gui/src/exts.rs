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

/// The extensions to search for, given the checked ones: none (any file) when every extension of
/// the groups is checked. Extensions in no group are left out; the rest go in the order of the
/// groups.
pub fn request_exts(checked: &[String]) -> Vec<String> {
    let all = EXT_GROUPS.iter().flat_map(|g| g.exts);
    let kept: Vec<String> = all
        .clone()
        .filter(|e| checked.iter().any(|c| c == *e))
        .map(|e| e.to_string())
        .collect();
    if kept.len() == all.count() {
        return Vec::new();
    }
    kept
}

#[cfg(test)]
mod tests {
    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    mod request_exts {
        use super::strings;
        use crate::exts::{EXT_GROUPS, request_exts};

        #[test]
        fn sends_the_checked_extensions() {
            assert_eq!(request_exts(&strings(&["mp4", "png"])), ["mp4", "png"]);
        }

        #[test]
        fn sends_nothing_when_every_extension_is_checked() {
            // Any file then, also those of extensions in no group (.txt etc.).
            let every: Vec<String> = EXT_GROUPS
                .iter()
                .flat_map(|g| g.exts.iter().map(|e| e.to_string()))
                .collect();
            assert!(request_exts(&every).is_empty());
        }

        #[test]
        fn sends_nothing_when_nothing_is_checked() {
            // An empty result would never be wanted: nothing checked is taken as any file.
            assert!(request_exts(&[]).is_empty());
        }

        #[test]
        fn leaves_out_extensions_in_no_group() {
            // From a settings.json written by hand, or a group changed later.
            assert_eq!(request_exts(&strings(&["txt", "mp4", "MP4"])), ["mp4"]);
        }
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
}
