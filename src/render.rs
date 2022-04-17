//! Traits for low-level rendering.

use super::{colour, error, font, metrics, resource};

/// Trait of things that provide rendering facilities.
///
/// The trait is parameterised by the specific maps used to look up font metrics and colours
/// in the application.  Background colours may resolve to `None` (transparency).
pub trait Renderer<
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
>
{
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
        font: font::Spec<Font::Id, Fg::Id>,
        s: &str,
    ) -> error::Result<metrics::Point>;

    /// Fills the rectangle `rect`, whose top-left is positioned relative to
    /// the current position, with the background colour `bg`.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer fails to blit the rect onto the screen.
    fn fill(&mut self, rect: metrics::Rect, colour: Bg::Id) -> error::Result<()>;

    // TODO(@MattWindsor91): replace these with RAII

    /// Clears the screen to the given background colour.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer fails to clear the screen.
    fn clear(&mut self, colour: Bg::Id) -> error::Result<()>;

    /// Refreshes the screen.
    fn present(&mut self);

    /// Borrows the font metrics map being used by this renderer.
    fn font_metrics(&self) -> &Font::MetricsMap;
}
