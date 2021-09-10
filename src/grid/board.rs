use std::collections::HashSet;

use super::{Grid, Pos, VecOnGrid};
use crate::basis::Movement;

#[derive(Debug, Clone)]
pub(crate) struct Board {
    select: Pos,
    forward: VecOnGrid<Pos>,
    reverse: VecOnGrid<Pos>,
    locked: HashSet<Pos>,
}

impl Board {
    pub(crate) fn new(select: Pos, field: VecOnGrid<Pos>) -> Self {
        let mut reverse = field.clone();
        for (pos, &elem) in field.iter_with_pos() {
            reverse[elem] = pos;
        }
        Self {
            select,
            forward: field,
            reverse,
            locked: HashSet::new(),
        }
    }

    pub(crate) fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        self.forward.grid.looping_manhattan_dist(a, b)
    }

    pub(crate) fn grid(&self) -> Grid {
        self.forward.grid
    }

    pub(crate) fn selected(&self) -> Pos {
        self.select
    }

    pub(crate) fn select(&mut self, to_select: Pos) {
        if self.locked.contains(&to_select) {
            panic!("the position was locked: {:?}", to_select);
        }
        self.select = to_select;
    }

    pub(crate) fn field(&self) -> &VecOnGrid<Pos> {
        &self.forward
    }

    pub(crate) fn forward(&self, pos: Pos) -> Pos {
        self.forward[pos]
    }

    pub(crate) fn reverse(&self, pos: Pos) -> Pos {
        self.reverse[pos]
    }

    pub(crate) fn swap_to(&mut self, to_swap: Pos) {
        let dist = self.looping_manhattan_dist(self.select, to_swap);
        if dist == 0 {
            return;
        }
        if self.locked.contains(&to_swap) || self.locked.contains(&self.select) {
            panic!("the position was locked: {:?}", to_swap);
        }
        assert_eq!(
            1, dist,
            "swapping position must be a neighbor\nselect: {:?}, to_swap: {:?}",
            self.select, to_swap
        );
        self.reverse
            .swap(self.forward[self.select], self.forward[to_swap]);
        self.forward.swap(self.select, to_swap);
        self.select = to_swap;
    }

    pub(crate) fn swap_many_to(&mut self, to_swaps: &[Pos]) {
        for &to_swap in to_swaps {
            self.swap_to(to_swap);
        }
    }

    fn width(&self) -> u8 {
        self.forward.grid.width()
    }
    fn height(&self) -> u8 {
        self.forward.grid.height()
    }

    fn up_of(&self, pos: Pos) -> Pos {
        if pos.y() == 0 {
            Pos::new(pos.x(), self.height() - 1)
        } else {
            Pos::new(pos.x(), pos.y() - 1)
        }
    }
    fn right_of(&self, pos: Pos) -> Pos {
        if pos.x() + 1 == self.width() {
            Pos::new(0, pos.y())
        } else {
            Pos::new(pos.x() + 1, pos.y())
        }
    }
    fn down_of(&self, pos: Pos) -> Pos {
        if pos.y() + 1 == self.height() {
            Pos::new(pos.x(), 0)
        } else {
            Pos::new(pos.x(), pos.y() + 1)
        }
    }
    fn left_of(&self, pos: Pos) -> Pos {
        if pos.x() == 0 {
            Pos::new(self.width() - 1, pos.y())
        } else {
            Pos::new(pos.x() - 1, pos.y())
        }
    }

    pub(crate) fn around_of(&self, pos: Pos) -> Vec<Pos> {
        [
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ]
        .iter()
        .copied()
        .filter(|pos| !self.locked.contains(pos))
        .collect()
    }

    pub(crate) fn is_locked(&self, pos: Pos) -> bool {
        self.locked.contains(&pos)
    }

    pub(crate) fn lock(&mut self, pos: Pos) -> bool {
        self.locked.insert(pos)
    }

    pub(crate) fn unlock(&mut self, pos: Pos) -> bool {
        self.locked.remove(&pos)
    }

    pub(crate) fn first_unlocked(&self) -> Option<Pos> {
        self.forward
            .grid
            .all_pos()
            .find(|p| !self.locked.contains(p))
    }

    pub(crate) fn new_finder(&self) -> BoardFinder {
        BoardFinder::new(self)
    }
}

#[test]
fn test_reverse() {
    use crate::move_resolve::edges_nodes::Nodes;

    // 10 11
    // 01 00
    let grid = Grid::new(2, 2);
    let Nodes { nodes, .. } = Nodes::new(
        grid,
        &[
            (grid.pos(0, 0), grid.pos(1, 1)),
            (grid.pos(1, 1), grid.pos(1, 0)),
            (grid.pos(1, 0), grid.pos(0, 0)),
        ],
    );
    let board = Board::new(grid.pos(0, 0), nodes);

    assert_eq!(board.forward[grid.pos(0, 0)], grid.pos(1, 0));
    assert_eq!(board.forward[grid.pos(1, 0)], grid.pos(1, 1));
    assert_eq!(board.forward[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.forward[grid.pos(1, 1)], grid.pos(0, 0));

    assert_eq!(board.reverse[grid.pos(0, 0)], grid.pos(1, 1));
    assert_eq!(board.reverse[grid.pos(1, 0)], grid.pos(0, 0));
    assert_eq!(board.reverse[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.reverse[grid.pos(1, 1)], grid.pos(1, 0));
}

/// `Board` に移動や回転を加えてアクセスするための覗き窓.
#[derive(Debug, Clone)]
pub(crate) struct BoardFinder {
    offset: Pos,
    width: u8,
    height: u8,
    rotation: u8,
}

impl BoardFinder {
    pub(crate) fn new(board: &Board) -> Self {
        Self {
            offset: board.forward.grid.pos(0, 0),
            width: board.forward.grid.width(),
            height: board.forward.grid.height(),
            rotation: 0,
        }
    }

    pub(crate) fn width(&self) -> u8 {
        self.width
    }
    pub(crate) fn height(&self) -> u8 {
        self.height
    }
    pub(crate) fn offset(&self) -> Pos {
        self.offset
    }
    pub(crate) fn rotation(&self) -> u8 {
        self.rotation
    }

    pub(crate) fn iter(&self) -> FinderIter {
        let grid = Grid::new(self.width, self.height);
        let movement = match self.rotation {
            0 => Movement::Right,
            1 => Movement::Down,
            2 => Movement::Left,
            3 => Movement::Up,
            _ => unreachable!(),
        };
        FinderIter::new(
            self,
            self.offset,
            self.move_pos_to(self.offset, movement.opposite()),
            grid,
            movement,
        )
    }

    fn as_grid(&self) -> Grid {
        Grid::new(self.width, self.height)
    }

    pub(crate) fn move_pos_to(&self, pos: Pos, movement: Movement) -> Pos {
        let grid = self.as_grid();
        let movement = match self.rotation {
            0 => movement,
            1 => movement.turn_left(),
            2 => movement.opposite(),
            3 => movement.turn_right(),
            _ => unreachable!(),
        };
        match movement {
            Movement::Up => {
                if pos.y() == 0 {
                    grid.pos(pos.x(), grid.height() - 1)
                } else {
                    grid.pos(pos.x(), pos.y() - 1)
                }
            }
            Movement::Right => {
                if pos.x() == grid.width() - 1 {
                    grid.pos(0, pos.y())
                } else {
                    grid.pos(pos.x() + 1, pos.y())
                }
            }
            Movement::Down => {
                if pos.y() == grid.height() - 1 {
                    grid.pos(pos.x(), 0)
                } else {
                    grid.pos(pos.x(), pos.y() + 1)
                }
            }
            Movement::Left => {
                if pos.x() == 0 {
                    grid.pos(grid.width() - 1, pos.y())
                } else {
                    grid.pos(pos.x() - 1, pos.y())
                }
            }
        }
    }

    /// 時計回りに 90 度単位の `rotation` で回転する.
    pub(crate) fn rotate_to(&mut self, rotation: u8) {
        let grid = self.as_grid();
        self.rotation += rotation;
        self.rotation %= 4;

        std::mem::swap(&mut self.width, &mut self.height);
        self.offset = rotated_pos(self.rotation, self.offset, grid);
    }

    /// 窓の上端を 1 つ削る.
    pub(crate) fn slice_up(&mut self, board: &Board) {
        self.offset = board.forward.grid.pos(self.offset.x(), self.offset.y());
        self.height -= 1;
    }
}

pub(crate) struct FinderIter<'f> {
    next: Option<Pos>,
    end: Pos,
    grid: Grid,
    movement: Movement,
    finder: &'f BoardFinder,
}

impl<'f> FinderIter<'f> {
    fn new(finder: &'f BoardFinder, start: Pos, end: Pos, grid: Grid, movement: Movement) -> Self {
        let mut iter = Self {
            next: None,
            end,
            grid,
            movement,
            finder,
        };
        iter.next = Some(start);
        iter
    }

    fn advance(&self) -> Option<Pos> {
        self.next
            .map(|next| self.finder.move_pos_to(next, self.movement))
    }
}

impl Iterator for FinderIter<'_> {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next?;
        self.next = if ret == self.end {
            None
        } else {
            self.advance()
        };
        Some(ret)
    }
}

/// 時計回りに 90 度単位の `rotation` で回転した位置を計算する.
fn rotated_pos(rotation: u8, pos: Pos, grid: Grid) -> Pos {
    match rotation % 4 {
        0 => pos,
        1 => grid.pos(grid.width() - 1 - pos.y(), pos.x()),
        2 => grid.pos(grid.width() - 1 - pos.x(), grid.height() - 1 - pos.y()),
        3 => grid.pos(pos.y(), grid.height() - 1 - pos.x()),
        _ => unreachable!(),
    }
}
