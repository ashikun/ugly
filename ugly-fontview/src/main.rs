use std::collections::HashMap;

use pollster::FutureExt;
use ugly::backends::wgpu::render::Core;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use ugly::metrics::Rect;
use ugly::resource::Map;
use ugly::ui::{Layoutable, Renderable};
use ugly::{font, Renderer};

const WIN_WIDTH: u32 = 640;
const WIN_HEIGHT: u32 = 480;

#[derive(Default)]
struct App {
    context: Option<Context>,
}

#[ouroboros::self_referencing]
struct Context {
    window: Window,
    #[covariant]
    #[borrows(window)]
    renderer: ugly::backends::wgpu::Renderer<'this, FontMap, ugly::colour::Ega, ugly::colour::Ega>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = PhysicalSize::new(WIN_WIDTH, WIN_HEIGHT);
        let attributes = Window::default_attributes().with_inner_size(size);

        let window = event_loop.create_window(attributes).unwrap();

        let fonts = get_fonts();

        let palette = ugly::colour::Palette {
            fg: ugly::colour::EGA,
            bg: ugly::colour::EGA,
        };

        let ctx = ContextBuilder {
            window,
            renderer_builder: |w| {
                let core = Core::new(w).block_on().unwrap();
                ugly::backends::wgpu::Renderer::new(fonts, palette, core).unwrap()
            },
        }
        .build();

        ctx.borrow_window().request_redraw();

        self.context = Some(ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                let _ = self.context.take();
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                let Some(ctx) = &mut self.context else {
                    return;
                };

                ctx.with_renderer_mut(|r| r.core.resize(new_size));
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                self.render().unwrap();

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                if let Some(ref ctx) = self.context {
                    ctx.borrow_window().request_redraw();
                }
            }
            _ => (),
        }
    }
}

impl App {
    fn render(&mut self) -> anyhow::Result<()> {
        let Some(ctx) = &mut self.context else {
            return Ok(());
        };

        ctx.with_renderer_mut(|ren| {
            use ugly::colour::ega;

            ren.clear(ega::Id::Dark(ega::BaseId::Cyan))?;
            ren.fill(Rect::new(32, 16, 16, 32), ega::Id::Dark(ega::BaseId::White))?;
            ren.fill(Rect::new(0, 0, 32, 16), ega::Id::Dark(ega::BaseId::Red))?;
            ren.present();

            Ok(())
        })
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())

    /*

    let gfx = ugly::backends::sdl::Manager::new(window, &fonts, &colours)?;
    let mut ren = gfx.renderer()?;

    let colours = [
        ega::Id::Bright(ega::BaseId::Black),
        ega::Id::Bright(ega::BaseId::Red),
        ega::Id::Bright(ega::BaseId::Green),
        ega::Id::Bright(ega::BaseId::Yellow),
        ega::Id::Bright(ega::BaseId::Blue),
        ega::Id::Bright(ega::BaseId::Magenta),
        ega::Id::Bright(ega::BaseId::Cyan),
        ega::Id::Bright(ega::BaseId::White),
    ];

    let font_height = metrics.get(0).padded_h();

    let mut labels: [Label<FontMap, Ega, Ega>; 8] = std::array::from_fn(|i| {
        let mut label = Label::new(font::Spec {
            id: 0,
            colour: colours[i],
        });

        label.layout(
            &metrics,
            Rect::new(0, (i as i32) * font_height, WIN_WIDTH as i32, font_height),
        );

        label
    });

    'running: loop {
        for ev in event.poll_iter() {
            match ev {
                Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => break 'running,
                Event::Quit { .. } => break 'running,
                _ => (),
            }
        }

        ren.clear(ega::Id::Dark(ega::BaseId::Black))?;

        for label in &mut labels {
            label.update_display(
                &metrics,
                "The quick brown fox jumps over the lazy dog. 0123456789",
            );
        }

        for label in &mut labels {
            label.render(&mut ren)?;
        }

        ren.present();
    }

    std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

    Ok(())
    */
}

type FontMap = ugly::resource::DefaultingHashMap<usize, ugly::Font>;

fn get_fonts() -> ugly::resource::DefaultingHashMap<usize, ugly::Font> {
    let font = font::Font::from_dir("../assets/fonts/medium");

    let mut map: HashMap<usize, _> = HashMap::new();
    map.insert(0, font.clone());

    ugly::resource::DefaultingHashMap::new(map, font)
}

/*
#[derive(Debug, Error)]
enum Error {
    #[error("SDL init error: {0}")]
    Init(String),
    #[error("SDL window build error: {0}")]
    Window(sdl2::video::WindowBuildError),
}
 */
