//! `curly serve`: answers one JSON request per line, so the index is scanned only once.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn query_returns_the_matching_entries_and_the_total() {}

    #[test]
    #[ignore]
    fn query_echoes_the_id() {}

    #[test]
    #[ignore]
    fn query_cuts_the_entries_at_the_limit_but_counts_them_all() {}

    #[test]
    #[ignore]
    fn tags_returns_the_tag_counts() {}

    #[test]
    #[ignore]
    fn rescan_picks_up_a_file_added_since_the_start() {}

    #[test]
    #[ignore]
    fn ping_answers_ok() {}

    #[test]
    #[ignore]
    fn a_broken_query_answers_the_parse_error() {}

    #[test]
    #[ignore]
    fn a_query_without_q_answers_an_error() {}

    #[test]
    #[ignore]
    fn an_unknown_op_answers_an_error_with_the_id() {}

    #[test]
    #[ignore]
    fn invalid_json_answers_an_error_with_a_null_id() {}

    #[test]
    #[ignore]
    fn a_limit_of_zero_returns_no_entries_but_the_total() {}

    #[test]
    #[ignore]
    fn a_rescan_that_fails_answers_an_error_and_keeps_the_old_index() {}
}
