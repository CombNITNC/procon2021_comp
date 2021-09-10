use std::collections::HashSet;

use super::{Grid, Pos, VecOnGrid};

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
        let dist = self.grid().looping_manhattan_dist(self.select, to_swap);
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
        self.grid()
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

#[test]
fn test_rotate() {
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

    // 01 10
    // 00 11
    let rotated_1 = board.rotate_to(1);
    assert_eq!(rotated_1.forward[grid.pos(0, 0)], grid.pos(0, 1));
    assert_eq!(rotated_1.forward[grid.pos(1, 0)], grid.pos(1, 0));
    assert_eq!(rotated_1.forward[grid.pos(0, 1)], grid.pos(0, 0));
    assert_eq!(rotated_1.forward[grid.pos(1, 1)], grid.pos(1, 1));

    // 00 01
    // 11 10
    let rotated_2 = board.rotate_to(2);
    assert_eq!(rotated_2.forward[grid.pos(0, 0)], grid.pos(0, 0));
    assert_eq!(rotated_2.forward[grid.pos(1, 0)], grid.pos(0, 1));
    assert_eq!(rotated_2.forward[grid.pos(0, 1)], grid.pos(1, 1));
    assert_eq!(rotated_2.forward[grid.pos(1, 1)], grid.pos(1, 0));

    // 11 00
    // 10 01
    let rotated_3 = board.rotate_to(3);
    assert_eq!(rotated_3.forward[grid.pos(0, 0)], grid.pos(1, 1));
    assert_eq!(rotated_3.forward[grid.pos(1, 0)], grid.pos(0, 0));
    assert_eq!(rotated_3.forward[grid.pos(0, 1)], grid.pos(1, 0));
    assert_eq!(rotated_3.forward[grid.pos(1, 1)], grid.pos(0, 1));
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
            offset: board.grid().pos(0, 0),
            width: board.grid().width(),
            height: board.grid().height(),
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
        todo!()
    }

    /// 時計回りに 90 度単位の `rotation` で回転する.
    pub(crate) fn rotate_to(&mut self, rotation: u8, grid: Grid) {
        self.rotation += rotation;
        self.rotation %= 4;

        std::mem::swap(&mut self.width, &mut self.height);
        self.offset = self.rotated_pos(self.offset, grid);
    }

    /// 窓の上端を 1 つ削る.
    pub(crate) fn slice_up(&mut self, board: &Board) {
        self.offset = board.grid().pos(self.offset.x(), self.offset.y());
        self.height -= 1;
    }

    /// 時計回りに 90 度単位の `rotation` で回転した位置を計算する.
    fn rotated_pos(&self, pos: Pos, grid: Grid) -> Pos {
        match self.rotation % 4 {
            0 => pos,
            1 => grid.pos(grid.width() - 1 - pos.y(), pos.x()),
            2 => grid.pos(grid.width() - 1 - pos.x(), grid.height() - 1 - pos.y()),
            3 => grid.pos(pos.y(), grid.height() - 1 - pos.x()),
            _ => unreachable!(),
        }
    }
}

pub(crate) struct FinderIter {}

impl Iterator for FinderIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
