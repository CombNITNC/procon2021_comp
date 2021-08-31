use super::{Grid, Pos, VecOnGrid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CyclesNode {
    Child { parent: Pos },
    Root { len: u8 },
}

use CyclesNode::*;

/// 入れ替える頂点の互換を Union-Find で管理する.
#[derive(Debug, Clone)]
pub(crate) struct Cycles<'grid> {
    map: VecOnGrid<'grid, CyclesNode>,
}

impl<'grid> Cycles<'grid> {
    pub(crate) fn new(grid: &'grid Grid, cycles: &[(Pos, Pos)]) -> Self {
        let mut c = Self {
            map: VecOnGrid::with_init(grid, Root { len: 1 }),
        };
        for &(a, b) in cycles {
            c.union(a, b);
        }
        c
    }

    fn union(&mut self, a: Pos, b: Pos) {
        assert!(self.map.grid.is_pos_valid(a));
        assert!(self.map.grid.is_pos_valid(b));
        let a = self.repr(a);
        let b = self.repr(b);
        if a == b {
            return;
        }
        if let (Root { len: a_len }, Root { len: b_len }) = self.map.pick_two(a, b) {
            if a_len < b_len {
                self.map.swap(a, b);
            }
            if let (Root { len: a_len }, Root { len: b_len }) = self.map.pick_two_mut(a, b) {
                *a_len += *b_len;
            }
            self.map[b] = Child { parent: a };
        }
    }

    fn repr(&mut self, x: Pos) -> Pos {
        assert!(self.map.grid.is_pos_valid(x));
        match self.map[x] {
            Root { .. } => x,
            Child { parent } => {
                let parent = self.repr(parent);
                self.map[x] = Child { parent };
                parent
            }
        }
    }

    pub(crate) fn grid(&self) -> &Grid {
        self.map.grid
    }

    pub(crate) fn tree_count(&self) -> usize {
        self.map.iter().filter(|n| matches!(n, Root { .. })).count()
    }

    pub(crate) fn on_swap(&mut self, a: Pos, b: Pos) {
        todo!()
    }

    pub(crate) fn scatter_amount(&self) -> u64 {
        self.map
            .iter_with_pos()
            .map(|(pos, &i)| match i {
                Child { parent } => self.grid().looping_manhattan_dist(pos, parent) as u64,
                Root { .. } => 0,
            })
            .sum()
    }

    pub(crate) fn cycle_size(&mut self, belonged: Pos) -> u64 {
        assert!(self.map.grid.is_pos_valid(belonged));
        let repr = self.repr(belonged);
        match self.map[repr] {
            Child { .. } => unreachable!(),
            Root { len } => len as u64,
        }
    }

    pub(crate) fn different_cells(&'grid self) -> impl Iterator<Item = Pos> + 'grid {
        self.map
            .iter_with_pos()
            .filter(|&(_, &i)| !matches!(i, Root { len: 1 }))
            .map(|(p, _)| p)
    }
}

impl PartialEq for Cycles<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.map.iter().zip(other.map.iter()).all(|(a, b)| a == b)
    }
}