use std::ops;

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

/// `VecOnGrid` は `Grid` 上の `Pos` に対応付けた値を格納し `Pos` でアクセスできるコンテナを提供する.
#[derive(Clone)]
pub(crate) struct VecOnGrid<'grid, T> {
    vec: Vec<T>,
    grid: &'grid Grid,
}

impl<'grid, T> VecOnGrid<'grid, T> {
    pub(crate) fn with_init(grid: &'grid Grid, init: T) -> Self
    where
        T: Clone,
    {
        Self {
            vec: vec![init; grid.width as usize * grid.height as usize],
            grid,
        }
    }

    pub(crate) fn with_default(grid: &'grid Grid) -> Self
    where
        T: Default,
    {
        Self {
            vec: std::iter::repeat_with(T::default)
                .take(grid.width as usize * grid.height as usize)
                .collect(),
            grid,
        }
    }

    /// `a` の位置と `b` の位置の要素を入れ替える.
    pub(crate) fn swap(&mut self, a: Pos, b: Pos) {
        self.vec
            .swap(self.grid.pos_as_index(a), self.grid.pos_as_index(b))
    }

    /// 借用のイテレータを作る.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.into_iter()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for VecOnGrid<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

impl<'grid, T> std::iter::IntoIterator for VecOnGrid<'grid, T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, 'grid, T> std::iter::IntoIterator for &'a VecOnGrid<'grid, T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<T> ops::Index<Pos> for VecOnGrid<'_, T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.vec[self.grid.pos_as_index(index)]
    }
}

impl<T> ops::IndexMut<Pos> for VecOnGrid<'_, T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.vec[self.grid.pos_as_index(index)]
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
        if pos.y() != 0 {
            Pos::new(pos.x(), pos.y() - 1)
        } else {
            Pos::new(pos.x(), self.height - 1)
        }
    }
    pub(crate) fn right_of(&self, pos: Pos) -> Pos {
        if pos.x() + 1 != self.width {
            Pos::new(pos.x() + 1, pos.y())
        } else {
            Pos::new(0, pos.y())
        }
    }
    pub(crate) fn down_of(&self, pos: Pos) -> Pos {
        if pos.y() + 1 != self.height {
            Pos::new(pos.x(), pos.y() + 1)
        } else {
            Pos::new(pos.x(), 0)
        }
    }
    pub(crate) fn left_of(&self, pos: Pos) -> Pos {
        if pos.x() != 0 {
            Pos::new(pos.x() - 1, pos.y())
        } else {
            Pos::new(self.width - 1, pos.y())
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
}
