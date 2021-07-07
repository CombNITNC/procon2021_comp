/// `Pos` は `Grid` に存在する座標を表す.
///
/// フィールドの `u8` の上位 4 ビットに X 座標, 下位 4 ビットに Y 座標を格納する. それぞれは必ず `Grid` の `width` と `height` 未満になる.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
}

pub(crate) struct VecOnGrid<'grid, T> {
    vec: Vec<T>,
    grid: &'grid Grid,
}

impl<'grid, T> VecOnGrid<'grid, T> {
    pub(crate) fn new(grid: &'grid Grid) -> Self {
        Self {
            vec: Vec::with_capacity(grid.width as usize * grid.height as usize),
            grid,
        }
    }

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

impl<T> std::ops::Index<Pos> for VecOnGrid<'_, T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.vec[self.grid.pos_as_index(index)]
    }
}

impl<T> std::ops::IndexMut<Pos> for VecOnGrid<'_, T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.vec[self.grid.pos_as_index(index)]
    }
}

/// `Grid` は原画像を断片画像に分ける時の分割グリッドを表す. `Pos` はこれを介してのみ作成できる.
pub(crate) struct Grid {
    width: u8,
    height: u8,
}

impl Grid {
    pub(crate) fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    pub(crate) fn clamping_pos(&self, x: u8, y: u8) -> Pos {
        Pos::new(x.clamp(0, self.width - 1), y.clamp(0, self.height - 1))
    }

    pub(crate) fn up_of(&self, pos: Pos) -> Option<Pos> {
        (pos.y() != 0).then(|| Pos::new(pos.x(), pos.y() - 1))
    }
    pub(crate) fn right_of(&self, pos: Pos) -> Option<Pos> {
        (pos.x() + 1 != self.width).then(|| Pos::new(pos.x() + 1, pos.y()))
    }
    pub(crate) fn down_of(&self, pos: Pos) -> Option<Pos> {
        (pos.y() + 1 != self.height).then(|| Pos::new(pos.x(), pos.y() + 1))
    }
    pub(crate) fn left_of(&self, pos: Pos) -> Option<Pos> {
        (pos.x() != 0).then(|| Pos::new(pos.x() - 1, pos.y()))
    }

    pub(crate) fn around_of(&self, pos: Pos) -> Vec<Pos> {
        [
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ]
        .iter()
        .flatten()
        .cloned()
        .collect()
    }

    fn pos_as_index(&self, pos: Pos) -> usize {
        pos.y() as usize * self.width as usize + pos.x() as usize
    }

    fn index_to_pos(&self, i: usize) -> Pos {
        debug_assert!(i < self.width as usize * self.height as usize);
        self.clamping_pos(
            (i % self.width as usize) as u8,
            (i / self.width as usize) as u8,
        )
    }
}
