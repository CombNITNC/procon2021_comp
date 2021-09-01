pub(crate) use vec_on_grid::*;

mod vec_on_grid;

/// `Pos` は `Grid` に存在する座標を表す.
///
/// フィールドの `u8` の上位 4 ビットに X 座標, 下位 4 ビットに Y 座標を格納する. それぞれは必ず `Grid` の `width` と `height` 未満になる.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct Pos(u8);

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl Pos {
    fn new(x: u8, y: u8) -> Self {
        debug_assert!(x <= 0xf, "");
        debug_assert!(y <= 0xf, "");
        Self((x as u8) << 4 | y as u8)
    }

    pub(crate) fn x(&self) -> u8 {
        self.0 >> 4 & 0xf
    }

    pub(crate) fn y(&self) -> u8 {
        self.0 & 0xf
    }

    pub(crate) fn manhattan_distance(self, other: Self) -> u32 {
        ((self.x() as i32 - other.x() as i32).abs() + (self.y() as i32 - other.y() as i32).abs())
            as u32
    }
}

/// `RangePos` は `Grid` 上の矩形領域を表し, `Iterator` で走査できる.
pub(crate) struct RangePos {
    start: Pos,
    end: Pos,
    x: usize,
    y: usize,
}

impl Iterator for RangePos {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.end.y() as usize) < self.y {
            return None;
        }
        let ret = Pos::new(self.x as u8, self.y as u8);
        self.x += 1;
        if (self.end.x() as usize) < self.x {
            self.y += 1;
            self.x = self.start.x() as usize;
        }
        Some(ret)
    }
}

/// `Grid` は原画像を断片画像に分ける時の分割グリッドを表す. `Pos` はこれを介してのみ作成できる.
#[derive(Debug)]
pub(crate) struct Grid {
    width: u8,
    height: u8,
}

impl Grid {
    pub(crate) fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    pub(crate) fn width(&self) -> u8 {
        self.width
    }

    pub(crate) fn height(&self) -> u8 {
        self.height
    }

    pub(crate) fn is_pos_valid(&self, pos: Pos) -> bool {
        pos.x() < self.width && pos.y() < self.height
    }

    pub(crate) fn clamping_pos(&self, x: u8, y: u8) -> Pos {
        Pos::new(x.clamp(0, self.width - 1), y.clamp(0, self.height - 1))
    }

    #[cfg(test)]
    pub(crate) fn pos(&self, x: u8, y: u8) -> Pos {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        Pos::new(x, y)
    }

    pub(crate) fn up_of(&self, pos: Pos) -> Pos {
        if pos.y() == 0 {
            Pos::new(pos.x(), self.height - 1)
        } else {
            Pos::new(pos.x(), pos.y() - 1)
        }
    }
    pub(crate) fn right_of(&self, pos: Pos) -> Pos {
        if pos.x() + 1 == self.width {
            Pos::new(0, pos.y())
        } else {
            Pos::new(pos.x() + 1, pos.y())
        }
    }
    pub(crate) fn down_of(&self, pos: Pos) -> Pos {
        if pos.y() + 1 == self.height {
            Pos::new(pos.x(), 0)
        } else {
            Pos::new(pos.x(), pos.y() + 1)
        }
    }
    pub(crate) fn left_of(&self, pos: Pos) -> Pos {
        if pos.x() == 0 {
            Pos::new(self.width - 1, pos.y())
        } else {
            Pos::new(pos.x() - 1, pos.y())
        }
    }

    pub(crate) fn around_of(&self, pos: Pos) -> [Pos; 4] {
        [
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ]
    }

    pub(crate) fn range(&self, up_left: Pos, down_right: Pos) -> RangePos {
        assert!(up_left.x() <= down_right.x());
        assert!(up_left.y() <= down_right.y());
        RangePos {
            start: up_left,
            end: down_right,
            x: up_left.x() as usize,
            y: up_left.y() as usize,
        }
    }

    pub(crate) fn all_pos(&self) -> RangePos {
        RangePos {
            start: Pos::new(0, 0),
            end: Pos::new(self.width - 1, self.height - 1),
            x: 0,
            y: 0,
        }
    }

    fn pos_as_index(&self, pos: Pos) -> usize {
        pos.y() as usize * self.width as usize + pos.x() as usize
    }

    pub(crate) fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        let width = self.width as i32;
        let height = self.height as i32;
        let bx = b.x() as i32;
        let by = b.y() as i32;
        let other_points = match (a.x() < self.width / 2, a.y() < self.height / 2) {
            (true, true) => [
                (bx - width, by),
                (bx, by - height),
                (bx - width, by - height),
            ],
            (true, false) => [
                (bx - width, by),
                (bx, by + height),
                (bx - width, by + height),
            ],
            (false, true) => [
                (bx + width, by),
                (bx, by - height),
                (bx + width, by - height),
            ],
            (false, false) => [
                (bx + width, by),
                (bx, by + height),
                (bx + width, by + height),
            ],
        };
        std::iter::once(&(bx, by))
            .chain(other_points.iter())
            .map(|(bx, by)| (a.x() as i32 - bx).abs() + (a.y() as i32 - by).abs())
            .min()
            .unwrap() as u32
    }
}

#[test]
fn test_looping_manhattan_dist() {
    let grid = Grid::new(4, 4);
    let pos1 = grid.pos(0, 0);
    let pos2 = grid.pos(1, 1);
    let pos3 = grid.pos(2, 2);
    let pos4 = grid.pos(3, 3);
    let dist2_pos = [(pos1, pos2), (pos2, pos3), (pos3, pos4), (pos4, pos1)];
    for (a, b) in dist2_pos {
        assert_eq!(0, grid.looping_manhattan_dist(a, a), "{:?} {:?}", a, b);
        assert_eq!(0, grid.looping_manhattan_dist(b, b), "{:?} {:?}", a, b);
        assert_eq!(2, grid.looping_manhattan_dist(a, b), "{:?} {:?}", a, b);
        assert_eq!(2, grid.looping_manhattan_dist(b, a), "{:?} {:?}", a, b);
    }
    let dist4_pos = [(pos1, pos3), (pos2, pos4)];
    for (a, b) in dist4_pos {
        assert_eq!(0, grid.looping_manhattan_dist(a, a), "{:?} {:?}", a, b);
        assert_eq!(0, grid.looping_manhattan_dist(b, b), "{:?} {:?}", a, b);
        assert_eq!(4, grid.looping_manhattan_dist(a, b), "{:?} {:?}", a, b);
        assert_eq!(4, grid.looping_manhattan_dist(b, a), "{:?} {:?}", a, b);
    }
}
