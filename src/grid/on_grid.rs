use std::ops::{Index, IndexMut};

use super::{Grid, Pos, VecOnGrid};

/// `VecOnGrid` 及びその派生系に座標変換を提供する.
pub trait OnGrid: Index<Pos> + IndexMut<Pos> {
    /// この座標変換をした系のサイズである `Grid` を返す.
    fn grid(&self) -> Grid;

    fn get(&self, index: Pos) -> Option<&<Self as Index<Pos>>::Output> {
        self.grid().is_pos_valid(index).then(move || &self[index])
    }

    fn get_mut(&mut self, index: Pos) -> Option<&mut <Self as Index<Pos>>::Output> {
        self.grid()
            .is_pos_valid(index)
            .then(move || &mut self[index])
    }

    /// 系全体を転置するように座標変換する.
    fn transpose(self) -> Transpose<Self>
    where
        Self: Sized,
    {
        Transpose(self)
    }

    /// 系全体を X 軸で反転するように座標変換する.
    fn flip_x(self) -> FlipX<Self>
    where
        Self: Sized,
    {
        FlipX(self)
    }

    /// 系全体を Y 軸で反転するように座標変換する.
    fn flip_y(self) -> FlipY<Self>
    where
        Self: Sized,
    {
        FlipY(self)
    }

    /// 系全体を反時計回りに 90 度回すように座標変換する.
    fn rotate_to_left(self) -> FlipY<Transpose<Self>>
    where
        Self: Sized,
    {
        FlipY(Transpose(self))
    }

    /// 系全体を時計回りに 90 度回すように座標変換する.
    fn rotate_to_right(self) -> Transpose<FlipY<Self>>
    where
        Self: Sized,
    {
        Transpose(FlipY(self))
    }
}

impl<T> OnGrid for VecOnGrid<T> {
    fn grid(&self) -> Grid {
        self.grid
    }
}

#[derive(Debug, Clone)]
pub struct Transpose<V>(V);

impl<V: OnGrid + Index<Pos>> Index<Pos> for Transpose<V> {
    type Output = V::Output;

    fn index(&self, index: Pos) -> &Self::Output {
        let x = index.x();
        let y = index.y();
        self.0.index(self.0.grid().pos(y, x))
    }
}

impl<V: OnGrid + IndexMut<Pos>> IndexMut<Pos> for Transpose<V> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let x = index.x();
        let y = index.y();
        self.0.index_mut(self.0.grid().pos(y, x))
    }
}

impl<V: OnGrid + Index<Pos> + IndexMut<Pos>> OnGrid for Transpose<V> {
    fn grid(&self) -> Grid {
        let width = self.0.grid().width();
        let height = self.0.grid().height();
        Grid::new(height, width)
    }
}

#[derive(Debug, Clone)]
pub struct FlipX<V>(V);

impl<V: OnGrid + Index<Pos>> Index<Pos> for FlipX<V> {
    type Output = V::Output;

    fn index(&self, index: Pos) -> &Self::Output {
        let width = self.0.grid().width();
        self.0
            .index(self.0.grid().pos(width - index.x() - 1, index.y()))
    }
}

impl<V: OnGrid + IndexMut<Pos>> IndexMut<Pos> for FlipX<V> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let width = self.0.grid().width();
        self.0
            .index_mut(self.0.grid().pos(width - index.x() - 1, index.y()))
    }
}
impl<V: OnGrid + Index<Pos> + IndexMut<Pos>> OnGrid for FlipX<V> {
    fn grid(&self) -> Grid {
        self.0.grid()
    }
}

#[derive(Debug, Clone)]
pub struct FlipY<V>(V);

impl<V: OnGrid + Index<Pos>> Index<Pos> for FlipY<V> {
    type Output = V::Output;

    fn index(&self, index: Pos) -> &Self::Output {
        let height = self.0.grid().height();
        self.0
            .index(self.0.grid().pos(index.x(), height - index.y() - 1))
    }
}

impl<V: OnGrid + IndexMut<Pos>> IndexMut<Pos> for FlipY<V> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let height = self.0.grid().height();
        self.0
            .index_mut(self.0.grid().pos(index.x(), height - index.y() - 1))
    }
}
impl<V: OnGrid + Index<Pos> + IndexMut<Pos>> OnGrid for FlipY<V> {
    fn grid(&self) -> Grid {
        self.0.grid()
    }
}

#[test]
fn case1() {
    let grid = Grid::new(3, 4);
    let mut vec = VecOnGrid::with_default(grid);
    vec.iter_mut()
        .enumerate()
        .for_each(|(idx, elem)| *elem = idx);
    assert_eq!(vec.grid(), Grid::new(3, 4));
    assert_eq!(
        vec.iter().copied().collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    );

    let vec = vec.transpose();
    assert_eq!(vec.grid(), Grid::new(4, 3));
    assert_eq!(
        vec.grid().all_pos().map(|pos| vec[pos]).collect::<Vec<_>>(),
        vec![0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8, 11],
    );

    let vec = vec.flip_y();
    assert_eq!(vec.grid(), Grid::new(4, 3));
    assert_eq!(
        vec.grid().all_pos().map(|pos| vec[pos]).collect::<Vec<_>>(),
        vec![2, 5, 8, 11, 1, 4, 7, 10, 0, 3, 6, 9],
    );

    let vec = vec.flip_x();
    assert_eq!(vec.grid(), Grid::new(4, 3));
    assert_eq!(
        vec.grid().all_pos().map(|pos| vec[pos]).collect::<Vec<_>>(),
        vec![11, 8, 5, 2, 10, 7, 4, 1, 9, 6, 3, 0],
    );
}
