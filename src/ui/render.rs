//! The [Renderable] trait.

/// Trait for things that can render their state to a renderer.
pub trait Renderable<R: ?Sized> {
    /// Renders this item onto `r`.
    ///
    /// # Errors
    ///
    /// Fails if the underlying renderer can't render what is sent to it.
    fn render(&self, r: &mut R) -> crate::Result<()>;
}
