use crate::{
    grid::{board::Board, Grid},
    move_resolve::edges_nodes::EdgesNodes,
};

use super::estimate_solve_row;

#[test]
fn test_estimate_solve_row() {
    // 22 10 20 30 40
    // 01 11 21 32 41
    // 02 12{00}31 42
    // 03 13 23 33 43
    // 04 14 24 34 44
    let grid = Grid::new(5, 5);
    let movements = &[
        (grid.pos(0, 0), grid.pos(2, 2)),
        (grid.pos(2, 2), grid.pos(0, 0)),
        (grid.pos(3, 1), grid.pos(3, 2)),
        (grid.pos(3, 2), grid.pos(3, 1)),
    ];
    let EdgesNodes { nodes, .. } = EdgesNodes::new(&grid, movements);
    let board = Board::new(grid.pos(3, 1), nodes);
    let actual = estimate_solve_row(board, 0);
    eprintln!("{:#?}", actual.moves);
}
