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

use super::{
    arrow_texture::arrow_texture, Axis, GuiState, Pos, RecalculateArtifact, Renderer, Sides,
};

pub(super) struct RecoveredImagePreview<'tc> {
    image: RecalculateArtifact,
    recovered_image_texture: Texture<'tc>,
    arrow_texture: Texture<'tc>,

    pub(super) selecting_at: Pos,
    dragging_from: Option<Pos>,
    show_fragment_debug: bool,
}

impl<'tc> RecoveredImagePreview<'tc> {
    pub(super) fn new(renderer: &mut Renderer<'tc>, mut image: RecalculateArtifact) -> Self {
        Self {
            recovered_image_texture: create_image_texture(renderer, &mut image.recovered_image),
            arrow_texture: arrow_texture(renderer.texture_creator),

            selecting_at: image.root_pos.into(),
            dragging_from: None,
            show_fragment_debug: false,

            image,
        }
    }

    pub(super) fn process_sdl_event(&mut self, event: Event, global_state: &mut GuiState) {
        use Event::*;
        let grid = self.image.recovered_image.grid;

        match event {
            KeyDown {
                keycode: Some(k @ (Keycode::Up | Keycode::Down | Keycode::Left | Keycode::Right)),
                ..
            } => {
                let mut updated = self.selecting_at;

                match k {
                    Keycode::Up => updated.1 = updated.1.saturating_sub(1),
                    Keycode::Left => updated.0 = updated.0.saturating_sub(1),
                    Keycode::Down if self.selecting_at.1 < grid.height() - 1 => updated.1 += 1,
                    Keycode::Right if self.selecting_at.0 < grid.width() - 1 => updated.0 += 1,
                    _ => unreachable!(),
                }

                let root = self.image.root_pos.into();

                match self.dragging_from {
                    Some(d) if !Self::is_draggable(root, d, updated) => return,
                    _ => {}
                }

                self.selecting_at = updated;
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
                let grid = self.image.recovered_image.grid;
                let root_pos: Pos = self.image.root_pos.into();
                let selecting_at: Pos = self.selecting_at;
                let dragging_from = self.dragging_from.take().unwrap();

                // Ctrl押しただけ
                if dragging_from == selecting_at {
                    return;
                }

                let dragging_axis = Self::dragging_axis(dragging_from, selecting_at);
                let dragging_axis_of = |p: Pos| p.get(dragging_axis);

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
                    let mut table =
                        [self.selecting_at, root_pos, dragging_from].map(dragging_axis_of);

                    table.sort_unstable();

                    if table[0] != table[1]
                        && table[1] != table[2]
                        && table[1] == dragging_axis_of(root_pos)
                    {
                        println!("rootを跨げません");
                        return;
                    }
                }

                let mut table = [self.selecting_at, dragging_from];
                table.sort_unstable_by_key(|a| {
                    diff_u8(dragging_axis_of(*a), dragging_axis_of(root_pos))
                });
                let [near_to_root, far_from_root] = table;

                // dragging_axis(near_to_root)..=dragging_axis(far_from_root)のイテレータがほしいだけ。
                // わざわざこうしているのは、Rangeがそのままでは逆順に走査できないため。
                // (例: 3..=5 は [3, 4, 5] だが、 5..=3 は [] になる。ここでは 5..=3 であっても [5, 4, 3] になってほしい。)
                let mut iter = (
                    dragging_axis_of(near_to_root)..=dragging_axis_of(far_from_root),
                    (dragging_axis_of(far_from_root)..=dragging_axis_of(near_to_root)).rev(),
                );

                let iter = if dragging_axis_of(near_to_root) > dragging_axis_of(far_from_root) {
                    &mut iter.1 as &mut dyn Iterator<Item = u8>
                } else {
                    &mut iter.0 as _
                };

                let list = iter
                    .map(|x| {
                        let pos = near_to_root.replace(dragging_axis, x);
                        let fragment = self.image.recovered_image[pos.into_grid_pos(grid)]
                            .as_ref()
                            .unwrap();

                        (fragment.pos, fragment.rot)
                    })
                    .collect::<Vec<_>>();

                let reference_side = Self::calc_reference_side(root_pos, near_to_root);
                let reference_pos = near_to_root.move_to(reference_side);

                let reference_image_pos = self.image.recovered_image
                    [reference_pos.into_grid_pos(grid)]
                .as_ref()
                .unwrap()
                .pos;

                let edgepos = EdgePos {
                    pos: reference_image_pos,
                    dir: reference_side.opposite(),
                };

                global_state.push_hint(Hint::ConfirmedPair(edgepos, list));
            }

            KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                let root = self.image.root_pos;
                let selecting = self.selecting_at;
                let grid = self.image.recovered_image.grid;

                let reference_side = Self::calc_reference_side(root.into(), selecting);
                let reference_pos = selecting.move_to(reference_side);

                let selecting = grid.pos(selecting.0, selecting.1);
                let selecting_fragment = self.image.recovered_image[selecting].as_ref().unwrap();

                let entry = EdgePos {
                    pos: selecting_fragment.pos,
                    dir: Self::calc_intersects_dir(selecting_fragment, reference_side),
                };

                let reference_fragment = self.image.recovered_image
                    [reference_pos.into_grid_pos(grid)]
                .as_ref()
                .unwrap();

                global_state.push_hint(Hint::Blacklist(reference_fragment.pos, entry));
            }

            _ => {}
        }
    }

    /// fragment.rot 回転したときの、reference 方向の辺の dir を求める
    ///
    /// 例: selecting_fragment = { rot: R90, .. }; reference = Dir::North;
    ///   N                 W
    /// W   E  -- R90 --> S   N このとき答えは West
    ///   S                 E
    fn calc_intersects_dir(fragment: &Fragment, reference: Dir) -> Dir {
        reference.rotate(fragment.rot + Rot::R270)
    }

    /// Pos にある fragment の reference となる fragment の方向を返す
    /// ↓↓↓
    /// →r←
    /// ↑↑↑
    fn calc_reference_side(root: Pos, pos: Pos) -> Dir {
        use Ordering::*;
        match (root.x().cmp(&pos.x()), root.y().cmp(&pos.y())) {
            (_, Greater) => Dir::South,
            (_, Less) => Dir::North,
            (Less, Equal) => Dir::West,
            (Greater, Equal) => Dir::East,
            (Equal, Equal) => panic!("called on exact root pos"),
        }
    }

    fn dragging_axis(from: Pos, to: Pos) -> Axis {
        match from {
            Pos(x, _) if x == to.x() => Axis::Y,
            Pos(_, y) if y == to.y() => Axis::X,
            _ => panic!("invalid dragging state, from: {:?}, to: {:?}", from, to),
        }
    }

    fn is_draggable(root: Pos, from: Pos, to: Pos) -> bool {
        if matches!((from, to), (Pos(x1, y1), Pos(x2, y2)) if (x1 != x2 && y1 != y2)) {
            return false;
        }

        if matches!((from, to), (Pos(x1, y1), Pos(x2, y2)) if (x1 == x2 && y1 == y2)) {
            return true;
        }

        /*
          123
          4f5
          678
        */

        use Ordering::*;
        let draggable_axis = match (root.x().cmp(&from.x()), root.y().cmp(&from.y())) {
            /*1*/ (Greater, Greater) |
            /*2*/ (Equal, Greater) |
            /*3*/ (Less, Greater) |
            /*6*/ (Greater, Less) |
            /*7*/ (Equal, Less) |
            /*8*/ (Less, Less) => Axis::Y,

            /*4*/ (Greater, Equal) |
            /*5*/ (Less, Equal) => Axis::X,

            /*f (from)*/ (Equal, Equal) => unreachable!(),
        };

        let dragging_axis = Self::dragging_axis(from, to);
        dragging_axis == draggable_axis
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

        let offset_of_single = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |p: Pos| (offset_of_single(p.x()), offset_of_single(p.y()));

        // root
        renderer.set_draw_color(SdlColor::BLUE);
        renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());

        // drag
        if matches!(self.dragging_from, Some(f) if f != selecting_at) {
            let from = self.dragging_from.unwrap();
            let dragging_axis = Self::dragging_axis(from, selecting_at);

            let table = IntoIterator::into_iter([from, selecting_at]);
            let begin = table.min_by_key(|x| x.get(dragging_axis)).unwrap();

            let size = match dragging_axis {
                Axis::X => (
                    offset_of_single(diff_u8(from.0, selecting_at.0) + (1/* for selecting pos */)),
                    cell_side_length as i32,
                ),
                Axis::Y => (
                    cell_side_length as i32,
                    offset_of_single(diff_u8(from.1, selecting_at.1) + (1/* for selecting pos */)),
                ),
            };

            renderer.set_draw_color(SdlColor::MAGENTA);
            renderer.draw_partial_rect(offset_of(begin), size, Sides::all());
        }

        // selection

        use Ordering::*;

        let sides = match (root.x().cmp(&selecting_at.0), root.y().cmp(&selecting_at.1)) {
            (Less, Less) => Sides::LEFT | Sides::TOP,
            (Less, Greater) => Sides::LEFT | Sides::BOTTOM,
            (Greater, Less) => Sides::RIGHT | Sides::TOP,
            (Greater, Greater) => Sides::RIGHT | Sides::BOTTOM,

            (Equal, Greater) => Sides::BOTTOM,
            (Equal, Less) => Sides::TOP,
            (Less, Equal) => Sides::LEFT,
            (Greater, Equal) => Sides::RIGHT,

            (Equal, Equal) => Sides::all(),
        };

        renderer.set_draw_color(SdlColor::RED);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, sides);
        renderer.set_draw_color(SdlColor::GREEN);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, !sides);
    }

    fn render_fragment_debug(&self, renderer: &mut Renderer<'_>, image_size: (u32, u32)) {
        let grid = self.image.recovered_image.grid;

        let cell_side_length = image_size.0 as f64 / grid.width() as f64;

        let scale_by_side = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |x, y| (scale_by_side(x), scale_by_side(y));

        for (pos, fragment) in self.image.recovered_image.iter_with_pos() {
            let fragment = fragment.as_ref().unwrap();

            renderer.render_text(
                format!("{}, {}", fragment.pos.x(), fragment.pos.y()),
                offset_of(pos.x(), pos.y()),
                SdlColor::GREEN,
                false,
            );

            // assuming arrow is always square.
            let arrow_side_length = 20;
            let arrow_pos = offset_of(pos.x() + 1, pos.y() + 1);
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

            renderer
                .copy_ex(
                    &self.arrow_texture,
                    None,
                    rect,
                    fragment.rot.as_degrees(),
                    None,
                    false,
                    false,
                )
                .unwrap();
        }
    }
}

#[inline]
fn diff_u8(a: u8, b: u8) -> u8 {
    if a > b {
        a - b
    } else {
        b - a
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
