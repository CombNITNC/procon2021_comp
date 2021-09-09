use std::collections::HashSet;

use super::{Grid, Pos, VecOnGrid};

#[derive(Debug, Clone)]
pub(crate) struct Board<'grid> {
    select: Pos,
    forward: VecOnGrid<'grid, Pos>,
    reverse: VecOnGrid<'grid, Pos>,
    locked: HashSet<Pos>,
}

impl<'grid> Board<'grid> {
    pub(crate) fn new(select: Pos, field: VecOnGrid<'grid, Pos>) -> Board<'grid> {
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

    pub(crate) fn grid(&self) -> &Grid {
        self.forward.grid
    }

    pub(crate) fn selected(&self) -> Pos {
        self.select
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

    pub(crate) fn lock(&mut self, pos: Pos) -> bool {
        self.locked.insert(pos)
    }

    pub(crate) fn unlock(&mut self, pos: Pos) -> bool {
        self.locked.remove(&pos)
    }
}

#[test]
fn test_reverse() {
    use crate::move_resolve::edges_nodes::EdgesNodes;

    // 10 11
    // 01 00
    let grid = Grid::new(2, 2);
    let EdgesNodes { nodes, .. } = EdgesNodes::new(
        &grid,
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
