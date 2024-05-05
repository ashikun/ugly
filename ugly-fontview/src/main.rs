use std::collections::HashMap;

use anyhow::anyhow;
use pollster::FutureExt;
use wgpu::TextureFormat;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use ugly::font;
use ugly::resource::Map;
use ugly::ui::{Layoutable, Renderable};
use ugly::Renderer;

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
    surface: SurfaceContext<'this>,
}

struct SurfaceContext<'a> {
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> SurfaceContext<'a> {
    async fn new(window: &'a Window) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .ok_or_else(|| anyhow!("no adapter available"))?;

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        };
        let (device, queue) = adapter.request_device(&device_desc, None).await?;

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            surface,
            adapter,
            device,
            queue,
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        let builder = ContextBuilder {
            window,
            surface_builder: |w| SurfaceContext::new(w).block_on().unwrap(),
        };

        self.context = Some(builder.build());

        self.context
            .as_ref()
            .unwrap()
            .borrow_window()
            .request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
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
                self.context
                    .as_ref()
                    .unwrap()
                    .borrow_window()
                    .request_redraw();
            }
            _ => (),
        }
    }
}

impl App {
    fn render(&self) -> anyhow::Result<()> {
        let Some(ctx) = &self.context else {
            return Ok(());
        };

        let surface_ctx = &ctx.borrow_surface();
        let output = surface_ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            surface_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        surface_ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())

    /*
    let fonts = get_fonts();
    let metrics = font::Map::load_metrics(&fonts)?;

    let colours = ugly::colour::Palette {
        fg: ugly::colour::EGA,
        bg: ugly::colour::EGA,
    };

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
