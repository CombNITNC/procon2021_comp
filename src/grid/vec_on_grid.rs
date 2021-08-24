use std::ops;

use super::{Grid, Pos};

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

    /// 各 Pos のタプルとなるイテレータを作る.
    pub(crate) fn iter_with_pos(&self) -> impl Iterator<Item = (Pos, &T)> {
        self.grid.all_pos().zip(self.iter())
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
