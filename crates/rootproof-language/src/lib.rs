pub mod adapter;
pub mod detection;
pub mod failure;
pub mod inspect;
pub mod rust;
pub mod source;

pub use adapter::LanguageAdapter;
pub use detection::detect_language;
pub use failure::parse_rust_failure;
pub use inspect::inspect_repository;
pub use rust::RustAdapter;
pub use source::{read_source_context, resolve_failure_file};
