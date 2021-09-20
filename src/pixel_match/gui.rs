use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color as SdlColor,
    pixels::PixelFormatEnum,
    rect::{Point, Rect},
    render::{Canvas, Texture, TextureCreator, TextureQuery},
    rwops::RWops,
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
};

use crate::{
    fragment::Fragment,
    grid::{Grid, Pos, VecOnGrid},
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;

pub(super) enum GuiRequest {
    Recalculate {
        /// pos に .0 と .1 を持つ Fragment が隣り合わないことを示す
        blacklist: Vec<(Pos, Pos)>,
    },

    Quit,
}

pub(super) enum GuiResponse {
    Recalculated {
        recovered_image: VecOnGrid<Option<Fragment>>,
        root_pos: Pos,
    },
}

pub(super) struct GuiContext {
    pub(super) tx: Sender<GuiRequest>,
    pub(super) rx: Receiver<GuiResponse>,
}

pub(super) fn begin(context: GuiContext) {
    let sdl = sdl2::init().expect("failed to initialize sdl");
    let video = sdl.video().expect("failed to initialize video subsystem");
    let ttf = sdl2::ttf::init().expect("failed to initialize ttf subsystem");

    let font_ttf = RWops::from_bytes(include_bytes!("../../mplus-1m-medium.ttf"))
        .expect("failed to create rwops");
    let font = ttf
        .load_font_from_rwops(font_ttf, 30)
        .expect("failed to load font");

    let mut canvas = video
        .window("procon2021_comp", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    let mut state = GuiState {
        running: true,
        selecting_at: (0, 0),
        window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
        image: ImageState::Waiting,
        ctx: context,
    };

    let mut recovered_image_preview: Option<RecoveredImagePreview> = None;
    let processing_message = WaitingMessage::new(&font, &texture_creator);

    loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
            state.process_sdl_event(event)
        }

        if !state.running {
            break;
        }

        canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
        canvas.clear();

        if let Some(ref preview) = recovered_image_preview {
            preview.render(&mut canvas, &state);
        } else {
            processing_message.render(&mut canvas);

            // not to forget to process other GuiResponse
            #[allow(clippy::single_match)]
            match state.ctx.rx.try_recv() {
                Ok(GuiResponse::Recalculated {
                    mut recovered_image,
                    root_pos,
                }) => {
                    recovered_image_preview = Some(RecoveredImagePreview {
                        texture: create_image_texture(&mut recovered_image, &texture_creator),
                    });

                    state.image = ImageState::Idle {
                        recovered_image,
                        root_pos,
                    };
                }

                Err(_) => {}
            }
        }

        canvas.present();

        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }

    state.ctx.tx.send(GuiRequest::Quit).unwrap();
}

fn create_image_texture<'t>(
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
    texture_creator: &'t TextureCreator<WindowContext>,
) -> Texture<'t> {
    let grid = fragment_grid.grid;
    let side_length = fragment_grid[grid.pos(0, 0)]
        .as_ref()
        .unwrap()
        .side_length();

    let width = (side_length * grid.width() as usize) as u32;
    let height = (side_length * grid.height() as usize) as u32;
    let mut surface = Surface::new(width, height, PixelFormatEnum::RGB24).unwrap();

    let mut data = Vec::with_capacity(
        side_length
            * side_length
            * grid.width() as usize
            * grid.height() as usize
            * (3/* each pixel has 3 bytes for RGB */),
    );

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                let grid_pos = grid.pos(x, y);

                if let Some(ref mut x) = &mut fragment_grid[grid_pos] {
                    data.extend(
                        x.pixels()[(py * side_length) as usize..((py + 1) * side_length) as usize]
                            .iter()
                            .flat_map(|x| [x.r, x.g, x.b]),
                    );
                } else {
                    data.extend(std::iter::repeat(0).take(side_length));
                }
            }
        }
    }

    surface.with_lock_mut(|x| x.copy_from_slice(&data));

    texture_creator
        .create_texture_from_surface(surface)
        .unwrap()
}

struct GuiState {
    running: bool,
    window_size: (u32, u32),

    ctx: GuiContext,
    image: ImageState,
    selecting_at: (u8, u8),
}

enum ImageState {
    Waiting,
    Idle {
        recovered_image: VecOnGrid<Option<Fragment>>,
        root_pos: Pos,
    },
}

impl GuiState {
    fn process_sdl_event(&mut self, event: Event) {
        use Event::*;

        match event {
            Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                self.window_size.0 += w as u32;
                self.window_size.1 += h as u32;
            }

            Quit { .. }
            | KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                self.running = false;
            }

            _ => {}
        }

        if let Some(grid) = self.image.grid() {
            match event {
                KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    self.selecting_at.0 = self.selecting_at.0.saturating_sub(1);
                }

                KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    self.selecting_at.1 = self.selecting_at.1.saturating_sub(1);
                }

                KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    if self.selecting_at.0 < grid.width() - 1 {
                        self.selecting_at.0 += 1;
                    }
                }

                KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    panic!("test panic")
                }

                KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    if self.selecting_at.1 < grid.height() - 1 {
                        self.selecting_at.1 += 1;
                    }
                }

                _ => {}
            }
        }
    }
}

impl ImageState {
    fn grid(&self) -> Option<Grid> {
        match self {
            ImageState::Waiting => None,
            ImageState::Idle {
                ref recovered_image,
                ..
            } => Some(recovered_image.grid),
        }
    }
}

struct RecoveredImagePreview<'tc> {
    texture: Texture<'tc>,
}

impl<'tc> RecoveredImagePreview<'tc> {
    fn new(texture: Texture<'tc>) -> Self {
        Self { texture }
    }

    fn render(&self, canvas: &mut Canvas<Window>, state: &GuiState) {
        let image_size = {
            let query = self.texture.query();

            let src = state.window_size;
            let dst = (query.width as u32, query.height as u32);

            let candidate_a = (
                src.0,
                ((src.0 as f64) / (dst.0 as f64) * (dst.1 as f64)) as u32,
            );
            let candidate_b = (
                ((src.1 as f64) / (dst.1 as f64) * (dst.0 as f64)) as u32,
                src.1,
            );

            if candidate_a.1 > src.1 {
                candidate_b
            } else {
                candidate_a
            }
        };

        canvas
            .copy(
                &self.texture,
                None,
                Some(Rect::new(0, 0, image_size.0, image_size.1)),
            )
            .unwrap();

        self.render_selection_box(canvas, state, image_size);
    }

    fn render_selection_box(
        &self,
        canvas: &mut Canvas<Window>,
        state: &GuiState,
        image_size: (u32, u32),
    ) {
        let grid = state
            .image
            .grid()
            .expect("RecoveredImagePreview::render is called while waiting for recovered image");

        let cell_width = image_size.0 as f64 / grid.width() as f64;
        let cell_height = image_size.1 as f64 / grid.height() as f64;

        let offset_x = (cell_width * state.selecting_at.0 as f64) as i32;
        let offset_y = (cell_height * state.selecting_at.1 as f64) as i32;

        let point = |x, y| Point::new(offset_x + x, offset_y + y);

        let cell_width = cell_width as i32;
        let cell_height = cell_height as i32;

        canvas.set_draw_color(SdlColor::RGB(255, 0, 0));
        canvas
            .draw_lines(&[
                point(0, 0),
                point(cell_width, 0),
                point(cell_width, cell_height),
                point(0, cell_height),
                point(0, 0),
            ] as &[_])
            .unwrap();
    }
}

struct WaitingMessage<'tc> {
    msg_texture: Texture<'tc>,
}

impl<'tc> WaitingMessage<'tc> {
    fn new(font: &Font, tc: &'tc TextureCreator<WindowContext>) -> Self {
        let surface = font
            .render("Waiting for recovered image")
            .blended(SdlColor::RGB(255, 255, 255))
            .unwrap();

        let msg_texture = tc.create_texture_from_surface(surface).unwrap();

        Self { msg_texture }
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        let TextureQuery { width, height, .. } = self.msg_texture.query();
        canvas
            .copy(&self.msg_texture, None, Rect::new(0, 0, width, height))
            .unwrap();
    }
}
