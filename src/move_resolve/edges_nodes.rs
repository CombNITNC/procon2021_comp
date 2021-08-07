use crate::grid::{Grid, Pos, VecOnGrid};

pub(super) struct EdgesNodes<'grid> {
    pub(super) edges: Vec<(Pos, Pos)>,
    pub(super) nodes: VecOnGrid<'grid, Pos>,
    pub(super) reversed_nodes: VecOnGrid<'grid, Pos>,
}

impl<'grid> EdgesNodes<'grid> {
    /// 頂点の移動元と移動先からグラフの重み付き辺と頂点に対する移動先を格納したものを作る.
    pub(super) fn new(grid: &'grid Grid, movements: &[(Pos, Pos)]) -> Self {
        let w = grid.width();
        let h = grid.height();
        let mut nodes = VecOnGrid::with_init(grid, grid.clamping_pos(0, 0));
        let mut reversed_nodes = VecOnGrid::with_init(grid, grid.clamping_pos(0, 0));
        for col in 0..h {
            for row in 0..w {
                let pos = grid.clamping_pos(row, col);
                nodes[pos] = pos;
                reversed_nodes[pos] = pos;
            }
        }
        for &(from, to) in movements {
            nodes[to] = from;
            reversed_nodes[from] = to;
        }
        let mut edges = Vec::with_capacity(2 * w as usize * h as usize - w as usize - h as usize);
        for pos in grid.range(grid.clamping_pos(0, 0), grid.clamping_pos(w - 2, h - 1)) {
            let right = grid.right_of(pos).unwrap();
            let a = nodes[pos];
            let b = nodes[right];
            edges.push((a, b));
        }
        for pos in grid.range(grid.clamping_pos(0, 0), grid.clamping_pos(w - 1, h - 2)) {
            let down = grid.down_of(pos).unwrap();
            let a = nodes[pos];
            let b = nodes[down];
            edges.push((a, b));
        }
        Self {
            edges,
            nodes,
            reversed_nodes,
        }
    }
}

#[test]
fn edges_case1() {
    // (0, 0) (2, 0) (3, 1) (3, 0)
    // (1, 0) (1, 1) (2, 1) (0, 1)
    let grid = Grid::new(4, 2);
    let case = &[
        (grid.pos(0, 1), grid.pos(3, 1)),
        (grid.pos(3, 1), grid.pos(2, 0)),
        (grid.pos(2, 0), grid.pos(1, 0)),
        (grid.pos(1, 0), grid.pos(0, 1)),
    ];
    let expected = vec![
        (grid.pos(0, 0), grid.pos(2, 0)),
        (grid.pos(2, 0), grid.pos(3, 1)),
        (grid.pos(3, 1), grid.pos(3, 0)),
        (grid.pos(0, 0), grid.pos(1, 0)),
        (grid.pos(2, 0), grid.pos(1, 1)),
        (grid.pos(3, 1), grid.pos(2, 1)),
        (grid.pos(3, 0), grid.pos(0, 1)),
        (grid.pos(1, 0), grid.pos(1, 1)),
        (grid.pos(1, 1), grid.pos(2, 1)),
        (grid.pos(2, 1), grid.pos(0, 1)),
    ];
    let EdgesNodes { edges: actual, .. } = EdgesNodes::new(&grid, case);
    test_edges(expected, actual);
}

#[test]
fn edge_cases2() {
    // (0, 1) (1, 0) (2, 0) (3, 1)
    // (3, 0) (1, 1) (2, 1) (0, 0)
    let grid = Grid::new(4, 2);
    let case = &[
        (grid.pos(0, 0), grid.pos(3, 1)),
        (grid.pos(3, 1), grid.pos(3, 0)),
        (grid.pos(3, 0), grid.pos(0, 1)),
        (grid.pos(0, 1), grid.pos(0, 0)),
    ];
    let expected = vec![
        (grid.pos(0, 1), grid.pos(1, 0)),
        (grid.pos(1, 0), grid.pos(2, 0)),
        (grid.pos(2, 0), grid.pos(3, 1)),
        (grid.pos(0, 1), grid.pos(3, 0)),
        (grid.pos(1, 0), grid.pos(1, 1)),
        (grid.pos(2, 0), grid.pos(2, 1)),
        (grid.pos(3, 1), grid.pos(0, 0)),
        (grid.pos(3, 0), grid.pos(1, 1)),
        (grid.pos(1, 1), grid.pos(2, 1)),
        (grid.pos(2, 1), grid.pos(0, 0)),
    ];
    let EdgesNodes { edges: actual, .. } = EdgesNodes::new(&grid, case);
    test_edges(expected, actual);
}

fn test_edges(mut expected: Vec<(Pos, Pos)>, mut actual: Vec<(Pos, Pos)>) {
    assert_eq!(expected.len(), actual.len());
    expected.sort();
    actual.sort();
    expected
        .into_iter()
        .zip(actual.into_iter())
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));
}
