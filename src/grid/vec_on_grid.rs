use std::{hash::Hash, ops};

use super::{Grid, Pos};

/// `VecOnGrid` は `Grid` 上の `Pos` に対応付けた値を格納し `Pos` でアクセスできるコンテナを提供する.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct VecOnGrid<T> {
    vec: Vec<T>,
    pub(crate) grid: Grid,
}

impl<T: Hash> Hash for VecOnGrid<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

impl<T> VecOnGrid<T> {
    pub(crate) fn with_init(grid: Grid, init: T) -> Self
    where
        T: Clone,
    {
        Self {
            vec: vec![init; grid.width as usize * grid.height as usize],
            grid,
        }
    }

    pub(crate) fn with_default(grid: Grid) -> Self
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

    pub(crate) fn from_vec(grid: Grid, vec: Vec<T>) -> Option<Self> {
        if vec.len() != grid.width as usize * grid.height as usize {
            return None;
        }

        Some(Self { grid, vec })
    }

    /// `a` の位置と `b` の位置の要素を入れ替える.
    pub(crate) fn swap(&mut self, a: Pos, b: Pos) {
        self.vec
            .swap(self.grid.pos_as_index(a), self.grid.pos_as_index(b))
    }

    pub(crate) fn pick_two(&self, a: Pos, b: Pos) -> (&T, &T) {
        assert!(self.grid.is_pos_valid(a));
        assert!(self.grid.is_pos_valid(b));
        let (a, b) = if self.grid.pos_as_index(b) < self.grid.pos_as_index(a) {
            (b, a)
        } else {
            (a, b)
        };
        let (a_seg, b_seg) = self.vec.split_at(self.grid.pos_as_index(b));
        (&a_seg[self.grid.pos_as_index(a)], &b_seg[0])
    }

    pub(crate) fn pick_two_mut(&mut self, a: Pos, b: Pos) -> (&mut T, &mut T) {
        assert!(self.grid.is_pos_valid(a));
        assert!(self.grid.is_pos_valid(b));
        let (a, b) = if self.grid.pos_as_index(b) < self.grid.pos_as_index(a) {
            (b, a)
        } else {
            (a, b)
        };
        let (a_seg, b_seg) = self.vec.split_at_mut(self.grid.pos_as_index(b));
        (&mut a_seg[self.grid.pos_as_index(a)], &mut b_seg[0])
    }

    /// 借用のイテレータを作る.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.into_iter()
    }

    /// 可変借用のイテレータを作る.
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.into_iter()
    }

    /// 各 Pos のタプルとなるイテレータを作る.
    pub(crate) fn iter_with_pos(&self) -> impl Iterator<Item = (Pos, &T)> {
        self.grid.all_pos().zip(self.iter())
    }

    /// 各 Pos のタプルとなる所有権を持つイテレータを作る.
    pub(crate) fn into_iter_with_pos(self) -> impl Iterator<Item = (Pos, T)> {
        self.grid.all_pos().zip(self.vec.into_iter())
    }

    /// 各 Pos のタプルとなる可変借用のイテレータを作る.
    pub(crate) fn iter_mut_with_pos(&mut self) -> impl Iterator<Item = (Pos, &mut T)> {
        self.grid.all_pos().zip(self.iter_mut())
    }

    /// アサーションなしで要素にアクセスする.
    ///
    /// # Safety
    ///
    /// 範囲外の `Pos` で呼び出すと未定義動作となる.
    pub(crate) unsafe fn get_unchecked(&self, pos: Pos) -> &T {
        self.vec.get_unchecked(self.grid.pos_as_index(pos))
    }

    /// アサーションなしで可変要素にアクセスする.
    ///
    /// # Safety
    ///
    /// 範囲外の `Pos` で呼び出すと未定義動作となる.
    pub(crate) unsafe fn get_unchecked_mut(&mut self, pos: Pos) -> &mut T {
        self.vec.get_unchecked_mut(self.grid.pos_as_index(pos))
    }

    /// 要素にアクセスする.
    pub(crate) fn get(&self, pos: Pos) -> Option<&T> {
        assert!(self.grid.is_pos_valid(pos));
        self.vec.get(self.grid.pos_as_index(pos))
    }

    /// 可変要素にアクセスする.
    pub(crate) fn get_mut(&mut self, pos: Pos) -> Option<&mut T> {
        assert!(self.grid.is_pos_valid(pos));
        self.vec.get_mut(self.grid.pos_as_index(pos))
    }

    /// `Grid` の X 方向で全体を巡回させる.
    pub(crate) fn rotate_x(&mut self, offset: isize) {
        for y in 0..self.grid.height() {
            let start = self.grid.pos_as_index(self.grid.pos(0, y));
            let end = self
                .grid
                .pos_as_index(self.grid.pos(self.grid.width() - 1, y));
            if 0 < offset {
                self.vec[start..=end].rotate_right(offset.max(0) as usize);
            } else {
                self.vec[start..=end].rotate_left((-offset).max(0) as usize);
            }
        }
    }

    /// `Grid` の Y 方向で全体を巡回させる.
    pub(crate) fn rotate_y(&mut self, offset: isize) {
        let mut chunks: Vec<_> = self.vec.chunks_exact(self.grid.width() as usize).collect();
        if 0 < offset {
            chunks.rotate_right(offset.max(0) as usize);
        } else {
            chunks.rotate_left((-offset).max(0) as usize);
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for VecOnGrid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "[")?;
            for y in 0..self.grid.height() as usize {
                write!(f, "    ")?;
                for x in 0..self.grid.width() as usize {
                    if x != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{:?}", self.vec[y * self.grid.width() as usize + x])?;
                }
                writeln!(f)?;
            }
            write!(f, "]")
        } else {
            self.vec.fmt(f)
        }
    }
}

impl<T> std::iter::IntoIterator for VecOnGrid<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, T> std::iter::IntoIterator for &'a VecOnGrid<T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a, T> std::iter::IntoIterator for &'a mut VecOnGrid<T> {
    type Item = &'a mut T;

    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
    }
}

impl<T> ops::Index<Pos> for VecOnGrid<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.vec[self.grid.pos_as_index(index)]
    }
}

impl<T> ops::IndexMut<Pos> for VecOnGrid<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.vec[self.grid.pos_as_index(index)]
    }
}
