//! Extracts tags from paths.
//!
//! - A `{...}` in a folder or file name is a tag (files inherit the tags of their folders)
//! - Folder names outside `{}` are not tags. Put `{}` on a folder you want to match exactly

use std::ops::Range;

/// Returns the tags of a path relative to the root (`/`-separated), in order of appearance.
pub fn path_tags(rel: &str) -> Vec<String> {
    let mut tags: Vec<String> = rel.split('/').flat_map(brace_tags).collect();
    let mut seen = std::collections::HashSet::new();
    tags.retain(|tag| seen.insert(tag.clone()));
    tags
}

/// A problem with the braces in one folder or file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Issue {
    /// Full-width `｛` `｝`. They still make a tag, but should be half-width
    FullWidthBrace,
    /// An unclosed `{`
    Unclosed,
    /// A `}` without a matching `{`
    StrayClose,
    /// Nested `{...}`
    Nested,
    /// An empty `{}`
    Empty,
}

pub fn segment_issues(segment: &str) -> Vec<Issue> {
    scan_braces(segment).issues
}

fn brace_tags(segment: &str) -> Vec<String> {
    scan_braces(segment)
        .spans
        .into_iter()
        .map(|span| segment[span].trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

struct Braces {
    /// Byte ranges of the contents of `{...}` (without the braces)
    spans: Vec<Range<usize>>,
    issues: Vec<Issue>,
}

/// Full-width `｛｝` count the same as `{}` (lint asks to make them half-width).
/// Nested `{...}` are ignored along with the outer pair, and an unclosed `{` is ignored to the end.
fn scan_braces(segment: &str) -> Braces {
    let mut spans = Vec::new();
    let mut issues = Vec::new();
    if segment.contains(['｛', '｝']) {
        issues.push(Issue::FullWidthBrace);
    }
    let mut depth = 0;
    let mut open = 0;
    let mut nested = false;
    for (i, c) in segment.char_indices() {
        match c {
            '{' | '｛' => {
                if depth == 0 {
                    open = i + c.len_utf8();
                    nested = false;
                } else {
                    nested = true;
                }
                depth += 1;
            }
            '}' | '｝' if depth > 0 => {
                depth -= 1;
                if depth == 0 && nested {
                    issues.push(Issue::Nested);
                } else if depth == 0 {
                    if segment[open..i].trim().is_empty() {
                        issues.push(Issue::Empty);
                    }
                    spans.push(open..i);
                }
            }
            '}' | '｝' => issues.push(Issue::StrayClose),
            _ => {}
        }
    }
    if depth > 0 {
        issues.push(Issue::Unclosed);
    }
    Braces { spans, issues }
}

#[cfg(test)]
mod tests {
    mod path_tags {
        use crate::tags::path_tags;

        #[test]
        fn a_brace_in_the_file_name_becomes_a_tag() {
            assert_eq!(path_tags("game-a/opening{solo}.mp4"), ["solo"]);
        }

        #[test]
        fn a_brace_in_a_folder_name_is_inherited_by_the_files_below() {
            assert_eq!(
                path_tags("game-a{3D}/song{game-a}/opening{solo}.mp4"),
                ["3D", "game-a", "solo"]
            );
        }

        #[test]
        fn a_directory_name_outside_braces_is_not_a_tag() {
            assert!(path_tags("clips/game-a/MV/good morning.mp4").is_empty());
        }

        #[test]
        fn several_braces_in_one_name_become_several_tags() {
            assert_eq!(path_tags("song{3D}{solo}.mp4"), ["3D", "solo"]);
        }

        #[test]
        fn the_same_tag_on_a_folder_and_a_file_is_listed_once() {
            assert_eq!(path_tags("MV{3D}/song{3D}.mp4"), ["3D"]);
        }

        #[test]
        fn spaces_inside_braces_are_trimmed() {
            assert_eq!(path_tags("song{ 3D }.mp4"), ["3D"]);
        }

        #[test]
        fn full_width_braces_also_become_tags() {
            assert_eq!(path_tags("MV｛3D｝/song｛solo｝.mp4"), ["3D", "solo"]);
        }

        #[test]
        fn empty_braces_give_no_tag() {
            assert!(path_tags("song{}{ }.mp4").is_empty());
        }

        #[test]
        fn an_unclosed_brace_gives_no_tag() {
            assert!(path_tags("song{solo.mp4").is_empty());
            assert!(path_tags("song{so{lo}.mp4").is_empty());
        }

        #[test]
        fn nested_braces_give_no_tag() {
            assert_eq!(path_tags("song{so{lo}}{3D}.mp4"), ["3D"]);
        }
    }

    mod segment_issues {
        use crate::tags::{Issue, segment_issues};

        #[test]
        fn a_clean_name_has_no_issues() {
            assert!(segment_issues("opening{solo}{3D} [abcDEF12345].mp4").is_empty());
        }

        #[test]
        fn full_width_braces_are_reported() {
            assert_eq!(
                segment_issues("song｛solo｝｛3D｝.mp4"),
                [Issue::FullWidthBrace]
            );
        }

        #[test]
        fn an_unclosed_brace_is_reported() {
            assert_eq!(segment_issues("song{solo.mp4"), [Issue::Unclosed]);
        }

        #[test]
        fn a_stray_closing_brace_is_reported() {
            assert_eq!(segment_issues("songsolo}.mp4"), [Issue::StrayClose]);
        }

        #[test]
        fn nested_braces_are_reported() {
            assert_eq!(segment_issues("song{so{lo}}.mp4"), [Issue::Nested]);
        }

        #[test]
        fn empty_braces_are_reported() {
            assert_eq!(segment_issues("song{ }.mp4"), [Issue::Empty]);
        }
    }
}
