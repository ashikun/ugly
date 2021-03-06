//! A dummy renderer ([Logger]) that just logs [Command]s without rendering anything.

use super::{
    super::{colour, error, font, metrics},
    Renderer,
};
use crate::resource::Map;

/// Enumeration of rendering commands.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Command<FontId, FgId, BgId> {
    /// Represents a `load_font` command (both the input spec and the output index).
    LoadFont(font::Spec<FontId, FgId>, font::Index),
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
pub struct Logger<Font: font::Map, Fg: Map<colour::Definition>, Bg: Map<colour::Definition>> {
    /// Log of commands requested on this renderer.
    pub log: Vec<Command<Font::Id, Fg::Id, Bg::Id>>,
    /// Metrics map for the renderer.
    pub metrics: Font::MetricsMap,
    /// Mock font cache.
    pub fonts: std::collections::HashMap<font::Spec<Font::Id, Fg::Id>, font::Index>,
}

impl<Font, Fg, Bg> Logger<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    pub fn new(metrics: Font::MetricsMap) -> Self {
        Self {
            metrics,
            log: Vec::new(),
            fonts: std::collections::HashMap::new(),
        }
    }
}

impl<Font, Fg, Bg> Renderer<'static, Font, Fg, Bg> for Logger<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.metrics
    }

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
}
