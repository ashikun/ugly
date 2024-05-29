//! Traits for low-level rendering.

pub mod logger;

use super::{error, font, metrics};

/// Trait of things that provide rendering facilities.
///
/// The trait is parameterised by the specific maps used to look up font metrics and colours
/// in the application.
///
/// The lifetime `'f` captures any lifetime constraints on font data.
pub trait Renderer<'f, FontId, FgId, BgId> {
    /// Writes the layout-calculated string `str` with the font `font` and foreground colour `fg`.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't render the writing.
    fn write(&mut self, font: FontId, fg: FgId, str: &font::layout::String) -> error::Result<()>;

    /// Fills the rectangle `rect`, whose top-left is positioned relative to
    /// the current position, with the background colour `bg`.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer fails to blit the rect onto the screen.
    fn fill(&mut self, rect: metrics::Rect, colour: BgId) -> error::Result<()>;

    // TODO(@MattWindsor91): replace these with RAII

    /// Clears the screen to the given background colour.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer fails to clear the screen.
    fn clear(&mut self, colour: BgId) -> error::Result<()>;

    /// Refreshes the screen.
    fn present(&mut self);
}
