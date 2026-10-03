//! Choosing the root directory to scan.

use std::ffi::OsString;
use std::path::PathBuf;

/// Picks the root from the first command-line argument, then from `CURLY_ROOT`,
/// then from the folder last opened in the GUI.
pub fn resolve(
    arg: Option<OsString>,
    var: Option<OsString>,
    saved: Option<PathBuf>,
) -> Option<PathBuf> {
    arg.or(var).map(PathBuf::from).or(saved)
}

#[cfg(test)]
mod tests {
    mod resolve {
        use std::path::PathBuf;

        use crate::root::resolve;

        #[test]
        fn uses_the_first_argument() {
            assert_eq!(
                resolve(Some(r"D:\clips".into()), None, None),
                Some(PathBuf::from(r"D:\clips"))
            );
        }

        #[test]
        fn falls_back_to_the_curly_root_variable() {
            assert_eq!(
                resolve(None, Some(r"D:\clips".into()), None),
                Some(PathBuf::from(r"D:\clips"))
            );
        }

        #[test]
        fn falls_back_to_the_saved_folder() {
            assert_eq!(
                resolve(None, None, Some(PathBuf::from(r"D:\clips"))),
                Some(PathBuf::from(r"D:\clips"))
            );
        }

        #[test]
        fn returns_none_when_nothing_is_given() {
            assert_eq!(resolve(None, None, None), None);
        }

        #[test]
        fn prefers_the_argument_over_the_variable() {
            assert_eq!(
                resolve(Some(r"D:\a".into()), Some(r"D:\b".into()), None),
                Some(PathBuf::from(r"D:\a"))
            );
        }

        #[test]
        #[ignore = "todo"]
        fn prefers_the_variable_over_the_saved_folder() {}
    }
}
