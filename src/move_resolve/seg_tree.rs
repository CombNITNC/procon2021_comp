use std::ops::Range;

/// Monoid は以下を満たさなければならない.
/// ```rs
/// fn test<M: Monoid>(m: M, n: M, l: M) {
///     m.op(M::identity()) == m;
///     M::identity().op(m) == m;
///     m.op(n.op(l)) == m.op(n).op(l);
/// }
/// ```
pub trait Monoid: Clone + Copy + PartialEq {
    fn identity() -> Self;
    fn op(self, other: Self) -> Self;
}

/// セグメント木, N 個の Monoid に対して値の挿入と範囲内の計算結果を以下の計算量で行う.
/// 挿入: O(log N), クエリ: O(log N)
#[derive(Debug)]
pub struct SegTree<T> {
    vec: Vec<T>,
    size: usize,
}

impl<T: Monoid> SegTree<T> {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();
        Self {
            vec: vec![T::identity(); size * 2 - 1],
            size,
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        let mut index = index + self.size - 1;
        self.vec[index] = value;
        while 0 < index {
            index = (index - 1) / 2;
            self.vec[index] = self.vec[index * 2 + 1].op(self.vec[index * 2 + 2]);
        }
    }

    fn query_sub(&self, querying: Range<usize>, index: usize, looking: Range<usize>) -> T {
        if looking.end <= querying.start || querying.end <= looking.start {
            // querying が looking を含まない場合
            T::identity()
        } else if querying.start <= looking.start && looking.end <= querying.end {
            // querying が looking を完全に含む場合
            self.vec[index]
        } else {
            let mid = (looking.start + looking.end) / 2;
            self.query_sub(querying.clone(), index * 2 + 1, looking.start..mid)
                .op(self.query_sub(querying, index * 2 + 2, mid..looking.end))
        }
    }

    pub fn query(&self, querying: Range<usize>) -> T {
        self.query_sub(querying, 0, 0..self.size)
    }
}
