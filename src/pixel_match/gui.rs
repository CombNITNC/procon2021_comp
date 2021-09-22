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

use self::arrow_texture::arrow_texture;

use super::ResolveHints;

mod arrow_texture;
mod image_preview;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EdgePos {
    pub(super) pos: Pos,
    pub(super) dir: Dir,
}

impl EdgePos {
    #[inline]
    pub(super) fn new(pos: Pos, dir: Dir) -> Self {
        Self { pos, dir }
    }
}

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;

pub(super) enum GuiRequest {
    Recalculate(ResolveHints),
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
        window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
        ctx,
        hints: ResolveHints::default(),
        hints_edit_history: vec![],
        hints_updated: false,
    };

    let mut renderer = Renderer {
        canvas: &mut canvas,
        texture_creator: &texture_creator,
        big_font: &big_font,
        small_font: &small_font,
        text_cache: vec![],
    };

    let mut preview: Option<RecoveredImagePreview> = None;

    loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
            state.process_sdl_event(&event);

            if let Some(ref mut preview) = preview {
                preview.process_sdl_event(event, &mut state);
            }
        }

        if !state.running {
            break;
        }

        if state.hints_updated {
            preview = None;
            state.send_recalculate_request();
        }

        renderer.set_draw_color(SdlColor::BLACK);
        renderer.clear();

        if let Some(ref preview) = preview {
            preview.render(&mut renderer, &state);
        } else {
            WaitingMessage.render(&mut renderer);

            // not to forget to process other GuiResponse
            #[allow(clippy::single_match)]
            match state.ctx.rx.try_recv() {
                Ok(GuiResponse::Recalculated {
                    recovered_image,
                    root_pos,
                }) => {
                    preview = Some(RecoveredImagePreview::new(
                        &mut renderer,
                        ImageStateIdle {
                            recovered_image,
                            root_pos,
                        },
                    ));
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
                            .map(|x| ((x as f32) * 0.8) as u8),
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

    hints: ResolveHints,
    hints_edit_history: Vec<HintsEditKind>,
    hints_updated: bool,

    ctx: GuiContext,
}

enum HintsEditKind {
    Blacklist,
    ConfirmedPairs,
}

struct ImageStateIdle {
    recovered_image: VecOnGrid<Option<Fragment>>,
    root_pos: Pos,
}

enum ImageState {
    Waiting,
    Idle(ImageStateIdle),
}

enum Hint {
    Blacklist(Pos, EdgePos),
    ConfirmedPair(EdgePos, Vec<(Pos, Rot)>),
}

impl GuiState {
    fn push_hint(&mut self, hint: Hint) {
        self.hints_updated = true;

        match hint {
            Hint::Blacklist(p, e) => {
                self.hints_edit_history.push(HintsEditKind::Blacklist);
                self.hints.blacklist.push((p, e));
            }

            Hint::ConfirmedPair(e, t) => {
                self.hints_edit_history.push(HintsEditKind::ConfirmedPairs);
                self.hints.confirmed_pairs.push((e, t));
            }
        }
    }

    fn pop_hints(&mut self) {
        self.hints_updated = true;

        match self.hints_edit_history.pop() {
            Some(HintsEditKind::Blacklist) => {
                self.hints.blacklist.pop();
            }

            Some(HintsEditKind::ConfirmedPairs) => {
                self.hints.confirmed_pairs.pop();
            }

            _ => {}
        }
    }

    fn process_sdl_event(&mut self, event: &Event) {
        use Event::*;

        match event {
            Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                self.window_size.0 = *w as u32;
                self.window_size.1 = *h as u32;
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
    }

    fn send_recalculate_request(&mut self) {
        self.hints_updated = false;
        self.ctx
            .tx
            .send(GuiRequest::Recalculate(self.hints.clone()))
            .unwrap();
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
    image: ImageStateIdle,
    recovered_image_texture: Texture<'tc>,
    arrow_texture: Texture<'tc>,

    selecting_at: (u8, u8),
    show_fragment_debug: bool,
}

impl<'tc> RecoveredImagePreview<'tc> {
    fn new(renderer: &mut Renderer<'tc>, mut image: ImageStateIdle) -> Self {
        Self {
            recovered_image_texture: create_image_texture(renderer, &mut image.recovered_image),
            arrow_texture: arrow_texture(renderer.texture_creator),
            image,

            selecting_at: (0, 0),
            show_fragment_debug: false,
        }
    }

    fn process_sdl_event(&mut self, event: Event, global_state: &mut GuiState) {
        use Event::*;
        let grid = self.image.recovered_image.grid;

        match event {
            KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                self.selecting_at.1 = self.selecting_at.1.saturating_sub(1);
            }

            KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                if self.selecting_at.1 < grid.height() - 1 {
                    self.selecting_at.1 += 1;
                }
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
                keycode: Some(Keycode::Left),
                ..
            } => {
                self.selecting_at.0 = self.selecting_at.0.saturating_sub(1);
            }

            KeyDown {
                keycode: Some(Keycode::LShift),
                ..
            } => {
                self.show_fragment_debug = true;
            }

            KeyUp {
                keycode: Some(Keycode::LShift),
                ..
            } => {
                self.show_fragment_debug = false;
            }

            KeyDown {
                keycode: Some(Keycode::U),
                ..
            } => {
                global_state.pop_hints();
            }

            KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                use Ordering::*;

                let root = self.image.root_pos;
                let selecting = self.selecting_at;

                let reference_side = match (root.x().cmp(&selecting.0), root.y().cmp(&selecting.1))
                {
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

                let reference_pos = match reference_side {
                    Dir::North => grid.pos(selecting.0, selecting.1 - 1),
                    Dir::South => grid.pos(selecting.0, selecting.1 + 1),
                    Dir::West => grid.pos(selecting.0 - 1, selecting.1),
                    Dir::East => grid.pos(selecting.0 + 1, selecting.1),
                };

                let selecting = grid.pos(selecting.0, selecting.1);

                let selecting_fragment = self.image.recovered_image[selecting].as_ref().unwrap();

                let mut table = [Dir::North, Dir::East, Dir::South, Dir::West];
                table.rotate_right(selecting_fragment.rot.as_num() as usize);

                let index = match reference_side {
                    Dir::North => 0,
                    Dir::East => 1,
                    Dir::South => 2,
                    Dir::West => 3,
                };

                let entry = EdgePos {
                    pos: selecting_fragment.pos,
                    dir: table[index],
                };

                let reference_fragment =
                    self.image.recovered_image[reference_pos].as_ref().unwrap();

                global_state.push_hint(Hint::Blacklist(reference_fragment.pos, entry));
            }

            _ => {}
        }
    }

    fn render(&self, renderer: &mut Renderer<'_>, global_state: &GuiState) {
        let image_size = {
            let query = self.recovered_image_texture.query();

            let src = global_state.window_size;
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

        self.render_selection_and_root(renderer, image_size);

        if self.show_fragment_debug {
            self.render_fragment_debug(renderer, image_size);
        }
    }

    fn render_selection_and_root(&self, renderer: &mut Renderer<'_>, image_size: (u32, u32)) {
        let root = self.image.root_pos;
        let grid = self.image.recovered_image.grid;

        let cell_side_length = image_size.0 as f64 / grid.width() as f64;
        let cell_size = (cell_side_length as i32, cell_side_length as i32);

        let offset_of = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |(x, y): (u8, u8)| (offset_of(x), offset_of(y));

        // root
        renderer.set_draw_color(SdlColor::BLUE);
        renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());

        let selecting_at = self.selecting_at;

        // selection
        renderer.set_draw_color(SdlColor::GREEN);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, Sides::all());

        use Ordering::*;

        if self.selecting_at == root.into() {
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
    }

    fn render_fragment_debug(&self, renderer: &mut Renderer<'_>, image_size: (u32, u32)) {
        let grid = self.image.recovered_image.grid;

        let cell_side_length = image_size.0 as f64 / grid.width() as f64;

        let offset_of = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |(x, y): (u8, u8)| (offset_of(x), offset_of(y));

        for x in 0..grid.width() {
            for y in 0..grid.height() {
                let pos = grid.pos(x, y);
                let fragment = self.image.recovered_image[pos].as_ref().unwrap();

                renderer.render_text(
                    format!("{}, {}", fragment.pos.x(), fragment.pos.y()),
                    offset_of((x, y)),
                    SdlColor::GREEN,
                    false,
                );

                // assuming arrow is always square.
                let arrow_side_length = 20;
                let arrow_pos = offset_of((x + 1, y + 1));
                let arrow_pos = (
                    arrow_pos.0 - arrow_side_length as i32,
                    arrow_pos.1 - arrow_side_length as i32,
                );

                let rect = Rect::new(
                    arrow_pos.0,
                    arrow_pos.1,
                    arrow_side_length,
                    arrow_side_length,
                );
                let angle = match fragment.rot {
                    Rot::R0 => 0.0,
                    Rot::R90 => 90.0,
                    Rot::R180 => 180.0,
                    Rot::R270 => 270.0,
                };

                renderer
                    .copy_ex(&self.arrow_texture, None, rect, angle, None, false, false)
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
            let texture = self
                .texture_creator
                .create_texture_from_surface(surface)
                .unwrap();

            self.text_cache.push(TextEntry {
                text: text.into_owned(),
                color,
                big,
                texture,
            });

            cache_entry = Some(self.text_cache.last().unwrap());
        }

        let texture = &cache_entry.unwrap().texture;
        let query = texture.query();
        let rect = Rect::new(pos.0, pos.1, query.width, query.height);

        self.canvas.set_draw_color(SdlColor::BLACK);
        self.canvas.fill_rect(rect).unwrap();
        self.canvas.set_draw_color(color);
        self.canvas.copy(texture, None, rect).unwrap();
    }

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
