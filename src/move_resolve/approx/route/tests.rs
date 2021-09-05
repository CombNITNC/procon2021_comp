use crate::{
    grid::{board::Board, Grid},
    move_resolve::edges_nodes::EdgesNodes,
};

use super::route_target_to_goal;

#[test]
fn test_route_target_to_goal() {
    // 01 10 20 30 40
    // 00 21 11 31 41
    // 12 22 02 32 42
    // 03 13 23 33 43
    //       ↓
    // 00 10 20 30 40
    // 01 11 21 31 41
    // 02 12 22 32 42
    // 03 13 23 33 43
    let grid = Grid::new(5, 4);
    let movements = &[(grid.pos(0, 0), grid.pos(2, 2))];
    let EdgesNodes { nodes, .. } = EdgesNodes::new(&grid, movements);
    let board = Board::new(grid.pos(1, 1), nodes);

    let actual = route_target_to_goal(&board, grid.pos(0, 1), grid.all_pos()).unwrap();

    let expected = vec![
        grid.pos(1, 1),
        grid.pos(1, 0),
        grid.pos(0, 0),
        grid.pos(0, 1),
    ];
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| {
            assert_eq!(e, a, "{:?} {:?}, index: {}", e, a, i);
        });
}
