use std::collections::HashMap;
use std::time::Duration;

use sdl2::event::Event;
use thiserror::Error;

use ugly::colour::{ega, Ega};
use ugly::font;
use ugly::metrics::Rect;
use ugly::resource::Map;
use ugly::ui::widgets::Label;
use ugly::ui::{Layoutable, Renderable};
use ugly::Renderer;

const WIN_WIDTH: u32 = 640;
const WIN_HEIGHT: u32 = 480;

fn main() -> anyhow::Result<()> {
    let sdl = sdl2::init().map_err(Error::Init)?;
    let mut event = sdl.event_pump().map_err(Error::Init)?;
    let video = sdl.video().map_err(Error::Init)?;

    let window = video
        .window("Font Viewer", WIN_WIDTH, WIN_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(Error::Window)?;

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
}

type FontMap = ugly::resource::DefaultingHashMap<usize, ugly::Font>;

fn get_fonts() -> ugly::resource::DefaultingHashMap<usize, ugly::Font> {
    let font = font::Font::from_dir("../assets/fonts/medium");

    let mut map: HashMap<usize, _> = HashMap::new();
    map.insert(0, font.clone());

    ugly::resource::DefaultingHashMap::new(map, font)
}

#[derive(Debug, Error)]
enum Error {
    #[error("SDL init error: {0}")]
    Init(String),
    #[error("SDL window build error: {0}")]
    Window(sdl2::video::WindowBuildError),
}
