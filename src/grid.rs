pub use vec_on_grid::*;

pub mod board;
pub mod on_grid;
mod vec_on_grid;

/// `Pos` は `Grid` に存在する座標を表す.
///
/// フィールドの `u8` の上位 4 ビットに X 座標, 下位 4 ビットに Y 座標を格納する. それぞれは必ず `Grid` の `width` と `height` 未満になる.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Pos(u8);

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:X}{:X})", self.x(), self.y())
    }
}

impl Pos {
    fn new(x: u8, y: u8) -> Self {
        debug_assert!(x <= 0xf, "x coordinate out of range: {}", x);
        debug_assert!(y <= 0xf, "y coordinate out of range: {}", y);
        Self((x as u8) << 4 | y as u8)
    }

    pub fn x(&self) -> u8 {
        self.0 >> 4 & 0xf
    }

    pub fn y(&self) -> u8 {
        self.0 & 0xf
    }

    pub fn manhattan_distance(self, other: Self) -> u32 {
        ((self.x() as i32 - other.x() as i32).abs() + (self.y() as i32 - other.y() as i32).abs())
            as u32
    }
}

/// `RangePos` は `Grid` 上の矩形領域を表し, `Iterator` で走査できる.
#[derive(Debug, Clone)]
pub struct RangePos {
    start: Pos,
    end: Pos,
    x: usize,
    y: usize,
}

impl RangePos {
    pub fn single(pos: Pos) -> Self {
        Self {
            start: pos,
            end: pos,
            x: pos.x() as _,
            y: pos.y() as _,
        }
    }

    pub fn is_in(&self, pos: Pos) -> bool {
        (self.start.x()..=self.end.x()).contains(&pos.x())
            && (self.start.y()..=self.end.y()).contains(&pos.y())
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    width: u8,
    height: u8,
}

impl Grid {
    pub fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn is_pos_valid(&self, pos: Pos) -> bool {
        pos.x() < self.width && pos.y() < self.height
    }

    pub fn clamping_pos(&self, x: u8, y: u8) -> Pos {
        Pos::new(x.clamp(0, self.width - 1), y.clamp(0, self.height - 1))
    }

    pub fn pos(&self, x: u8, y: u8) -> Pos {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        Pos::new(x, y)
    }

    pub fn range(&self, up_left: Pos, down_right: Pos) -> RangePos {
        assert!(up_left.x() <= down_right.x());
        assert!(up_left.y() <= down_right.y());
        RangePos {
            start: up_left,
            end: down_right,
            x: up_left.x() as usize,
            y: up_left.y() as usize,
        }
    }

    pub fn all_pos(&self) -> RangePos {
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

    pub fn looping_manhattan_dist(&self, a: Pos, b: Pos) -> u32 {
        let vec = self.looping_min_vec(a, b);
        manhattan_dist(vec) as u32
    }

    pub fn looping_min_vec(&self, from: Pos, to: Pos) -> (i32, i32) {
        let width = self.width as i32;
        let height = self.height as i32;
        let to_x = to.x() as i32;
        let to_y = to.y() as i32;
        let other_points = match (from.x() < self.width / 2, from.y() < self.height / 2) {
            (true, true) => [
                (to_x - width, to_y),
                (to_x, to_y - height),
                (to_x - width, to_y - height),
            ],
            (true, false) => [
                (to_x - width, to_y),
                (to_x, to_y + height),
                (to_x - width, to_y + height),
            ],
            (false, true) => [
                (to_x + width, to_y),
                (to_x, to_y - height),
                (to_x + width, to_y - height),
            ],
            (false, false) => [
                (to_x + width, to_y),
                (to_x, to_y + height),
                (to_x + width, to_y + height),
            ],
        };
        std::iter::once(&(to_x, to_y))
            .chain(other_points.iter())
            .cloned()
            .map(|(to_x, to_y)| (to_x - from.x() as i32, to_y - from.y() as i32))
            .min_by(|&a, &b| manhattan_dist(a).cmp(&manhattan_dist(b)))
            .unwrap()
    }
}

fn manhattan_dist((dx, dy): (i32, i32)) -> i32 {
    dx.abs() + dy.abs()
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

#[test]
fn test_looping_min_vec() {
    let grid = Grid::new(5, 5);
    for p in grid.all_pos() {
        assert_eq!((0, 0), grid.looping_min_vec(p, p));
    }
    assert_eq!((1, 0), grid.looping_min_vec(grid.pos(4, 0), grid.pos(0, 0)));
}
