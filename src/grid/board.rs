use std::{collections::HashSet, hash::Hash, ops::Deref};

use bos::Bos;

use super::{Grid, Pos, VecOnGrid};

mod finder;

pub(crate) use finder::*;

#[derive(Debug, Clone)]
pub(crate) struct Board<'b> {
    select: Pos,
    forward: Bos<'b, VecOnGrid<Pos>>,
    reverse: Bos<'b, VecOnGrid<Pos>>,
    locked: Bos<'b, HashSet<Pos>>,
}

impl Hash for Board<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.forward.hash(state);
    }
}

impl PartialEq for Board<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.forward == other.forward
    }
}

impl Eq for Board<'_> {}

impl<'b> Board<'b> {
    pub(crate) fn new(select: Pos, field: VecOnGrid<Pos>) -> Self {
        let mut reverse = field.clone();
        for (pos, &elem) in field.iter_with_pos() {
            reverse[elem] = pos;
        }
        Self {
            select,
            forward: Bos::Owned(field),
            reverse: Bos::Owned(reverse),
            locked: Bos::Owned(HashSet::new()),
        }
    }

    pub(crate) fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        self.forward.to_borrowed().grid.looping_manhattan_dist(a, b)
    }

    pub(crate) fn grid(&self) -> Grid {
        self.forward.to_borrowed().grid
    }

    pub(crate) fn selected(&self) -> Pos {
        self.select
    }

    pub(crate) fn select(&mut self, to_select: Pos) {
        if self.locked.to_borrowed().contains(&to_select) {
            panic!("the position was locked: {:?}", to_select);
        }
        self.select = to_select;
    }

    pub(crate) fn field<'a>(
        &'a self,
    ) -> impl Deref<Target = VecOnGrid<Pos>> + std::fmt::Debug + 'a {
        self.forward.to_borrowed()
    }

    pub(crate) fn forward(&self, pos: Pos) -> Pos {
        self.forward.to_borrowed()[pos]
    }

    pub(crate) fn reverse(&self, pos: Pos) -> Pos {
        self.reverse.to_borrowed()[pos]
    }

    pub(crate) fn swap_to(&mut self, to_swap: Pos) {
        let dist = self.looping_manhattan_dist(self.select, to_swap);
        if dist == 0 {
            return;
        }
        if self.locked.to_borrowed().contains(&to_swap) {
            panic!("the position was locked: {:?}", to_swap);
        }
        assert_eq!(
            1, dist,
            "swapping position must be a neighbor\nselect: {:?}, to_swap: {:?}",
            self.select, to_swap
        );
        self.reverse.to_mut().swap(
            self.forward.to_borrowed()[self.select],
            self.forward.to_borrowed()[to_swap],
        );
        self.forward.to_mut().swap(self.select, to_swap);
        self.select = to_swap;
    }

    pub(crate) fn swap_many_to(&mut self, to_swaps: &[Pos]) {
        for &to_swap in to_swaps {
            self.swap_to(to_swap);
        }
    }

    fn width(&self) -> u8 {
        self.forward.to_borrowed().grid.width()
    }
    fn height(&self) -> u8 {
        self.forward.to_borrowed().grid.height()
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

    pub(crate) fn around_of<'a>(&'a self, pos: Pos) -> impl Iterator<Item = Pos> + 'a {
        std::array::IntoIter::new([
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ])
        .filter(move |pos| !self.locked.to_borrowed().contains(pos))
    }

    pub(crate) fn is_locked(&self, pos: Pos) -> bool {
        self.locked.to_borrowed().contains(&pos)
    }

    pub(crate) fn lock(&mut self, pos: Pos) -> bool {
        if pos == self.select {
            panic!("tried to lock the selected pos: {:?}", pos);
        }
        self.locked.to_mut().insert(pos)
    }

    pub(crate) fn unlock(&mut self, pos: Pos) -> bool {
        self.locked.to_mut().remove(&pos)
    }

    pub(crate) fn first_unlocked(&self) -> Option<Pos> {
        self.forward
            .to_borrowed()
            .grid
            .all_pos()
            .find(|p| !self.locked.to_borrowed().contains(p))
    }

    pub(crate) fn new_finder(&self) -> BoardFinder {
        BoardFinder::new(self.grid())
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

    assert_eq!(board.forward.to_borrowed()[grid.pos(0, 0)], grid.pos(1, 0));
    assert_eq!(board.forward.to_borrowed()[grid.pos(1, 0)], grid.pos(1, 1));
    assert_eq!(board.forward.to_borrowed()[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.forward.to_borrowed()[grid.pos(1, 1)], grid.pos(0, 0));

    assert_eq!(board.reverse.to_borrowed()[grid.pos(0, 0)], grid.pos(1, 1));
    assert_eq!(board.reverse.to_borrowed()[grid.pos(1, 0)], grid.pos(0, 0));
    assert_eq!(board.reverse.to_borrowed()[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.reverse.to_borrowed()[grid.pos(1, 1)], grid.pos(1, 0));
}
