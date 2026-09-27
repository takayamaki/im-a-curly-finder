//! The search engine: find files by the tags written in their paths and names.
//!
//! Tags live only in paths and file names. There is no external database or dictionary file.
//! How to classify files is up to the data (folder names and file names).

pub mod index;
pub mod normalize;
pub mod query;
pub mod tags;
