//! Extracts tags from paths.
//!
//! - Each directory name in the path is a tag
//! - A `{...}` in a folder or file name is a tag (files inherit the tags of their folders)

use std::ops::Range;

/// Returns the tags of a path relative to the root (`/`-separated), in order of appearance.
pub fn path_tags(rel: &str) -> Vec<String> {
    let mut segments: Vec<&str> = rel.split('/').collect();
    let file_name = segments.pop().unwrap_or_default();
    let mut tags = Vec::new();
    for folder in segments {
        let level = level_name(folder);
        if !level.is_empty() {
            tags.push(level);
        }
        tags.extend(brace_tags(folder));
    }
    tags.extend(brace_tags(file_name));
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

/// A directory name as a tag: `{...}` removed and surrounding spaces trimmed.
fn level_name(folder: &str) -> String {
    let mut name = String::new();
    let mut rest = 0;
    for span in brace_spans(folder) {
        name.push_str(&folder[rest..span.outer.start]);
        rest = span.outer.end;
    }
    name.push_str(&folder[rest..]);
    name.trim().to_string()
}

fn brace_tags(segment: &str) -> Vec<String> {
    brace_spans(segment)
        .into_iter()
        .map(|span| segment[span.inner].trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

/// Byte ranges of a `{...}`. `outer` includes the braces and `inner` does not.
struct BraceSpan {
    outer: Range<usize>,
    inner: Range<usize>,
}

fn brace_spans(segment: &str) -> Vec<BraceSpan> {
    scan_braces(segment).spans
}

struct Braces {
    spans: Vec<BraceSpan>,
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
    let mut open = (0, 0);
    let mut nested = false;
    for (i, c) in segment.char_indices() {
        match c {
            '{' | '｛' => {
                if depth == 0 {
                    open = (i, i + c.len_utf8());
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
                    if segment[open.1..i].trim().is_empty() {
                        issues.push(Issue::Empty);
                    }
                    spans.push(BraceSpan {
                        outer: open.0..i + c.len_utf8(),
                        inner: open.1..i,
                    });
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
        fn a_brace_in_a_folder_name_is_inherited_by_the_files_below() {
            let tags = path_tags("game-a{3D}/song/opening.mp4");
            assert!(tags.contains(&"3D".to_string()), "{tags:?}");
        }

        #[test]
        fn several_braces_in_one_name_become_several_tags() {
            assert_eq!(path_tags("song{3D}{solo}.mp4"), ["3D", "solo"]);
        }

        #[test]
        fn a_level_tag_leaves_out_the_braces_in_the_folder_name() {
            assert_eq!(path_tags("live {3D}/good morning.mp4"), ["live", "3D"]);
        }

        #[test]
        fn a_folder_named_only_with_braces_gives_no_level_tag() {
            assert_eq!(
                path_tags("{self-shot}/250926/song.mp4"),
                ["self-shot", "250926"]
            );
        }

        #[test]
        fn the_same_tag_on_a_folder_and_a_file_is_listed_once() {
            assert_eq!(path_tags("MV{3D}/song{3D}.mp4"), ["MV", "3D"]);
        }

        #[test]
        fn spaces_inside_braces_are_trimmed() {
            assert_eq!(path_tags("song{ 3D }.mp4"), ["3D"]);
        }

        #[test]
        fn full_width_braces_also_become_tags() {
            assert_eq!(path_tags("MV｛3D｝/song｛solo｝.mp4"), ["MV", "3D", "solo"]);
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
