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
    /// Writes the layout-calculated string `str` with the font `font`.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't render the writing.
    fn write(
        &mut self,
        font: font::Spec<Font::Id, Fg::Id>,
        str: &font::layout::String,
    ) -> error::Result<()>;

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

/// Enumeration of rendering commands.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Command<FontId, FgId, BgId> {
    /// Represents a `write` command.
    Write(font::Spec<FontId, FgId>, font::layout::String),
    /// Represents a `fill` command.
    Fill(metrics::Rect, BgId),
    /// Represents a `clear` command.
    Clear(BgId),
    /// Represents a `present` command.
    Present,
}

/// A renderer that just logs rendering commands, rather than executing them.
///
/// Useful for testing.
#[derive(Debug, Clone)]
pub struct Logger<
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
> {
    /// Log of commands requested on this renderer.
    pub log: Vec<Command<Font::Id, Fg::Id, Bg::Id>>,
    /// Metrics map for the renderer.
    pub metrics: Font::MetricsMap,
}

impl<Font, Fg, Bg> Renderer<Font, Fg, Bg> for Logger<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    fn write(
        &mut self,
        font: font::Spec<Font::Id, Fg::Id>,
        str: &font::layout::String,
    ) -> crate::Result<()> {
        self.log.push(Command::Write(font, str.clone()));
        Ok(())
    }

    fn fill(&mut self, rect: metrics::Rect, colour: Bg::Id) -> crate::Result<()> {
        self.log.push(Command::Fill(rect, colour));
        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> error::Result<()> {
        self.log.push(Command::Clear(colour));
        Ok(())
    }

    fn present(&mut self) {
        self.log.push(Command::Present);
    }

    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.metrics
    }
}
