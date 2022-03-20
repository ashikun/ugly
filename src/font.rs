//! Fonts, their metrics, and ways of loading and referring to them.

pub mod error;
pub mod metrics;
pub mod path;
pub mod spec;

pub use error::{Error, Result};
pub use metrics::Metrics;
pub use path::Path;
pub use spec::{Id, Spec};
