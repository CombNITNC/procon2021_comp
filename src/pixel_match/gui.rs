use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color as SdlColor,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    rwops::RWops,
    ttf::Font,
    video::{Window, WindowContext},
};

use crate::{
    basis::{Dir, Rot},
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
    pixel_match::gui::image_preview::RecoveredImagePreview,
};

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
    Recalculated(RecalculateArtifact),
}

pub(super) struct RecalculateArtifact {
    pub(super) recovered_image: VecOnGrid<Option<Fragment>>,
    pub(super) root_pos: Pos,
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
                Ok(GuiResponse::Recalculated(a)) => {
                    preview = Some(RecoveredImagePreview::new(&mut renderer, a));
                }

                Err(_) => {}
            }
        }

        renderer.present();

        // 60fps
        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }

    state.ctx.tx.send(GuiRequest::Quit).unwrap();
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

struct WaitingMessage;

impl WaitingMessage {
    fn render(&self, canvas: &mut Renderer<'_>) {
        canvas.render_text("Waiting for recovered image", (0, 0), SdlColor::WHITE, true);
    }
}

struct TextEntry<'a> {
    text: String,
    color: SdlColor,
    big: bool,
    texture: Texture<'a>,
}

struct Renderer<'a> {
    canvas: &'a mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    big_font: &'a Font<'a, 'a>,
    small_font: &'a Font<'a, 'a>,
    text_cache: Vec<TextEntry<'a>>,
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

bitflags::bitflags! {
    struct Sides: u8 {
        const TOP =    0b0001;
        const LEFT =   0b0010;
        const RIGHT =  0b0100;
        const BOTTOM = 0b1000;
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
