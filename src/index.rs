//! The index. Built in memory by scanning the directory at startup.

#[cfg(test)]
mod tests {
    mod scan {
        #[test]
        #[ignore = "todo"]
        fn lists_every_file_under_the_root() {}

        #[test]
        #[ignore = "todo"]
        fn splits_each_name_into_dir_stem_and_extension() {}
    }

    mod query {
        #[test]
        #[ignore = "todo"]
        fn returns_the_matching_entries_in_path_order() {}

        #[test]
        #[ignore = "todo"]
        fn returns_the_parse_error_for_a_broken_query() {}
    }

    mod tags {
        #[test]
        #[ignore = "todo"]
        fn counts_files_per_tag_most_used_first() {}

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
