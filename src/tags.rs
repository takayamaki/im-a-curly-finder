//! Extracts tags from paths.
//!
//! - Each directory name in the path is a tag
//! - A `{...}` in a folder or file name is a tag (files inherit the tags of their folders)

/// Returns the tags of a path relative to the root (`/`-separated), in order of appearance.
pub fn path_tags(rel: &str) -> Vec<String> {
    let mut segments: Vec<&str> = rel.split('/').collect();
    let file_name = segments.pop().unwrap_or_default();
    let mut tags: Vec<String> = segments.into_iter().map(str::to_string).collect();
    tags.extend(brace_tags(file_name));
    tags
}

fn brace_tags(segment: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut open: Option<usize> = None;
    for (i, c) in segment.char_indices() {
        match c {
            '{' => open = Some(i + c.len_utf8()),
            '}' => {
                if let Some(start) = open.take() {
                    tags.push(segment[start..i].to_string());
                }
            }
            _ => {}
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    mod path_tags {
        use crate::tags::path_tags;

        #[test]
        fn each_directory_level_becomes_a_tag() {
            assert_eq!(
                path_tags("clips/game-a/MV/good morning.mp4"),
                ["clips", "game-a", "MV"]
            );
        }

        #[test]
        fn the_file_name_itself_is_not_a_level_tag() {
            assert!(path_tags("clip.mp4").is_empty());
        }

        #[test]
        fn a_brace_in_the_file_name_becomes_a_tag() {
            assert_eq!(path_tags("game-a/opening{solo}.mp4"), ["game-a", "solo"]);
        }

        #[test]
        #[ignore = "todo"]
        fn a_brace_in_a_folder_name_is_inherited_by_the_files_below() {}

        #[test]
        #[ignore = "todo"]
        fn several_braces_in_one_name_become_several_tags() {}

        #[test]
        #[ignore = "todo"]
        fn a_level_tag_leaves_out_the_braces_in_the_folder_name() {}

        #[test]
        #[ignore = "todo"]
        fn a_folder_named_only_with_braces_gives_no_level_tag() {}

        #[test]
        #[ignore = "todo"]
        fn the_same_tag_on_a_folder_and_a_file_is_listed_once() {}

        #[test]
        #[ignore = "todo"]
        fn spaces_inside_braces_are_trimmed() {}

        #[test]
        #[ignore = "todo"]
        fn full_width_braces_also_become_tags() {}

        #[test]
        #[ignore = "todo"]
        fn empty_braces_give_no_tag() {}

        #[test]
        #[ignore = "todo"]
        fn an_unclosed_brace_gives_no_tag() {}

        #[test]
        #[ignore = "todo"]
        fn nested_braces_give_no_tag() {}
    }

    mod segment_issues {
        #[test]
        #[ignore = "todo"]
        fn a_clean_name_has_no_issues() {}

        #[test]
        #[ignore = "todo"]
        fn full_width_braces_are_reported() {}

        #[test]
        #[ignore = "todo"]
        fn an_unclosed_brace_is_reported() {}

        #[test]
        #[ignore = "todo"]
        fn a_stray_closing_brace_is_reported() {}

        #[test]
        #[ignore = "todo"]
        fn nested_braces_are_reported() {}

        #[test]
        #[ignore = "todo"]
        fn empty_braces_are_reported() {}
    }
}
