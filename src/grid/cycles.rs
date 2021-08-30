use super::{Grid, Pos, VecOnGrid};

/// 入れ替える頂点の互換を Union-Find で管理する.
#[derive(Debug, Clone)]
pub(crate) struct Cycles<'grid> {
    // 正の場合は親要素のインデックス, 負の場合は木のサイズ
    map: VecOnGrid<'grid, isize>,
}

impl<'grid> Cycles<'grid> {
    pub(crate) fn new(grid: &'grid Grid, cycles: &[(Pos, Pos)]) -> Self {
        let mut c = Self {
            map: VecOnGrid::with_init(grid, -1),
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
        let pos_a = self.pos(a);
        let pos_b = self.pos(b);
        if self.map[pos_a] > self.map[pos_b] {
            self.map.swap(pos_a, pos_b);
        }
        self.map[pos_a] += self.map[pos_b];
        self.map[pos_b] = a as isize;
    }

    fn repr(&mut self, x: Pos) -> usize {
        assert!(self.map.grid.is_pos_valid(x));
        if self.map[x].is_negative() {
            return self.map.grid.pos_as_index(x);
        }
        let parent = self.map[x] as usize;
        let parent = self.repr(self.pos(parent));
        self.map[x] = parent as isize;
        parent
    }

    fn pos(&self, index: usize) -> Pos {
        self.map.grid.index_as_pos(index)
    }

    pub(crate) fn grid(&self) -> &Grid {
        self.map.grid
    }

    pub(crate) fn tree_count(&self) -> usize {
        self.map.iter().filter(|&&i| i < 0).count()
    }

    pub(crate) fn on_swap(&mut self, a: Pos, b: Pos) {}

    pub(crate) fn scatter_amount(&self) -> u64 {
        self.map
            .iter_with_pos()
            .filter(|&(_, &i)| i < -1)
            .map(|(p, _)| {
                self.grid()
                    .looping_manhattan_dist(p, self.pos(self.map[p] as usize))
                    as u64
            })
            .sum()
    }

    pub(crate) fn cycle_size(&mut self, belonged: Pos) -> u64 {
        assert!(self.map.grid.is_pos_valid(belonged));
        let repr = self.repr(belonged);
        -self.map[self.pos(repr)] as u64
    }

    pub(crate) fn different_cells(&'grid self) -> impl Iterator<Item = Pos> + 'grid {
        self.map
            .iter_with_pos()
            .filter(|&(_, &i)| i != -1)
            .map(|(p, _)| p)
    }
}

impl PartialEq for Cycles<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.map.iter().zip(other.map.iter()).all(|(a, b)| a == b)
    }
}
