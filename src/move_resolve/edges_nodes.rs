use crate::grid::{Grid, Pos, VecOnGrid};

pub(crate) struct EdgesNodes {
    pub(crate) edges: Vec<(Pos, Pos)>,
    pub(crate) nodes: VecOnGrid<Pos>,
    pub(crate) reversed_nodes: VecOnGrid<Pos>,
}

impl EdgesNodes {
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
        let mut edges = Vec::with_capacity(2 * w as usize * h as usize - w as usize - h as usize);
        if 2 <= w {
            for pos in grid.range(grid.pos(0, 0), grid.pos(w - 2, h - 1)) {
                let right = grid.right_of(pos);
                let a = nodes[pos];
                let b = nodes[right];
                edges.push((a, b));
            }
        }
        if 2 <= h {
            for pos in grid.range(grid.pos(0, 0), grid.pos(w - 1, h - 2)) {
                let down = grid.down_of(pos);
                let a = nodes[pos];
                let b = nodes[down];
                edges.push((a, b));
            }
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
    let EdgesNodes { edges: actual, .. } = EdgesNodes::new(grid, case);
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
    let EdgesNodes { edges: actual, .. } = EdgesNodes::new(grid, case);
    test_edges(expected, actual);
}

#[test]
fn edge_cases3() {
    // (1, 0) (0, 0)
    let grid = Grid::new(2, 1);
    let case = &[
        (grid.pos(0, 0), grid.pos(1, 0)),
        (grid.pos(1, 0), grid.pos(0, 0)),
    ];
    let expected = vec![(grid.pos(1, 0), grid.pos(0, 0))];
    let actual = EdgesNodes::new(grid, case);
    test_edges(expected, actual.edges);
    assert_eq!(actual.nodes[grid.pos(0, 0)], grid.pos(1, 0));
    assert_eq!(actual.nodes[grid.pos(1, 0)], grid.pos(0, 0));
}

#[cfg(test)]
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
