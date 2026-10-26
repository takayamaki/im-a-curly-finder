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

/// The extensions to search for, given the checked ones.
pub fn request_exts(checked: &[String]) -> Vec<String> {
    checked.to_vec()
}

#[cfg(test)]
mod tests {
    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    mod request_exts {
        use super::strings;
        use crate::exts::request_exts;

        #[test]
        fn sends_the_checked_extensions() {
            assert_eq!(request_exts(&strings(&["mp4", "png"])), ["mp4", "png"]);
        }

        #[test]
        #[ignore]
        fn sends_nothing_when_every_extension_is_checked() {}

        #[test]
        #[ignore]
        fn sends_nothing_when_nothing_is_checked() {}

        #[test]
        #[ignore]
        fn leaves_out_extensions_in_no_group() {}
    }

    mod ext_groups {
        #[test]
        #[ignore]
        fn are_videos_images_and_audio() {}
    }
}
