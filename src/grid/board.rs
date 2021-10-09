use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    ops::Deref,
    rc::Rc,
};

use super::{Grid, Pos, VecOnGrid};

mod finder;

pub(crate) use finder::*;

#[derive(Debug)]
enum CowRc<T> {
    Borrowed(Rc<RefCell<T>>),
    Owned(Rc<RefCell<T>>),
}

impl<T> From<T> for CowRc<T> {
    fn from(v: T) -> Self {
        CowRc::Owned(Rc::new(RefCell::new(v)))
    }
}

impl<T> Clone for CowRc<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(x) => Self::Borrowed(Rc::clone(x)),
            Self::Owned(x) => Self::Borrowed(Rc::clone(x)),
        }
    }
}

impl<T: ToOwned<Owned = T>> CowRc<T> {
    fn to_mut(&mut self) -> RefMut<'_, T> {
        match self {
            CowRc::Owned(r) => r.borrow_mut(),
            CowRc::Borrowed(r) => {
                let o = Rc::new(RefCell::new(r.borrow().to_owned()));
                *self = CowRc::Owned(o);
                self.to_mut()
            }
        }
    }

    fn borrow(&self) -> Ref<'_, T> {
        match self {
            CowRc::Borrowed(r) => r.borrow(),
            CowRc::Owned(r) => r.borrow(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Board {
    select: Pos,
    forward: CowRc<VecOnGrid<Pos>>,
    reverse: CowRc<VecOnGrid<Pos>>,
    locked: CowRc<HashSet<Pos>>,
}

impl Board {
    pub(crate) fn new(select: Pos, field: VecOnGrid<Pos>) -> Self {
        let mut reverse = field.clone();
        for (pos, &elem) in field.iter_with_pos() {
            reverse[elem] = pos;
        }
        Self {
            select,
            forward: field.into(),
            reverse: reverse.into(),
            locked: HashSet::new().into(),
        }
    }

    pub(crate) fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        self.forward.borrow().grid.looping_manhattan_dist(a, b)
    }

    pub(crate) fn grid(&self) -> Grid {
        self.forward.borrow().grid
    }

    pub(crate) fn selected(&self) -> Pos {
        self.select
    }

    pub(crate) fn select(&mut self, to_select: Pos) {
        if self.locked.borrow().contains(&to_select) {
            panic!("the position was locked: {:?}", to_select);
        }
        self.select = to_select;
    }

    pub(crate) fn field<'a>(
        &'a self,
    ) -> impl Deref<Target = VecOnGrid<Pos>> + std::fmt::Debug + 'a {
        self.forward.borrow()
    }

    pub(crate) fn forward(&self, pos: Pos) -> Pos {
        self.forward.borrow()[pos]
    }

    pub(crate) fn reverse(&self, pos: Pos) -> Pos {
        self.reverse.borrow()[pos]
    }

    pub(crate) fn swap_to(&mut self, to_swap: Pos) {
        let dist = self.looping_manhattan_dist(self.select, to_swap);
        if dist == 0 {
            return;
        }
        if self.locked.borrow().contains(&to_swap) {
            panic!("the position was locked: {:?}", to_swap);
        }
        assert_eq!(
            1, dist,
            "swapping position must be a neighbor\nselect: {:?}, to_swap: {:?}",
            self.select, to_swap
        );
        self.reverse.to_mut().swap(
            self.forward.borrow()[self.select],
            self.forward.borrow()[to_swap],
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
        self.forward.borrow().grid.width()
    }
    fn height(&self) -> u8 {
        self.forward.borrow().grid.height()
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
        .filter(|pos| !self.locked.borrow().contains(pos))
        .collect()
    }

    pub(crate) fn is_locked(&self, pos: Pos) -> bool {
        self.locked.borrow().contains(&pos)
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
            .borrow()
            .grid
            .all_pos()
            .find(|p| !self.locked.borrow().contains(p))
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

    assert_eq!(board.forward.borrow()[grid.pos(0, 0)], grid.pos(1, 0));
    assert_eq!(board.forward.borrow()[grid.pos(1, 0)], grid.pos(1, 1));
    assert_eq!(board.forward.borrow()[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.forward.borrow()[grid.pos(1, 1)], grid.pos(0, 0));

    assert_eq!(board.reverse.borrow()[grid.pos(0, 0)], grid.pos(1, 1));
    assert_eq!(board.reverse.borrow()[grid.pos(1, 0)], grid.pos(0, 0));
    assert_eq!(board.reverse.borrow()[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(board.reverse.borrow()[grid.pos(1, 1)], grid.pos(1, 0));
}
