//! Fantasia's endpoints.

pub mod fallback_404;
pub mod health;
pub mod index;

pub use fallback_404::fallback_404;
pub use health::health_check;
pub use index::index;
