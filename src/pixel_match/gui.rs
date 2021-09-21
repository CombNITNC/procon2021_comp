use std::{
    borrow::Cow,
    cmp::Ordering,
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color as SdlColor,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    rwops::RWops,
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
};

use crate::{
    basis::{Dir, Rot},
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

#[derive(Clone)]
pub(super) struct EdgePos {
    pos: Pos,
    rot: Rot,
}

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;

pub(super) enum GuiRequest {
    Recalculate {
        /// 0 と .1 の示す Edge が隣り合わないことを示す
        blacklist: Vec<(EdgePos, EdgePos)>,
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

pub(super) fn begin(ctx: GuiContext) {
    let sdl = sdl2::init().expect("failed to initialize sdl");
    let video = sdl.video().expect("failed to initialize video subsystem");
    let ttf = sdl2::ttf::init().expect("failed to initialize ttf subsystem");

    let ttf_bytes = include_bytes!("../../mplus-1m-medium.ttf");

    let font_ttf = RWops::from_bytes(ttf_bytes).expect("failed to create rwops");
    let big_font = ttf
        .load_font_from_rwops(font_ttf, 30)
        .expect("failed to load font");

    let font_ttf = RWops::from_bytes(ttf_bytes).expect("failed to create rwops");
    let small_font = ttf
        .load_font_from_rwops(font_ttf, 12)
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
        ctx,
        blacklist: vec![],
    };

    let mut renderer = Renderer {
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        big_font: &big_font,
        small_font: &small_font,
        text_cache: vec![],
    };

    let mut recovered_image_preview: Option<RecoveredImagePreview> = None;

    loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
            state.process_sdl_event(event)
        }

        if !state.running {
            break;
        }

        if let ImageState::Waiting = state.image {
            recovered_image_preview = None;
        }

        renderer.set_draw_color(SdlColor::BLACK);
        renderer.clear();

        if let Some(ref preview) = recovered_image_preview {
            preview.render(&mut renderer, &state);
        } else {
            WaitingMessage.render(&mut renderer);

            // not to forget to process other GuiResponse
            #[allow(clippy::single_match)]
            match state.ctx.rx.try_recv() {
                Ok(GuiResponse::Recalculated {
                    mut recovered_image,
                    root_pos,
                }) => {
                    let recovered_image_texture =
                        create_image_texture(&mut renderer, &mut recovered_image);

                    recovered_image_preview = Some(RecoveredImagePreview::new(
                        &mut renderer,
                        recovered_image_texture,
                    ));

                    state.image = ImageState::Idle(ImageStateIdle {
                        recovered_image,
                        root_pos,
                    });
                }

                Err(_) => {}
            }
        }

        renderer.present();

        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }

    state.ctx.tx.send(GuiRequest::Quit).unwrap();
}

fn create_image_texture<'tc>(
    renderer: &mut Renderer<'tc>,
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
) -> Texture<'tc> {
    let grid = fragment_grid.grid;
    let side_length = fragment_grid[grid.pos(0, 0)]
        .as_ref()
        .unwrap()
        .side_length();

    let width = (side_length * grid.width() as usize) as u32;
    let height = (side_length * grid.height() as usize) as u32;

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
                            .flat_map(|x| [x.r, x.g, x.b])
                            .map(|x| ((x as f32) * 0.5) as u8),
                    );
                } else {
                    data.extend(std::iter::repeat(0).take(side_length * 3));
                }
            }
        }
    }

    let mut surface = Surface::new(width, height, PixelFormatEnum::RGB24).unwrap();
    surface.with_lock_mut(|x| x.copy_from_slice(&data));

    renderer
        .texture_creator
        .create_texture_from_surface(surface)
        .unwrap()
}

struct GuiState {
    running: bool,
    window_size: (u32, u32),

    ctx: GuiContext,
    image: ImageState,
    selecting_at: (u8, u8),

    blacklist: Vec<(EdgePos, EdgePos)>,
}

struct ImageStateIdle {
    recovered_image: VecOnGrid<Option<Fragment>>,
    root_pos: Pos,
}

enum ImageState {
    Waiting,
    Idle(ImageStateIdle),
}

impl GuiState {
    fn process_sdl_event(&mut self, event: Event) {
        use Event::*;

        match event {
            Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                self.window_size.0 = w as u32;
                self.window_size.1 = h as u32;
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

        if let Some(idle_state) = self.image.as_idle() {
            let grid = idle_state.recovered_image.grid;

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
                    use Ordering::*;

                    let root = idle_state.root_pos;
                    let selecting = self.selecting_at;

                    let side = match (root.x().cmp(&selecting.0), root.y().cmp(&selecting.1)) {
                        (Equal, Greater) => Dir::South,
                        (Equal, Less) => Dir::North,
                        (Less, Equal) => Dir::West,
                        (Greater, Equal) => Dir::East,

                        (Less, Less) => Dir::North,
                        (Less, Greater) => Dir::South,
                        (Greater, Less) => Dir::North,
                        (Greater, Greater) => Dir::South,
                        (Equal, Equal) => return,
                    };

                    let reference_pos = match side {
                        Dir::North => grid.pos(selecting.0, selecting.1 - 1),
                        Dir::South => grid.pos(selecting.0, selecting.1 + 1),
                        Dir::West => grid.pos(selecting.0 - 1, selecting.1),
                        Dir::East => grid.pos(selecting.0 + 1, selecting.1),
                    };

                    let selecting = grid.pos(selecting.0, selecting.1);

                    let selecting_fragment_pos =
                        idle_state.recovered_image[selecting].as_ref().unwrap().pos;

                    let reference_fragment_pos = idle_state.recovered_image[reference_pos]
                        .as_ref()
                        .unwrap()
                        .pos;

                    self.blacklist.push((todo!(), todo!()));

                    self.ctx
                        .tx
                        .send(GuiRequest::Recalculate {
                            blacklist: self.blacklist.clone(),
                        })
                        .unwrap();

                    self.image = ImageState::Waiting;
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
    fn as_idle(&self) -> Option<&ImageStateIdle> {
        match self {
            ImageState::Idle(ref t) => Some(t),
            ImageState::Waiting => None,
        }
    }
}

struct RecoveredImagePreview<'tc> {
    recovered_image_texture: Texture<'tc>,
    arrow_texture: Texture<'tc>,
}

impl<'tc> RecoveredImagePreview<'tc> {
    fn new(renderer: &mut Renderer<'tc>, recovered_image_texture: Texture<'tc>) -> Self {
        let bitmap = include_str!("./arrow.ascii");

        let mut surface = Surface::new(13, 13, PixelFormatEnum::RGB888).unwrap();
        surface.with_lock_mut(|surface_data| {
            for (i, c) in bitmap.chars().filter(|&x| x == '.' || x == '#').enumerate() {
                let mut write = |r, g, b, a| {
                    let i = i * 4;
                    surface_data[i] = r;
                    surface_data[i + 1] = g;
                    surface_data[i + 2] = b;
                    surface_data[i + 3] = a;
                };

                match c {
                    '.' => write(0, 0, 0, 0),
                    '#' => write(0, 255, 0, 255),
                    _ => unreachable!(),
                }
            }
        });

        Self {
            recovered_image_texture,
            arrow_texture: renderer
                .texture_creator
                .create_texture_from_surface(surface)
                .unwrap(),
        }
    }

    fn render(&self, renderer: &mut Renderer<'_>, state: &GuiState) {
        let image_size = {
            let query = self.recovered_image_texture.query();

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

        renderer
            .copy(
                &self.recovered_image_texture,
                None,
                Some(Rect::new(0, 0, image_size.0, image_size.1)),
            )
            .unwrap();

        self.render_selection_and_root(renderer, state, image_size);
    }

    fn render_selection_and_root(
        &self,
        renderer: &mut Renderer<'_>,
        state: &GuiState,
        image_size: (u32, u32),
    ) {
        let image_state = state
            .image
            .as_idle()
            .expect("RecoveredImagePreview::render is called while waiting for recovered image");

        let root = image_state.root_pos;
        let grid = image_state.recovered_image.grid;

        let cell_side_length = image_size.0 as f64 / grid.width() as f64;
        let cell_size = (cell_side_length as i32, cell_side_length as i32);

        let offset_of = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |(x, y): (u8, u8)| (offset_of(x), offset_of(y));

        // root
        renderer.set_draw_color(SdlColor::BLUE);
        renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());

        let selecting_at = state.selecting_at;

        // selection
        renderer.set_draw_color(SdlColor::GREEN);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, Sides::all());

        use Ordering::*;

        if state.selecting_at == root.into() {
            renderer.set_draw_color(SdlColor::RED);
            renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());
            return;
        }

        if root.x() == selecting_at.0 || root.y() == selecting_at.1 {
            let side = match (root.x().cmp(&selecting_at.0), root.y().cmp(&selecting_at.1)) {
                (Equal, Greater) => Sides::BOTTOM,
                (Equal, Less) => Sides::TOP,
                (Less, Equal) => Sides::LEFT,
                (Greater, Equal) => Sides::RIGHT,
                _ => unreachable!(),
            };

            renderer.set_draw_color(SdlColor::RED);
            renderer.draw_partial_rect(offset_of(selecting_at), cell_size, side);
            return;
        }

        let side = match (root.x().cmp(&selecting_at.0), root.y().cmp(&selecting_at.1)) {
            (Less, Less) => Sides::LEFT | Sides::TOP,
            (Less, Greater) => Sides::LEFT | Sides::BOTTOM,
            (Greater, Less) => Sides::RIGHT | Sides::TOP,
            (Greater, Greater) => Sides::RIGHT | Sides::BOTTOM,
            _ => unreachable!(),
        };

        renderer.set_draw_color(SdlColor::RED);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, side);

        for x in 0..grid.width() {
            for y in 0..grid.height() {
                let pos = grid.pos(x, y);
                let fragment = image_state.recovered_image[pos].as_ref().unwrap();

                renderer.render_text(
                    format!("{}, {}", fragment.pos.x(), fragment.pos.y()),
                    offset_of((x, y)),
                    SdlColor::GREEN,
                    false,
                );

                let mut query = self.arrow_texture.query();
                query.width *= 3;
                query.height *= 3;
                let arrow_pos = offset_of((x + 1, y + 1));
                let arrow_pos = (
                    arrow_pos.0 - query.width as i32,
                    arrow_pos.1 - query.height as i32,
                );
                let angle = match fragment.rot {
                    Rot::R0 => 0.0,
                    Rot::R90 => 90.0,
                    Rot::R180 => 180.0,
                    Rot::R270 => 270.0,
                };

                renderer
                    .copy_ex(
                        &self.arrow_texture,
                        None,
                        Some(Rect::new(
                            arrow_pos.0,
                            arrow_pos.1,
                            query.width,
                            query.height,
                        )),
                        angle,
                        None,
                        false,
                        false,
                    )
                    .unwrap();
            }
        }
    }
}

bitflags::bitflags! {
    struct Sides: u8 {
        const TOP =    0b0001;
        const LEFT =   0b0010;
        const RIGHT =  0b0100;
        const BOTTOM = 0b1000;
    }
}

trait CanvasExtension {
    fn draw_partial_rect(&mut self, pos: (i32, i32), size: (i32, i32), sides: Sides);
}

impl CanvasExtension for Canvas<Window> {
    /// 特定の辺のみの描画もできる draw_rect
    #[inline]
    fn draw_partial_rect(&mut self, (x, y): (i32, i32), (width, height): (i32, i32), sides: Sides) {
        if sides.is_all() {
            self.draw_rect(Rect::new(x, y, (width + 1) as _, (height + 1) as _))
                .unwrap();
            return;
        }
        if sides.intersects(Sides::TOP) {
            self.draw_line((x, y), (x + width, y)).unwrap();
        }
        if sides.intersects(Sides::LEFT) {
            self.draw_line((x, y), (x, y + height)).unwrap();
        }
        if sides.intersects(Sides::RIGHT) {
            self.draw_line((x + width, y), (x + width, y + height))
                .unwrap();
        }
        if sides.intersects(Sides::BOTTOM) {
            self.draw_line((x, y + height), (x + width, y + height))
                .unwrap();
        }
    }
}

struct WaitingMessage;

impl WaitingMessage {
    fn render(&self, canvas: &mut Renderer<'_>) {
        canvas.render_text("Waiting for recovered image", (0, 0), SdlColor::WHITE, true);
    }
}

struct Renderer<'a> {
    canvas: &'a mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    big_font: &'a Font<'a, 'a>,
    small_font: &'a Font<'a, 'a>,
    text_cache: Vec<TextEntry<'a>>,
}

struct TextEntry<'a> {
    text: String,
    color: SdlColor,
    big: bool,
    texture: Texture<'a>,
}

impl Deref for Renderer<'_> {
    type Target = Canvas<Window>;

    fn deref(&self) -> &Self::Target {
        self.canvas
    }
}

impl DerefMut for Renderer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.canvas
    }
}

impl Renderer<'_> {
    fn render_text<'a>(
        &'a mut self,
        text: impl Into<Cow<'a, str>>,
        pos: (i32, i32),
        color: SdlColor,
        big: bool,
    ) {
        let text = text.into();

        let mut cache_entry = self
            .text_cache
            .iter()
            .find(|&x| x.text == text && x.color == color && x.big == big);

        if cache_entry.is_none() {
            let font = if big {
                &self.big_font
            } else {
                &self.small_font
            };

            let surface = font.render(&text).blended(color).unwrap();
            let new_texture = self
                .texture_creator
                .create_texture_from_surface(surface)
                .unwrap();

            self.text_cache.push(TextEntry {
                text: text.into_owned(),
                color,
                big,
                texture: new_texture,
            });

            cache_entry = Some(self.text_cache.last().unwrap());
        }

        let texture = &cache_entry.unwrap().texture;
        let query = texture.query();

        self.canvas
            .copy(
                texture,
                None,
                Rect::new(pos.0, pos.1, query.width, query.height),
            )
            .unwrap();
    }
}
