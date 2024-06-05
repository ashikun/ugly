//! `winit` integration for the `wgpu` renderer.

use std::sync::Arc;

use winit::window::Window;

use crate::{colour, font, resource, Result};

use super::Renderer;

/// An adapter for feeding `winit` events into `ugly` and `wgpu`.
pub struct Adapter<Font, Fg, Bg>
where
    Font: font::Map,
{
    inner: Option<Inner<Font, Fg, Bg>>,
}
impl<Font, Fg, Bg> Default for Adapter<Font, Fg, Bg>
where
    Font: font::Map,
{
    fn default() -> Self {
        Self { inner: None }
    }
}

impl<Font, Fg, Bg> Adapter<Font, Fg, Bg>
where
    Font: font::Map,
{
    /// Borrows the renderer mutably, if it is open.
    pub fn renderer_mut(&mut self) -> Option<&mut Renderer<Font, Fg, Bg>> {
        self.inner.as_mut().map(|i| &mut i.renderer)
    }

    /// Borrows the window immutably, if it is open.
    pub fn window(&mut self) -> Option<&Window> {
        self.inner.as_ref().map(|i| &*i.window)
    }

    /// Propagates a resize to the renderer.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.on_core(|c| c.resize(new_size));
    }

    /// Propagates a rescale to the renderer.
    pub fn rescale(&mut self, new_scale: f64) {
        self.on_core(|c| c.rescale(new_scale as f32));
    }

    /// Closes any open window.
    pub fn close(&mut self) {
        self.inner = None;
    }

    fn on_core(&mut self, mut f: impl FnMut(&mut super::Core)) {
        let Some(ref mut inner) = self.inner else {
            return;
        };

        f(&mut inner.renderer.core);
        inner.window.request_redraw();
    }
}

impl<Font, Fg, Bg> Adapter<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    /// Sets up the adapter for the given window.
    ///
    /// # Errors
    ///
    /// Fails if we can't initialise `wgpu`.
    pub async fn resume(
        &mut self,
        window: Window,
        resources: resource::Set<Font, Fg, Bg>,
    ) -> Result<()> {
        let window = Arc::new(window);
        let renderer = Renderer::from_window(window.clone(), resources).await?;

        let inner = Inner { window, renderer };
        inner.window.request_redraw();

        self.inner = Some(inner);

        Ok(())
    }
}

struct Inner<Font, Fg, Bg>
where
    Font: font::Map,
{
    window: Arc<Window>,
    renderer: Renderer<Font, Fg, Bg>,
}
