use std::cmp::Ordering;
use std::ops::RangeInclusive;

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
                    _ => return,
                }

                let root = self.image.root_pos.into();

                if matches!(self.dragging_from, Some(d) if !Self::is_draggable(root, d, updated)) {
                    return;
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
                keycode: Some(Keycode::R),
                ..
            } => {
                global_state.force_update();
            }

            KeyDown {
                keycode: Some(Keycode::LCtrl),
                ..
            } => {
                self.dragging_from = Some(self.selecting_at);
            }

            KeyDown {
                keycode: Some(Keycode::F),
                ..
            } => {
                println!("gui: set confirmed_pair continue field to false");
                global_state.stop_continue_last_hint();
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

                let dragging_axis = dragging_from.aligned_axis(selecting_at).unwrap();
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

                let list = BidirectionalInclusiveRange::new(
                    dragging_axis_of(near_to_root)..=dragging_axis_of(far_from_root),
                )
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
                let root = self.image.root_pos.into();
                let selecting = self.selecting_at;
                let grid = self.image.recovered_image.grid;

                if selecting == root {
                    println!("gui: cannot apply blocklist on exact root pos");
                    return;
                }

                let reference_side = Self::calc_reference_side(root.into(), selecting);
                let reference_pos = selecting.move_to(reference_side);

                let selecting = grid.pos(selecting.0, selecting.1);
                let selecting_fragment = self.image.recovered_image[selecting].as_ref().unwrap();

                let entry = EdgePos {
                    pos: selecting_fragment.pos,
                    dir: Self::calc_intersects_dir(selecting_fragment.rot, reference_side),
                };

                let reference_fragment = self.image.recovered_image
                    [reference_pos.into_grid_pos(grid)]
                .as_ref()
                .unwrap();

                global_state.push_hint(Hint::Blocklist(reference_fragment.pos, entry));
                println!("gui: blocklist updated silently")
            }

            _ => {}
        }
    }

    /// fragment.rot 回転したときの、reference 方向の辺の dir を求める
    ///
    /// 例: fragment_rot = R90; reference = Dir::North;
    ///   N                 W
    /// W   E  -- R90 --> S   N このとき答えは West
    ///   S                 E
    fn calc_intersects_dir(fragment_rot: Rot, reference: Dir) -> Dir {
        let mut table = [Dir::North, Dir::East, Dir::South, Dir::West];
        table.rotate_right(fragment_rot.as_num() as usize);

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

    fn is_draggable(root: Pos, from: Pos, to: Pos) -> bool {
        if from == to {
            return true;
        }

        if from == root || to == root {
            return false;
        }

        if let Some(dragging_axis) = from.aligned_axis(to) {
            let draggable_axis = root.aligned_axis(from).unwrap_or(Axis::Y);
            dragging_axis == draggable_axis
        } else {
            false
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

        self.render_hints(renderer, global_state, image_size);
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

        let scale_by_side = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |p: Pos| (scale_by_side(p.x()), scale_by_side(p.y()));

        // root
        renderer.set_draw_color(SdlColor::BLUE);
        renderer.draw_partial_rect(offset_of(root.into()), cell_size, Sides::all());

        // drag
        if matches!(self.dragging_from, Some(f) if f != selecting_at) {
            let from = self.dragging_from.unwrap();
            let dragging_axis = from.aligned_axis(selecting_at).unwrap();

            let table = IntoIterator::into_iter([from, selecting_at]);
            let begin = table.min_by_key(|x| x.get(dragging_axis)).unwrap();

            let size = match dragging_axis {
                Axis::X => (
                    scale_by_side(diff_u8(from.0, selecting_at.0) + (1/* for selecting pos */)),
                    cell_side_length as i32,
                ),
                Axis::Y => (
                    cell_side_length as i32,
                    scale_by_side(diff_u8(from.1, selecting_at.1) + (1/* for selecting pos */)),
                ),
            };

            renderer.set_draw_color(SdlColor::MAGENTA);
            renderer.draw_partial_rect(offset_of(begin), size, Sides::all());
        }

        // selection

        use Ordering::*;

        let sides = match (root.x().cmp(&selecting_at.0), root.y().cmp(&selecting_at.1)) {
            (_, Less) => Sides::TOP,
            (_, Greater) => Sides::BOTTOM,
            (Less, Equal) => Sides::LEFT,
            (Greater, Equal) => Sides::RIGHT,
            (Equal, Equal) => Sides::empty(),
        };

        renderer.set_draw_color(SdlColor::GREEN);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, Sides::all());
        renderer.set_draw_color(SdlColor::RED);
        renderer.draw_partial_rect(offset_of(selecting_at), cell_size, sides);
    }

    fn render_hints(
        &self,
        renderer: &mut Renderer<'_>,
        global_state: &GuiState,
        image_size: (u32, u32),
    ) {
        // let root = self.image.root_pos.into();
        let grid = self.image.recovered_image.grid;
        let cell_side_length = image_size.0 as f64 / grid.width() as f64;

        let offset_of_single = |p: u8| (cell_side_length * p as f64) as i32;
        let offset_of = |Pos(x, y): Pos| (offset_of_single(x), offset_of_single(y));

        // Pos on Problem Image --> Pos on Recovered Image
        let pos_on_gui_grid = |pos: Pos| {
            self.image
                .recovered_image
                .iter_with_pos()
                .find(|(_, fragment)| Pos::from(fragment.as_ref().unwrap().pos) == pos)
                .map(|(pos, _)| pos)
                .unwrap()
                .into()
        };

        for (edgepos, list, _) in &global_state.hints.confirmed_pairs {
            let growing_dir = match (
                pos_on_gui_grid(edgepos.pos.into()),
                pos_on_gui_grid(list[0].0.into()),
            ) {
                (Pos(x1, y1), Pos(x2, y2)) if x1 > x2 && y1 == y2 => Dir::West,
                (Pos(x1, y1), Pos(x2, y2)) if x1 < x2 && y1 == y2 => Dir::East,
                (Pos(x1, y1), Pos(x2, y2)) if x1 == x2 && y1 > y2 => Dir::North,
                (Pos(x1, y1), Pos(x2, y2)) if x1 == x2 && y1 < y2 => Dir::South,
                _ => unreachable!("confirmed pair should be on 1-dimensional line"),
            };

            let offset = match growing_dir {
                Dir::North | Dir::West => offset_of(pos_on_gui_grid(list.last().unwrap().0.into())),
                d @ (Dir::South | Dir::East) => {
                    offset_of(pos_on_gui_grid(edgepos.pos.into()).move_to(d))
                }
            };

            let len = list.len() as u8;
            let mut size = (cell_side_length as i32, cell_side_length as i32);

            match growing_dir {
                Dir::North | Dir::South => size.1 = offset_of_single(len) as i32,
                Dir::West | Dir::East => size.0 = offset_of_single(len) as i32,
            }

            renderer.set_draw_color(SdlColor::YELLOW);
            renderer.draw_partial_rect(offset, size, Sides::all());
        }
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

    const BYTES_PER_PIXEL: usize = 3;
    let mut data = Vec::with_capacity(
        side_length
            * side_length
            * grid.width() as usize
            * grid.height() as usize
            * BYTES_PER_PIXEL,
    );

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                let grid_pos = grid.pos(x, y);

                if let Some(x) = &mut fragment_grid[grid_pos] {
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

struct BidirectionalInclusiveRange {
    range: RangeInclusive<u8>,
    processor: fn(&mut RangeInclusive<u8>) -> Option<u8>,
}

impl BidirectionalInclusiveRange {
    #[inline]
    fn new(range: RangeInclusive<u8>) -> Self {
        if range.start() > range.end() {
            Self {
                range: *range.end()..=*range.start(),
                processor: <RangeInclusive<u8> as DoubleEndedIterator>::next_back,
            }
        } else {
            Self {
                range,
                processor: <RangeInclusive<u8> as Iterator>::next,
            }
        }
    }
}

impl Iterator for BidirectionalInclusiveRange {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        (self.processor)(&mut self.range)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

#[test]
fn test_calc_intersects_dir() {
    assert_eq!(
        RecoveredImagePreview::calc_intersects_dir(Rot::R90, Dir::North),
        Dir::West
    );
}
