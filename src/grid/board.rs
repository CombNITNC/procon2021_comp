use std::{hash::Hash, ops::Deref};

use fxhash::FxHashSet as HashSet;

use super::{Grid, Pos, VecOnGrid};

mod finder;

pub use finder::*;

#[derive(Debug, Clone, Eq)]
pub struct Board {
    select: Option<Pos>,
    forward: VecOnGrid<Pos>,
    reverse: VecOnGrid<Pos>,
    locked: HashSet<Pos>,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.select == other.select && self.forward == other.forward
    }
}

impl Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.select.hash(state);
        self.forward.hash(state);
    }
}

impl Board {
    pub fn new(select: Option<Pos>, field: VecOnGrid<Pos>) -> Self {
        let mut reverse = field.clone();
        for (pos, &elem) in field.iter_with_pos() {
            reverse[elem] = pos;
        }
        Self {
            select,
            forward: field,
            reverse,
            locked: HashSet::default(),
        }
    }

    pub fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        self.forward.grid.looping_manhattan_dist(a, b)
    }

    pub fn grid(&self) -> Grid {
        self.forward.grid
    }

    pub fn selected(&self) -> Option<Pos> {
        self.select
    }

    pub fn select(&mut self, to_select: Pos) {
        if self.locked.contains(&to_select) {
            panic!("the position was locked: {:?}", to_select);
        }
        self.select.replace(to_select);
    }

    pub fn field(&'_ self) -> impl Deref<Target = VecOnGrid<Pos>> + std::fmt::Debug + '_ {
        &self.forward
    }

    pub fn into_field(self) -> VecOnGrid<Pos> {
        self.forward
    }

    pub fn forward(&self, pos: Pos) -> Pos {
        self.forward[pos]
    }

    pub fn reverse(&self, pos: Pos) -> Pos {
        self.reverse[pos]
    }

    pub fn swap_to(&mut self, to_swap: Pos) {
        let select = self.select.unwrap();
        let dist = self.looping_manhattan_dist(select, to_swap);
        if dist == 0 {
            return;
        }
        if self.locked.contains(&to_swap) {
            panic!("the position was locked: {:?}", to_swap);
        }
        assert_eq!(
            1, dist,
            "swapping position must be a neighbor\nselect: {:?}, to_swap: {:?}",
            select, to_swap
        );
        self.reverse
            .swap(self.forward[select], self.forward[to_swap]);
        self.forward.swap(select, to_swap);
        self.select.replace(to_swap);
    }

    pub fn swap_many_to(&mut self, to_swaps: &[Pos]) {
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

    pub fn around_of(&'_ self, pos: Pos) -> impl Iterator<Item = Pos> + '_ {
        [
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ]
        .into_iter()
        .filter(move |pos| !self.locked.contains(pos))
    }

    pub fn is_locked(&self, pos: Pos) -> bool {
        self.locked.contains(&pos)
    }

    pub fn lock(&mut self, pos: Pos) -> bool {
        if Some(pos) == self.select {
            panic!("tried to lock the selected pos: {:?}", pos);
        }
        self.locked.insert(pos)
    }

    pub fn unlock(&mut self, pos: Pos) -> bool {
        self.locked.remove(&pos)
    }

    pub fn first_unlocked(&self) -> Option<Pos> {
        self.forward
            .grid
            .all_pos()
            .find(|p| !self.locked.contains(p))
    }

    pub fn new_finder(&self) -> BoardFinder {
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
    let board = Board::new(Some(grid.pos(0, 0)), nodes);

    assert_eq!(board.forward(grid.pos(0, 0)), grid.pos(1, 0));
    assert_eq!(board.forward(grid.pos(1, 0)), grid.pos(1, 1));
    assert_eq!(board.forward(grid.pos(0, 1)), grid.pos(0, 1));
    assert_eq!(board.forward(grid.pos(1, 1)), grid.pos(0, 0));

    assert_eq!(board.reverse(grid.pos(0, 0)), grid.pos(1, 1));
    assert_eq!(board.reverse(grid.pos(1, 0)), grid.pos(0, 0));
    assert_eq!(board.reverse(grid.pos(0, 1)), grid.pos(0, 1));
    assert_eq!(board.reverse(grid.pos(1, 1)), grid.pos(1, 0));
}
