use std::cmp::Ordering;

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color as SdlColor, pixels::PixelFormatEnum,
    rect::Rect, render::Texture, surface::Surface,
};

use crate::{
    basis::{Dir, Rot},
    fragment::Fragment,
    grid::{Grid, Pos, VecOnGrid},
    pixel_match::gui::{EdgePos, Hint},
};

use super::{arrow_texture::arrow_texture, GuiState, RecalculateArtifact, Renderer, Sides};

pub(super) struct RecoveredImagePreview<'tc> {
    image: RecalculateArtifact,
    recovered_image_texture: Texture<'tc>,
    arrow_texture: Texture<'tc>,

    pub(super) selecting_at: (u8, u8),
    dragging_from: Option<(u8, u8)>,
    show_fragment_debug: bool,
}

impl<'tc> RecoveredImagePreview<'tc> {
    pub(super) fn new(
        renderer: &mut Renderer<'tc>,
        mut image: RecalculateArtifact,
        prev_selecting_at: Option<(u8, u8)>,
    ) -> Self {
        Self {
            recovered_image_texture: create_image_texture(renderer, &mut image.recovered_image),
            arrow_texture: arrow_texture(renderer.texture_creator),
            image,

            selecting_at: prev_selecting_at.unwrap_or((0, 0)),
            dragging_from: None,
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
                if self.dragging_from.is_some() {
                    return;
                }

                if self.selecting_at.0 < grid.width() - 1 {
                    self.selecting_at.0 += 1;
                }
            }

            KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                if self.dragging_from.is_some() {
                    return;
                }

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
                keycode: Some(Keycode::LCtrl),
                ..
            } => {
                self.dragging_from = Some(self.selecting_at);
            }

            KeyUp {
                keycode: Some(Keycode::LCtrl),
                ..
            } => {
                let dragging_from = self.dragging_from.take().unwrap();
                let root_pos = self.image.root_pos.into();

                // Ctrl押しただけ
                if dragging_from == self.selecting_at {
                    return;
                }

                /*
                    rootを跨ぐことは出来ない
                    OKな例:
                        x                           |     x
                        dragging_from & root_pos    |     root_pos
                        x                           |     selecting_at
                        x                           |     x
                        selecting_at                |     dragging_from

                    NGな例:
                        dragging_from
                        root_pos
                        selecting_at
                */
                {
                    let mut table = [self.selecting_at, root_pos, dragging_from];
                    table.sort_unstable_by_key(|x| x.1);
                    if table[0] != table[1] && table[1] != table[2] && table[1] == root_pos {
                        println!("rootを跨げません");
                        return;
                    }
                }

                let mut table = [self.selecting_at, dragging_from];
                table.sort_by_key(|a| i8::abs(a.1 as i8 - root_pos.1 as i8));

                let near_to_root = table[0];
                let far_from_root = table[1];

                let reference_side = Self::calc_reference_side(self.image.root_pos, near_to_root);

                let mut iter = (
                    near_to_root.1..=far_from_root.1,
                    (far_from_root.1..=near_to_root.1).rev(),
                );

                let list = if near_to_root.1 > far_from_root.1 {
                    &mut iter.1 as &mut dyn Iterator<Item = u8>
                } else {
                    &mut iter.0 as _
                }
                .map(|y| {
                    let fragment = self.image.recovered_image
                        [self.image.recovered_image.grid.pos(near_to_root.0, y)]
                    .as_ref()
                    .unwrap();

                    (fragment.pos, fragment.rot)
                })
                .collect::<Vec<_>>();

                let left = EdgePos {
                    pos: {
                        let pos = Self::move_on_grid(
                            self.image.recovered_image.grid,
                            near_to_root,
                            reference_side,
                        );
                        self.image.recovered_image[pos].as_ref().unwrap().pos
                    },
                    dir: reference_side.opposite(),
                };

                global_state.push_hint(Hint::ConfirmedPair(left, list));
            }

            KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                let root = self.image.root_pos;
                let selecting = self.selecting_at;

                let reference_side = Self::calc_reference_side(root, selecting);
                let reference_pos = Self::move_on_grid(grid, selecting, reference_side);

                let selecting = grid.pos(selecting.0, selecting.1);
                let selecting_fragment = self.image.recovered_image[selecting].as_ref().unwrap();

                let entry = EdgePos {
                    pos: selecting_fragment.pos,
                    dir: Self::calc_intersects_dir(selecting_fragment, reference_side),
                };

                let reference_fragment =
                    self.image.recovered_image[reference_pos].as_ref().unwrap();

                global_state.push_hint(Hint::Blacklist(reference_fragment.pos, entry));
            }

            _ => {}
        }
    }

    /// fragment.rot 回転したときの、reference 方向の辺の dir を求める
    ///
    /// 例: selecting_fragment = { rot: R90, ..}; reference = Dir::North;
    ///   N                 W
    /// W   E  -- R90 --> S   N このとき答えは West
    ///   S                 E
    fn calc_intersects_dir(fragment: &Fragment, reference: Dir) -> Dir {
        let mut table = [Dir::North, Dir::East, Dir::South, Dir::West];
        table.rotate_right(fragment.rot.as_num() as usize);

        let index = match reference {
            Dir::North => 0,
            Dir::East => 1,
            Dir::South => 2,
            Dir::West => 3,
        };

        table[index]
    }

    /// Pos にある fragment の reference となる fragment の方向を返す
    /// ↓↓↓
    /// →r←
    /// ↑↑↑
    fn calc_reference_side(root: Pos, pos: (u8, u8)) -> Dir {
        use Ordering::*;
        match (root.x().cmp(&pos.0), root.y().cmp(&pos.1)) {
            (Equal, Greater) => Dir::South,
            (Equal, Less) => Dir::North,
            (Less, Equal) => Dir::West,
            (Greater, Equal) => Dir::East,

            (Less, Less) => Dir::North,
            (Greater, Less) => Dir::North,
            (Less, Greater) => Dir::South,
            (Greater, Greater) => Dir::South,

            (Equal, Equal) => panic!("calc_reference_side is called on exact root pos"),
        }
    }

    fn move_on_grid(grid: Grid, pos: (u8, u8), dir: Dir) -> Pos {
        match dir {
            Dir::North => grid.pos(pos.0, pos.1 - 1),
            Dir::South => grid.pos(pos.0, pos.1 + 1),
            Dir::West => grid.pos(pos.0 - 1, pos.1),
            Dir::East => grid.pos(pos.0 + 1, pos.1),
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
        let selecting_at = self.selecting_at;

        let cell_side_length = image_size.0 as f64 / grid.width() as f64;
        let cell_size = (cell_side_length as i32, cell_side_length as i32);

        let offset_of = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |(x, y): (u8, u8)| (offset_of(x), offset_of(y));

        // root
        renderer.set_draw_color(SdlColor::BLUE);
        renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());

        // drag
        if matches!(self.dragging_from, Some(from) if from != selecting_at) {
            let from = self.dragging_from.unwrap();
            let range = if from.1 > selecting_at.1 {
                selecting_at.1..=from.1
            } else {
                from.1..=selecting_at.1
            };

            renderer.set_draw_color(SdlColor::MAGENTA);
            for y in range {
                renderer.draw_partial_rect(offset_of((from.0, y)), cell_size, Sides::all());
            }
        }

        // selection
        renderer.set_draw_color(SdlColor::GREEN);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, Sides::all());

        if self.selecting_at == root.into() {
            renderer.set_draw_color(SdlColor::RED);
            renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());
            return;
        }

        use Ordering::*;

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
