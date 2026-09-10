pub mod adapter;
pub mod detection;
pub mod failure;
pub mod inspect;
pub mod rust;

pub use adapter::LanguageAdapter;
pub use detection::detect_language;
pub use failure::parse_rust_failure;
pub use inspect::inspect_repository;
pub use rust::RustAdapter;
