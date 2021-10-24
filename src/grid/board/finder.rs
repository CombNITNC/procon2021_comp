use crate::{
    basis::Movement,
    grid::{Grid, Pos},
};

/// `Board` に移動や回転を加えてアクセスするための覗き窓.
#[derive(Debug, Clone)]
pub struct BoardFinder {
    offset: Pos,
    original: Grid,
    width: u8,
    height: u8,
    rotation: u8,
}

impl BoardFinder {
    pub fn new(grid: Grid) -> Self {
        Self {
            offset: grid.pos(0, 0),
            original: grid,
            width: grid.width(),
            height: grid.height(),
            rotation: 0,
        }
    }

    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn height(&self) -> u8 {
        self.height
    }
    pub fn offset(&self) -> Pos {
        self.offset
    }
    pub fn rotation(&self) -> u8 {
        self.rotation
    }

    fn as_grid(&self) -> Grid {
        Grid::new(self.width, self.height)
    }

    pub fn move_pos_to(&self, pos: Pos, movement: Movement) -> Pos {
        let grid = self.original;
        let movement = match self.rotation {
            0 => movement,
            1 => movement.turn_right(),
            2 => movement.opposite(),
            3 => movement.turn_left(),
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
    pub fn rotate_to(&mut self, rotation: u8) {
        let grid = self.original;
        let original_up_left = match self.rotation {
            0 => self.offset,
            1 => grid.pos(self.offset.x() + 1 - self.height, self.offset.y()),
            2 => grid.pos(
                self.offset.x() + 1 - self.width,
                self.offset.y() + 1 - self.height,
            ),
            3 => grid.pos(self.offset.x(), self.offset.y() + 1 - self.width),
            _ => unreachable!(),
        };

        self.rotation += rotation;
        self.rotation %= 4;

        if rotation % 2 != 0 {
            std::mem::swap(&mut self.width, &mut self.height);
        }

        self.offset = match self.rotation {
            0 => original_up_left,
            1 => grid.pos(original_up_left.x() + self.height - 1, original_up_left.y()),
            2 => grid.pos(
                original_up_left.x() + self.width - 1,
                original_up_left.y() + self.height - 1,
            ),
            3 => grid.pos(original_up_left.x(), original_up_left.y() + self.width - 1),
            _ => unreachable!(),
        };
    }

    /// 窓の上端を 1 つ削る.
    pub fn slice_up(&mut self) {
        self.offset = self.move_pos_to(self.offset, Movement::Down);
        self.height -= 1;
    }
}

#[test]
fn test_finder() {
    let grid = Grid::new(6, 6);
    let mut finder = BoardFinder::new(grid);

    assert_eq!(grid.pos(0, 0), finder.offset());

    finder.rotate_to(1);
    assert_eq!(grid.pos(5, 0), finder.offset());

    finder.rotate_to(1);
    assert_eq!(grid.pos(5, 5), finder.offset());

    finder.rotate_to(1);
    assert_eq!(grid.pos(0, 5), finder.offset());

    finder.rotate_to(2);
    // 00 ..[50]
    // :     :
    // 05 .. 55
    assert_eq!(grid.pos(5, 0), finder.offset());
    finder.slice_up();
    // 00 ..[40] | 50
    // :     :   | :
    // 05 .. 45  | 55
    assert_eq!(6, finder.width());
    assert_eq!(5, finder.height());
    assert_eq!(grid.pos(4, 0), finder.offset());
    finder.rotate_to(3);
    //[00].. 40  | 50
    // :     :   | :
    // 05 .. 45  | 55
    assert_eq!(grid.pos(0, 0), finder.offset());
    finder.rotate_to(2);
    // 00 .. 40  | 50
    // :     :   | :
    // 05 ..[45] | 55
    assert_eq!(5, finder.width());
    assert_eq!(6, finder.height());
    assert_eq!(grid.pos(4, 5), finder.offset());
    finder.slice_up();
    // 00 .. 40  | 50
    // :     :   | :
    // 04 ..[44] | 54
    // ----------+
    // 05 .. 45    55
    assert_eq!(5, finder.width());
    assert_eq!(5, finder.height());
    assert_eq!(grid.pos(4, 4), finder.offset());
}

/// 時計回りに 90 度単位の `rotation` で回転した位置を計算する.
fn rotated_pos(rotation: u8, pos: Pos, grid: Grid) -> Pos {
    match rotation % 4 {
        0 => pos,
        1 => grid.pos(grid.width() - 1 - pos.y(), pos.x()),
        2 => grid.pos(grid.width() - 1 - pos.x(), grid.height() - 1 - pos.y()),
        3 => grid.pos(pos.y(), grid.width() - 1 - pos.x()),
        _ => unreachable!(),
    }
}

#[test]
fn test_rotated_pos() {
    let grid = Grid::new(6, 6);

    for pos in grid.all_pos() {
        assert_eq!(pos, rotated_pos(0, pos, grid));
    }

    assert_eq!(grid.pos(4, 1), rotated_pos(1, grid.pos(1, 1), grid));
    assert_eq!(grid.pos(4, 4), rotated_pos(2, grid.pos(1, 1), grid));
    assert_eq!(grid.pos(1, 4), rotated_pos(3, grid.pos(1, 1), grid));

    assert_eq!(grid.pos(5, 1), rotated_pos(1, grid.pos(1, 0), grid));
    assert_eq!(grid.pos(4, 5), rotated_pos(2, grid.pos(1, 0), grid));
    assert_eq!(grid.pos(0, 4), rotated_pos(3, grid.pos(1, 0), grid));
}
