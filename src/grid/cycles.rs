use super::{Grid, Pos, VecOnGrid};

/// 入れ替える頂点の互換を Union-Find で管理する.
#[derive(Debug, Clone)]
pub(crate) struct Cycles<'grid> {
    // 正の場合は親要素のインデックス, 負の場合は木のサイズ
    map: VecOnGrid<'grid, isize>,
}

impl<'grid> Cycles<'grid> {
    pub(crate) fn new(grid: &'grid Grid, cycles: &[(Pos, Pos)]) -> Self {
        todo!()
    }

    pub(crate) fn grid(&self) -> &Grid {
        self.map.grid
    }

    pub(crate) fn tree_count(&self) -> usize {
        self.map.iter().filter(|&&i| i < 0).count()
    }

    pub(crate) fn on_swap(&mut self, a: Pos, b: Pos) {}

    pub(crate) fn scatter_amount(&self) -> u64 {
        todo!()
    }

    pub(crate) fn cycle_size(&mut self, belonged: Pos) -> u64 {
        todo!()
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
