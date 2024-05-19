use std::collections::HashMap;

use futures::future::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::ActiveEventLoop,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowId,
};

use ugly::{
    backends::wgpu::Core,
    colour::Ega,
    font,
    metrics::Rect,
    resource::Map,
    ui::{widgets::Label, Layoutable, Renderable},
    Renderer,
};

const WIN_WIDTH: u32 = 640;
const WIN_HEIGHT: u32 = 480;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The text to display on the font viewer.
    #[arg(short, long)]
    text: String,

    /// Directory of font to load
    #[arg(short, long, default_value = "../assets/fonts/medium")]
    font: std::path::PathBuf,
}

struct App {
    args: Args,
    context: Option<Context>,
}

#[ouroboros::self_referencing]
struct Context {
    window: Window,
    #[covariant]
    #[borrows(window)]
    renderer: ugly::backends::wgpu::Renderer<'this, FontMap, Ega, Ega>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = PhysicalSize::new(WIN_WIDTH, WIN_HEIGHT);
        let attributes = Window::default_attributes().with_inner_size(size);

        let window = event_loop.create_window(attributes).unwrap();

        let fonts = get_fonts(&self.args.font);
        let resources =
            ugly::resource::Set::new(fonts, ugly::colour::EGA, ugly::colour::EGA).unwrap();

        let ctx_builder = ContextAsyncTryBuilder {
            window,
            renderer_builder: |w| {
                Core::new(w)
                    .map(|core| -> anyhow::Result<_> {
                        let core = core?;
                        Ok(ugly::backends::wgpu::Renderer::from_core(core, resources))
                    })
                    .boxed()
            },
        };

        let ctx = pollster::block_on(ctx_builder.try_build()).unwrap();

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

                ctx.with_renderer_mut(|r| r.with_core_mut(|c| c.resize(new_size)));
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
    fn new(args: Args) -> Self {
        Self {
            args,
            context: None,
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        use ugly::colour::ega;

        let Some(ctx) = &mut self.context else {
            return Ok(());
        };

        let colours = [
            ega::Id::BRIGHT_RED,
            ega::Id::BRIGHT_YELLOW,
            ega::Id::BRIGHT_GREEN,
            ega::Id::BRIGHT_CYAN,
            ega::Id::BRIGHT_BLUE,
            ega::Id::BRIGHT_MAGENTA,
            ega::Id::BRIGHT_BLACK,
            ega::Id::BRIGHT_WHITE,
        ];

        let metrics = ctx.borrow_renderer().font_metrics();
        let font_height = metrics.get(0).padded_h();

        let mut labels: [Label<FontMap, Ega, Ega>; 8] = std::array::from_fn(|i| {
            let mut label = Label::new(font::Spec {
                id: 0,
                colour: colours[i],
            });

            label.layout(
                metrics,
                Rect::new(
                    5,
                    5 + (i as i32) * font_height,
                    WIN_WIDTH as i32,
                    font_height,
                ),
            );

            label
        });

        for label in &mut labels {
            label.update_display(metrics, &self.args.text);
        }

        ctx.with_renderer_mut(|ren| {
            ren.clear(ega::Id::Dark(ega::BaseId::Cyan))?;

            for label in &labels {
                label.render(ren)?;
            }

            ren.present();

            Ok(())
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(args);
    event_loop.run_app(&mut app)?;

    Ok(())
}

type FontMap = ugly::resource::DefaultingHashMap<usize, ugly::Font>;

fn get_fonts(path: &std::path::Path) -> ugly::resource::DefaultingHashMap<usize, ugly::Font> {
    let font = font::Font::from_dir(path);

    let mut map: HashMap<usize, _> = HashMap::new();
    map.insert(0, font.clone());

    ugly::resource::DefaultingHashMap::new(map, font)
}
