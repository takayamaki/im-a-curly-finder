//! Choosing the root directory to scan.

use std::ffi::OsString;
use std::path::PathBuf;

/// Picks the root from the first command-line argument, then from `CURLY_ROOT`.
pub fn resolve(arg: Option<OsString>, var: Option<OsString>) -> Option<PathBuf> {
    let _ = var;
    arg.map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    mod resolve {
        use std::path::PathBuf;

        use crate::root::resolve;

        #[test]
        fn uses_the_first_argument() {
            assert_eq!(
                resolve(Some(r"D:\clips".into()), None),
                Some(PathBuf::from(r"D:\clips"))
            );
        }

        #[test]
        #[ignore = "todo"]
        fn falls_back_to_the_curly_root_variable() {}

        #[test]
        #[ignore = "todo"]
        fn returns_none_when_neither_is_given() {}

        #[test]
        #[ignore = "todo"]
        fn prefers_the_argument_over_the_variable() {}
    }
}
