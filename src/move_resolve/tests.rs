use super::{h1, ida_star::State, resolve, GridState};
use crate::{
    basis::{Movement::*, Operation},
    grid::{Grid, VecOnGrid},
};

#[test]
fn test_h1() {
    // 00 11
    // 10 01
    let grid = Grid::new(2, 2);
    let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(0, 0);
    field[grid.pos(1, 0)] = grid.pos(1, 1);
    field[grid.pos(0, 1)] = grid.pos(1, 0);
    field[grid.pos(1, 1)] = grid.pos(0, 1);
    let state = GridState {
        grid: &grid,
        field,
        selecting: grid.pos(0, 1),
        swap_cost: 1,
        select_cost: 1,
    };
    assert_eq!(h1(&state), 1);
}

#[test]
fn simple_case() {
    //  00   11
    // [10]  01
    let grid = Grid::new(2, 2);
    let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(0, 0);
    field[grid.pos(1, 0)] = grid.pos(1, 1);
    field[grid.pos(0, 1)] = grid.pos(1, 0);
    field[grid.pos(1, 1)] = grid.pos(0, 1);
    let state = GridState {
        grid: &grid,
        field,
        selecting: grid.pos(0, 1),
        swap_cost: 1,
        select_cost: 1,
    };
    let next_states = state.next_states();
    assert_eq!(next_states.len(), 5);
    // [10]  11
    //  00   01
    assert_eq!(next_states[0].field[grid.pos(0, 0)], grid.pos(1, 0));
    assert_eq!(next_states[0].field[grid.pos(0, 1)], grid.pos(0, 0));
    assert_eq!(next_states[0].selecting, grid.pos(0, 0));
    // 00  11
    // 01 [10]
    assert_eq!(next_states[1].field[grid.pos(0, 1)], grid.pos(0, 1));
    assert_eq!(next_states[1].field[grid.pos(1, 1)], grid.pos(1, 0));
    assert_eq!(next_states[1].selecting, grid.pos(1, 1));
    // [00]  11
    //  10   01
    assert_eq!(next_states[2].selecting, grid.pos(0, 0));
    //  00  [11]
    //  10   01
    assert_eq!(next_states[3].selecting, grid.pos(1, 0));
    //  00   11
    //  10  [01]
    assert_eq!(next_states[4].selecting, grid.pos(1, 1));

    // 00 11
    // 10 01
    let path = resolve(
        &grid,
        &[
            (grid.pos(1, 0), grid.pos(0, 1)),
            (grid.pos(0, 1), grid.pos(1, 1)),
            (grid.pos(1, 1), grid.pos(1, 0)),
        ],
        1,
        1,
    );
    assert_eq!(path.len(), 1);
    assert_eq!(
        path[0],
        Operation {
            select: grid.pos(0, 1),
            movements: vec![Right, Up],
        }
    );
}

fn test_vec<E, A, T>(expected: E, actual: A)
where
    E: IntoIterator<Item = T>,
    A: IntoIterator<Item = T>,
    T: PartialEq + std::fmt::Debug,
    E::IntoIter: ExactSizeIterator,
    A::IntoIter: ExactSizeIterator,
{
    let expected = expected.into_iter();
    let actual = actual.into_iter();
    assert_eq!(expected.len(), actual.len());
    expected
        .zip(actual)
        .enumerate()
        .for_each(|(i, (e, a))| assert_eq!(e, a, "index: {}", i));
}

#[test]
fn case1() {
    // (0, 0) (2, 0) (3, 1) (3, 0)
    // (1, 0) (1, 1) (2, 1) (0, 1)
    let grid = Grid::new(4, 2);
    let case = &[
        (grid.pos(0, 1), grid.pos(3, 1)),
        (grid.pos(3, 1), grid.pos(2, 0)),
        (grid.pos(1, 0), grid.pos(0, 1)),
        (grid.pos(2, 0), grid.pos(1, 0)),
    ];
    let expected = vec![
        Operation {
            select: grid.pos(1, 0),
            movements: vec![Right, Right, Right, Up, Left],
        },
        Operation {
            select: grid.pos(0, 1),
            movements: vec![Left, Left, Left],
        },
    ];
    let actual = resolve(&grid, case, 1, 1);
    test_vec(expected, actual);
}

#[test]
fn case2() {
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
        Operation {
            select: grid.pos(0, 0),
            movements: vec![Up, Left, Left],
        },
        Operation {
            select: grid.pos(3, 0),
            movements: vec![Up, Right, Right, Right],
        },
    ];
    let actual = resolve(&grid, case, 1, 1);
    test_vec(expected, actual);
}
