use crate::{
    grid::{board::Board, Grid, RangePos},
    move_resolve::edges_nodes::EdgesNodes,
};

use super::{route_target_to_goal, route_target_to_pos};

#[test]
fn test_route_target_to_pos() {
    // target: {}
    // select: []
    // 22 10 20 30 40
    // 01 11 21[32]41
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

    let actual = route_target_to_pos(&board, grid.pos(2, 2), grid.pos(0, 0)).unwrap();

    let expected = vec![
        grid.pos(2, 2),
        grid.pos(1, 2),
        grid.pos(1, 1),
        grid.pos(1, 0),
        grid.pos(0, 0),
    ];
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| {
            assert_eq!(e, a, "index: {}", i);
        });
}

#[test]
fn test_route_target_to_goal() {
    // target: {}
    // select: []
    // 22 10 20 30 40
    // 01 11 21{32}41
    // 02 12[00]31 42
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

    let actual =
        route_target_to_goal(&board, grid.pos(2, 2), RangePos::single(grid.pos(0, 0))).unwrap();

    let expected = vec![
        grid.pos(2, 2),
        grid.pos(2, 1),
        grid.pos(1, 1),
        grid.pos(0, 1),
        grid.pos(0, 0),
    ];
    assert_eq!(expected.len(), actual.len(), "{:?} {:?}", expected, actual);
    expected
        .iter()
        .zip(actual.iter())
        .enumerate()
        .for_each(|(i, (e, a))| {
            assert_eq!(e, a, "index: {}", i);
        });
}
