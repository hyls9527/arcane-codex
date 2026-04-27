// Data models module

pub mod image;
pub mod task;

// Re-export commonly used types
pub use image::{Image, AIResult, SearchResult};
pub use task::Task;
