//! A dummy renderer ([Logger]) that just logs [Command]s without rendering anything.

use std::hash::Hash;

use crate::{error, font, metrics};

use super::Renderer;

/// Enumeration of rendering commands.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Command<FontId, FgId, BgId> {
    /// Represents a `write` command.
    Write(FontId, FgId, font::layout::String),
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
#[derive(Debug, Default, Clone)]
pub struct Logger<FontId, FgId, BgId> {
    /// Log of commands requested on this renderer.
    pub log: Vec<Command<FontId, FgId, BgId>>,
}

impl<FontId, FgId, BgId> Renderer<'static, FontId, FgId, BgId> for Logger<FontId, FgId, BgId>
where
    FontId: Default + Eq + Hash + Copy + Clone,
{
    fn write(&mut self, font: FontId, fg: FgId, str: &font::layout::String) -> crate::Result<()> {
        self.log.push(Command::Write(font, fg, str.clone()));
        Ok(())
    }

    fn fill(&mut self, rect: metrics::Rect, colour: BgId) -> crate::Result<()> {
        self.log.push(Command::Fill(rect, colour));
        Ok(())
    }

    fn clear(&mut self, colour: BgId) -> error::Result<()> {
        self.log.push(Command::Clear(colour));
        Ok(())
    }

    fn present(&mut self) {
        self.log.push(Command::Present);
    }
}
