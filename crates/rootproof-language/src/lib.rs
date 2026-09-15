pub mod adapter;
pub mod detection;
pub mod failure;
pub mod injection;
pub mod inspect;
pub mod reproduction;
pub mod rust;
pub mod source;

pub use adapter::LanguageAdapter;
pub use detection::detect_language;
pub use failure::parse_rust_failure;
pub use injection::inject_reproduction_test;
pub use inspect::inspect_repository;
pub use reproduction::{
    normalize_rust_reproduction_code, prepare_rust_reproduction_code,
    validate_rust_reproduction_code,
};
pub use rust::RustAdapter;
pub use source::{read_source_context, resolve_failure_file};
