//! Traits for low-level rendering.

use super::{error, font, metrics};

/// Trait of things that provide rendering facilities.
///
/// The trait is parameterised by the specific IDs used to look up font, foreground, and background
/// colours in the application.
pub trait Renderer<FId, Fg, Bg> {
    /// Writes the string `s` at position `pos` with the font `font`.
    ///
    /// Returns the position that the next character would be written to, if we continued writing.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't render the writing.
    fn write(
        &mut self,
        pos: metrics::Point,
        font: font::Spec<FId, Fg>,
        s: &str,
    ) -> error::Result<metrics::Point>;

    /// Fills the rectangle `rect`, whose top-left is positioned relative to
    /// the current position, with the background colour `bg`.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer fails to blit the rect onto the screen.
    fn fill(&mut self, rect: metrics::Rect, colour: Bg) -> error::Result<()>;

    // TODO(@MattWindsor91): replace these with RAII

    /// Clears the screen to the given background colour.
    fn clear(&mut self, colour: Bg) -> error::Result<()>;

    /// Refreshes the screen.
    fn present(&mut self);

    // TODO(@MattWindsor91): make the below obsolete?

    /// Borrows the font metrics map.
    fn font_metrics(&self) -> &font::metrics::Map<FId>;
}
