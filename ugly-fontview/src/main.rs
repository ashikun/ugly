use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::*,
    event_loop::ActiveEventLoop,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowId,
};

use ugly::{
    backends, colour,
    colour::Ega,
    font,
    metrics::Rect,
    resource::{self, Map},
    text::Writer,
    ui::{
        layout::{Boundable, LayoutContext},
        widgets::Label,
        Layoutable, Renderable, Updatable,
    },
    Renderer,
};

const WIN_WIDTH: u32 = 640;
const WIN_HEIGHT: u32 = 480;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Literal text to display on the font viewer.
    #[arg(short, long, group = "input")]
    text: Option<String>,

    /// A file to display on the font viewer.
    #[arg(short = 'f', long, group = "input")]
    text_file: Option<PathBuf>,

    /// Directory of font to load
    #[arg(short = 'F', long, default_value = "../assets/fonts/medium")]
    font: PathBuf,
}

struct App {
    args: Args,
    context: backends::wgpu::winit::Adapter<FontMap, Ega, Ega>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(WIN_WIDTH, WIN_HEIGHT);
        let attributes = Window::default_attributes().with_inner_size(size);

        let window = event_loop.create_window(attributes).unwrap();

        let fonts = get_fonts(&self.args.font);
        let resources = resource::Set::new(fonts, colour::EGA, colour::EGA).unwrap();

        let adapter_fut = self.context.resume(window, resources);
        pollster::block_on(adapter_fut).unwrap();

        if let Some(w) = self.context.window() {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.context.close();
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.context.resize(new_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.context.rescale(scale_factor);
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                self.render().unwrap();
            }
            _ => (),
        }
    }
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            args,
            context: backends::wgpu::winit::Adapter::default(),
        }
    }

    fn render(&mut self) -> anyhow::Result<()> {
        use ugly::colour::ega;

        let Some(ren) = self.context.renderer_mut() else {
            return Ok(());
        };

        let colours = [
            ega::BaseId::Red,
            ega::BaseId::Yellow,
            ega::BaseId::Green,
            ega::BaseId::Cyan,
            ega::BaseId::Blue,
            ega::BaseId::Magenta,
            ega::BaseId::Black,
            ega::BaseId::White,
        ];

        let metrics = ren.font_metrics();
        let font_height = metrics.get(0).padded_h();

        let mut labels: [Label<_, _, _>; 8] = std::array::from_fn(|i| {
            let writer = Writer::new(0, ega::Id::Bright(colours[i]));
            let mut label = Label::new(writer);
            label.set_bg(ega::Id::Dark(colours[i]));

            label.set_bounds(Rect::new(
                5,
                5 + (i as i32) * font_height,
                WIN_WIDTH as i32,
                font_height,
            ));

            // Don't lay out yet

            label
        });

        let text = if let Some(text) = self.args.text.as_deref() {
            text.to_owned()
        } else if let Some(path) = self.args.text_file.as_deref() {
            std::fs::read_to_string(path)?
        } else {
            "The quick brown fox jumps over the lazy dog".to_owned()
        };

        for label in &mut labels {
            label.update(&text);
            label.layout(ren);
        }

        ren.clear(ega::Id::Dark(ega::BaseId::Cyan))?;

        for label in &labels {
            label.render(ren)?;
        }

        ren.present();

        Ok(())
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
