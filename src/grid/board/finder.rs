use crate::{
    basis::Movement,
    grid::{Grid, Pos},
};

/// `Board` に移動や回転を加えてアクセスするための覗き窓.
#[derive(Debug, Clone)]
pub(crate) struct BoardFinder {
    offset: Pos,
    width: u8,
    height: u8,
    rotation: u8,
}

impl BoardFinder {
    pub(crate) fn new(grid: Grid) -> Self {
        Self {
            offset: grid.pos(0, 0),
            width: grid.width(),
            height: grid.height(),
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
        let rotated = rotated_pos((rotation + 3) % 4, grid.pos(0, 0), grid);
        self.offset = grid.pos(rotated.x() + self.offset.x(), rotated.y() + self.offset.y());
    }

    /// 窓の上端を 1 つ削る.
    pub(crate) fn slice_up(&mut self) {
        let grid = self.as_grid();
        self.offset = grid.pos(self.offset.x(), self.offset.y());
        self.height -= 1;
    }
}

#[test]
fn test_finder() {
    let grid = Grid::new(6, 6);
    let mut finder = BoardFinder::new(grid);

    let expected = &[
        grid.pos(0, 0),
        grid.pos(1, 0),
        grid.pos(2, 0),
        grid.pos(3, 0),
        grid.pos(4, 0),
        grid.pos(5, 0),
    ];
    let actual: Vec<_> = finder.iter().collect();
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));

    finder.rotate_to(1);
    let expected = &[
        grid.pos(0, 5),
        grid.pos(0, 4),
        grid.pos(0, 3),
        grid.pos(0, 2),
        grid.pos(0, 1),
        grid.pos(0, 0),
    ];
    let actual: Vec<_> = finder.iter().collect();
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));

    finder.rotate_to(1);
    let expected = &[
        grid.pos(5, 5),
        grid.pos(4, 5),
        grid.pos(3, 5),
        grid.pos(2, 5),
        grid.pos(1, 5),
    ];
    let actual: Vec<_> = finder.iter().collect();
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));

    finder.rotate_to(1);
    let expected = &[
        grid.pos(5, 0),
        grid.pos(5, 1),
        grid.pos(5, 2),
        grid.pos(5, 3),
        grid.pos(5, 4),
        grid.pos(5, 5),
    ];
    let actual: Vec<_> = finder.iter().collect();
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));
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
