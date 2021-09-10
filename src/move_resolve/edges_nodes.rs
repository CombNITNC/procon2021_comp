use crate::grid::{Grid, Pos, VecOnGrid};

pub(crate) struct Nodes {
    pub(crate) nodes: VecOnGrid<Pos>,
    pub(crate) reversed_nodes: VecOnGrid<Pos>,
}

impl Nodes {
    /// 頂点の移動元と移動先からグラフの重み付き辺と頂点に対する移動先を格納したものを作る.
    pub(crate) fn new(grid: Grid, movements: &[(Pos, Pos)]) -> Self {
        let w = grid.width();
        let h = grid.height();
        let mut nodes = VecOnGrid::with_init(grid, grid.pos(0, 0));
        let mut reversed_nodes = VecOnGrid::with_init(grid, grid.pos(0, 0));
        for col in 0..h {
            for row in 0..w {
                let pos = grid.pos(row, col);
                nodes[pos] = pos;
                reversed_nodes[pos] = pos;
            }
        }
        for &(from, to) in movements {
            nodes[to] = from;
            reversed_nodes[from] = to;
        }
        Self {
            nodes,
            reversed_nodes,
        }
    }
}
