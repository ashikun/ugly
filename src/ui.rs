//! Tools for building a UI on top of ugly.
//!
//! The `ugly` UI is primarily retained-mode: it consists of a widget tree that one constructs
//! statically, then manipulates in three phases: layout, which occurs whenever the widget tree
//! changes or the window dimensions change; update, which occurs whenever the UI state model
//! changes; and render, which occurs whenever the UI is dirty.
//!
//! We do not yet handle input, but keyboard and possibly eventually mouse input are on the table.

pub mod layout;
pub mod render;
pub mod update;
pub mod widgets;

pub use layout::Layoutable;
pub use render::Renderable;
pub use update::Updatable;
