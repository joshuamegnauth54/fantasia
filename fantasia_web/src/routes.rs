//! Fantasia's endpoints.

pub mod health;
pub mod index;

pub use health::health_check;
pub use index::index;
