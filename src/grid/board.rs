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

    pub(crate) fn move_pos_to(&self, pos: Pos, to: Movement) -> Pos {
        self.forward.grid.move_pos_to(pos, to)
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
        let dist = self
            .forward
            .grid
            .looping_manhattan_dist(self.select, to_swap);
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

    pub(crate) fn around_of(&self, pos: Pos) -> Vec<Pos> {
        self.forward
            .grid
            .around_of(pos)
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
}

#[test]
fn test_reverse() {
    use crate::move_resolve::edges_nodes::EdgesNodes;

    // 10 11
    // 01 00
    let grid = Grid::new(2, 2);
    let EdgesNodes { nodes, .. } = EdgesNodes::new(
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
            1 => Movement::Up,
            2 => Movement::Left,
            3 => Movement::Down,
            _ => unreachable!(),
        };
        FinderIter::new(
            self.offset,
            grid.move_pos_to(self.offset, movement.opposite()),
            grid,
            movement,
        )
    }

    fn as_grid(&self) -> Grid {
        Grid::new(self.width, self.height)
    }

    pub(crate) fn up_of(&self, pos: Pos) -> Pos {
        let grid = self.as_grid();
        match self.rotation {
            0 => grid.up_of(pos),
            1 => grid.left_of(pos),
            2 => grid.down_of(pos),
            3 => grid.right_of(pos),
            _ => unreachable!(),
        }
    }
    pub(crate) fn right_of(&self, pos: Pos) -> Pos {
        let grid = self.as_grid();
        match self.rotation {
            0 => grid.right_of(pos),
            1 => grid.down_of(pos),
            2 => grid.left_of(pos),
            3 => grid.up_of(pos),
            _ => unreachable!(),
        }
    }
    pub(crate) fn down_of(&self, pos: Pos) -> Pos {
        let grid = self.as_grid();
        match self.rotation {
            0 => grid.down_of(pos),
            1 => grid.right_of(pos),
            2 => grid.up_of(pos),
            3 => grid.left_of(pos),
            _ => unreachable!(),
        }
    }
    pub(crate) fn left_of(&self, pos: Pos) -> Pos {
        let grid = self.as_grid();
        match self.rotation {
            0 => grid.left_of(pos),
            1 => grid.up_of(pos),
            2 => grid.right_of(pos),
            3 => grid.down_of(pos),
            _ => unreachable!(),
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

pub(crate) struct FinderIter {
    next: Option<Pos>,
    end: Pos,
    grid: Grid,
    movement: Movement,
}

impl FinderIter {
    fn new(start: Pos, end: Pos, grid: Grid, movement: Movement) -> Self {
        let mut iter = Self {
            next: None,
            end,
            grid,
            movement,
        };
        iter.next = Some(start);
        iter
    }

    fn advance(&self) -> Option<Pos> {
        self.next
            .map(|next| self.grid.move_pos_to(next, self.movement))
    }
}

impl Iterator for FinderIter {
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
