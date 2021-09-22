use std::cmp::Ordering;

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color as SdlColor, pixels::PixelFormatEnum,
    rect::Rect, render::Texture, surface::Surface,
};

use crate::{
    basis::{Dir, Rot},
    fragment::Fragment,
    grid::VecOnGrid,
    pixel_match::gui::{EdgePos, Hint},
};

use super::{arrow_texture::arrow_texture, GuiState, RecalculateArtifact, Renderer, Sides};

pub(super) struct RecoveredImagePreview<'tc> {
    image: RecalculateArtifact,
    recovered_image_texture: Texture<'tc>,
    arrow_texture: Texture<'tc>,

    selecting_at: (u8, u8),
    show_fragment_debug: bool,
}

impl<'tc> RecoveredImagePreview<'tc> {
    pub(super) fn new(renderer: &mut Renderer<'tc>, mut image: RecalculateArtifact) -> Self {
        Self {
            recovered_image_texture: create_image_texture(renderer, &mut image.recovered_image),
            arrow_texture: arrow_texture(renderer.texture_creator),
            image,

            selecting_at: (0, 0),
            show_fragment_debug: false,
        }
    }

    pub(super) fn process_sdl_event(&mut self, event: Event, global_state: &mut GuiState) {
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

    pub(super) fn render(&self, renderer: &mut Renderer<'_>, global_state: &GuiState) {
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
