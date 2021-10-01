use super::{edges_nodes::Nodes, resolve, DifferentCells};
use crate::{
    basis::{Movement::*, Operation},
    grid::{board::BoardFinder, Grid, Pos, VecOnGrid},
};

#[test]
fn test_different_cells() {
    // 0(0, 0) 1(1, 1)
    // 2(1, 0) 1(0, 1)
    let grid = Grid::new(2, 2);
    let case = &[
        (grid.pos(0, 1), grid.pos(1, 1)),
        (grid.pos(1, 0), grid.pos(0, 1)),
        (grid.pos(1, 1), grid.pos(1, 0)),
    ];
    let Nodes { nodes: field, .. } = Nodes::new(grid, case);

    let diff = DifferentCells(4);
    assert_eq!(diff.on_swap(&field, grid.pos(0, 1), grid.pos(1, 1)).0, 2);
    assert_eq!(diff.on_swap(&field, grid.pos(0, 1), grid.pos(0, 0)).0, 4);
}

#[test]
fn smallest_case() {
    // 10 00
    let grid = Grid::new(2, 1);
    let mut field = VecOnGrid::with_init(grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(1, 0);
    field[grid.pos(1, 0)] = grid.pos(0, 0);

    let path = resolve(
        grid,
        &[
            (grid.pos(0, 0), grid.pos(1, 0)),
            (grid.pos(1, 0), grid.pos(0, 0)),
        ],
        1,
        1,
        1,
    );
    assert_eq!(path.len(), 1);
    assert_eq!(
        Operation {
            select: grid.pos(1, 0),
            movements: vec![Left],
        },
        path[0]
    );
}

#[test]
fn simple_case() {
    // 00 11
    // 10 01
    let grid = Grid::new(2, 2);
    let mut field = VecOnGrid::with_init(grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(0, 0);
    field[grid.pos(1, 0)] = grid.pos(1, 1);
    field[grid.pos(0, 1)] = grid.pos(1, 0);
    field[grid.pos(1, 1)] = grid.pos(0, 1);

    let path = resolve(
        grid,
        &[
            (grid.pos(1, 0), grid.pos(0, 1)),
            (grid.pos(0, 1), grid.pos(1, 1)),
            (grid.pos(1, 1), grid.pos(1, 0)),
        ],
        1,
        1,
        1,
    );
    assert_eq!(path.len(), 1);
    assert_eq!(
        Operation {
            select: grid.pos(0, 1),
            movements: vec![Right, Up],
        },
        path[0]
    );
}

fn test_vec<E, A, T>(expected: E, actual: A)
where
    E: IntoIterator<Item = T>,
    A: IntoIterator<Item = T>,
    T: PartialEq + std::fmt::Debug,
    E::IntoIter: ExactSizeIterator + std::fmt::Debug,
    A::IntoIter: ExactSizeIterator + std::fmt::Debug,
{
    let expected = expected.into_iter();
    let actual = actual.into_iter();
    assert_eq!(
        expected.len(),
        actual.len(),
        "expected: {:?}\nactual: {:?}",
        expected,
        actual
    );
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
            select: grid.pos(2, 0),
            movements: vec![Left, Down, Left, Left],
        },
        Operation {
            select: grid.pos(1, 1),
            movements: vec![Up],
        },
    ];
    let actual = resolve(grid, case, 2, 1, 2);
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
            select: grid.pos(0, 1),
            movements: vec![Up],
        },
        Operation {
            select: grid.pos(3, 1),
            movements: vec![Up, Right],
        },
    ];
    let actual = resolve(grid, case, 2, 1, 1);
    test_vec(expected, actual);
}

#[test]
fn case3() {
    // (2, 0) (0, 1) (1, 0)
    // (2, 1) (0, 0) (1, 1)
    let grid = Grid::new(3, 2);
    let case = &[
        (grid.pos(0, 0), grid.pos(1, 1)),
        (grid.pos(1, 0), grid.pos(2, 0)),
        (grid.pos(2, 0), grid.pos(0, 0)),
        (grid.pos(0, 1), grid.pos(1, 0)),
        (grid.pos(1, 1), grid.pos(2, 1)),
        (grid.pos(2, 1), grid.pos(0, 1)),
    ];
    let expected = vec![
        Operation {
            select: grid.pos(2, 0),
            movements: vec![Right, Right],
        },
        Operation {
            select: grid.pos(1, 1),
            movements: vec![Right, Right, Up],
        },
    ];
    let actual = resolve(grid, case, 2, 2, 3);
    test_vec(expected, actual);
}

#[test]
fn large_case1() {
    // 00 10 20 55 40 50
    // 01 30 21 31 41 51
    // 02 12 22 32 42 52
    // 03 13 23 33 43 53
    // 04 14 24 34 44 54
    // 05 15 25 35 45 11
    let grid = Grid::new(6, 6);
    let case = &[
        (grid.pos(5, 5), grid.pos(3, 0)),
        (grid.pos(3, 0), grid.pos(1, 1)),
        (grid.pos(1, 1), grid.pos(5, 5)),
    ];
    let Nodes { mut nodes, .. } = Nodes::new(grid, case);
    const SELECT_LIMIT: u8 = 3;
    const SWAP_COST: u16 = 1;
    const SELECT_COST: u16 = 8;

    let result = resolve(grid, case, SELECT_LIMIT, SWAP_COST, SELECT_COST);

    let finder = BoardFinder::new(grid);
    for Operation { select, movements } in result {
        let mut current = select;
        for movement in movements {
            let to_swap = finder.move_pos_to(current, movement);
            nodes.swap(current, to_swap);
            current = to_swap;
        }
    }
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
}

#[test]
fn large_case2() {
    // 10 20 30 40 50 00
    // 11 21 31 41 51 01
    // 12 22 32 42 52 02
    // 13 23 33 43 53 03
    // 14 24 34 44 54 04
    // 15 25 35 45 55 05
    let grid = Grid::new(6, 6);
    let case = &[
        (grid.pos(1, 0), grid.pos(0, 0)),
        (grid.pos(1, 1), grid.pos(0, 1)),
        (grid.pos(1, 2), grid.pos(0, 2)),
        (grid.pos(1, 3), grid.pos(0, 3)),
        (grid.pos(1, 4), grid.pos(0, 4)),
        (grid.pos(1, 5), grid.pos(0, 5)),
        (grid.pos(2, 0), grid.pos(1, 0)),
        (grid.pos(2, 1), grid.pos(1, 1)),
        (grid.pos(2, 2), grid.pos(1, 2)),
        (grid.pos(2, 3), grid.pos(1, 3)),
        (grid.pos(2, 4), grid.pos(1, 4)),
        (grid.pos(2, 5), grid.pos(1, 5)),
        (grid.pos(3, 0), grid.pos(2, 0)),
        (grid.pos(3, 1), grid.pos(2, 1)),
        (grid.pos(3, 2), grid.pos(2, 2)),
        (grid.pos(3, 3), grid.pos(2, 3)),
        (grid.pos(3, 4), grid.pos(2, 4)),
        (grid.pos(3, 5), grid.pos(2, 5)),
        (grid.pos(4, 0), grid.pos(3, 0)),
        (grid.pos(4, 1), grid.pos(3, 1)),
        (grid.pos(4, 2), grid.pos(3, 2)),
        (grid.pos(4, 3), grid.pos(3, 3)),
        (grid.pos(4, 4), grid.pos(3, 4)),
        (grid.pos(4, 5), grid.pos(3, 5)),
        (grid.pos(5, 0), grid.pos(4, 0)),
        (grid.pos(5, 1), grid.pos(4, 1)),
        (grid.pos(5, 2), grid.pos(4, 2)),
        (grid.pos(5, 3), grid.pos(4, 3)),
        (grid.pos(5, 4), grid.pos(4, 4)),
        (grid.pos(5, 5), grid.pos(4, 5)),
        (grid.pos(0, 0), grid.pos(5, 0)),
        (grid.pos(0, 1), grid.pos(5, 1)),
        (grid.pos(0, 2), grid.pos(5, 2)),
        (grid.pos(0, 3), grid.pos(5, 3)),
        (grid.pos(0, 4), grid.pos(5, 4)),
        (grid.pos(0, 5), grid.pos(5, 5)),
    ];
    let Nodes { mut nodes, .. } = Nodes::new(grid, case);
    const SELECT_LIMIT: u8 = 3;
    const SWAP_COST: u16 = 1;
    const SELECT_COST: u16 = 8;

    let result = resolve(grid, case, SELECT_LIMIT, SWAP_COST, SELECT_COST);

    let finder = BoardFinder::new(grid);
    for Operation { select, movements } in result {
        let mut current = select;
        for movement in movements {
            let to_swap = finder.move_pos_to(current, movement);
            nodes.swap(current, to_swap);
            current = to_swap;
        }
    }
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
}

#[test]
fn large_case3() {
    // test_cases/03.ppm の movements が元です.
    let grid = Grid::new(10, 4);
    let case = &[
        (grid.pos(0, 0), grid.pos(8, 0)),
        (grid.pos(1, 0), grid.pos(8, 1)),
        (grid.pos(2, 0), grid.pos(6, 1)),
        (grid.pos(3, 0), grid.pos(7, 3)),
        (grid.pos(4, 0), grid.pos(7, 1)),
        (grid.pos(5, 0), grid.pos(4, 2)),
        (grid.pos(6, 0), grid.pos(9, 0)),
        (grid.pos(7, 0), grid.pos(2, 1)),
        (grid.pos(8, 0), grid.pos(9, 3)),
        (grid.pos(9, 0), grid.pos(2, 0)),
        (grid.pos(0, 1), grid.pos(1, 0)),
        (grid.pos(1, 1), grid.pos(2, 3)),
        (grid.pos(2, 1), grid.pos(4, 3)),
        (grid.pos(3, 1), grid.pos(9, 2)),
        (grid.pos(4, 1), grid.pos(3, 2)),
        (grid.pos(5, 1), grid.pos(8, 3)),
        (grid.pos(6, 1), grid.pos(1, 2)),
        (grid.pos(7, 1), grid.pos(0, 1)),
        (grid.pos(8, 1), grid.pos(5, 1)),
        (grid.pos(9, 1), grid.pos(2, 2)),
        (grid.pos(0, 2), grid.pos(0, 0)),
        (grid.pos(1, 2), grid.pos(8, 2)),
        (grid.pos(2, 2), grid.pos(0, 2)),
        (grid.pos(3, 2), grid.pos(0, 3)),
        (grid.pos(4, 2), grid.pos(7, 2)),
        (grid.pos(6, 2), grid.pos(4, 1)),
        (grid.pos(7, 2), grid.pos(3, 1)),
        (grid.pos(8, 2), grid.pos(9, 1)),
        (grid.pos(9, 2), grid.pos(6, 0)),
        (grid.pos(0, 3), grid.pos(7, 0)),
        (grid.pos(1, 3), grid.pos(5, 0)),
        (grid.pos(2, 3), grid.pos(1, 1)),
        (grid.pos(3, 3), grid.pos(4, 0)),
        (grid.pos(4, 3), grid.pos(3, 0)),
        (grid.pos(5, 3), grid.pos(6, 3)),
        (grid.pos(6, 3), grid.pos(6, 2)),
        (grid.pos(7, 3), grid.pos(5, 3)),
        (grid.pos(8, 3), grid.pos(1, 3)),
        (grid.pos(9, 3), grid.pos(3, 3)),
    ];
    let Nodes { mut nodes, .. } = Nodes::new(grid, case);
    const SELECT_LIMIT: u8 = 10;
    const SWAP_COST: u16 = 10;
    const SELECT_COST: u16 = 4;

    let result = resolve(grid, case, SELECT_LIMIT, SWAP_COST, SELECT_COST);

    let finder = BoardFinder::new(grid);
    for Operation { select, movements } in result {
        let mut current = select;
        for movement in movements {
            let to_swap = finder.move_pos_to(current, movement);
            nodes.swap(current, to_swap);
            current = to_swap;
        }
    }
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
}

#[test]
fn rand_case() {
    fn gen_circular(grid: Grid, rng: &mut rand::rngs::ThreadRng) -> Vec<Pos> {
        use rand::{
            distributions::{Distribution, Uniform},
            seq::SliceRandom,
        };
        let mut points: Vec<_> = grid.all_pos().collect();
        points.shuffle(rng);
        let between = Uniform::from(2..points.len());
        let taking = between.sample(rng);
        points.into_iter().take(taking).collect()
    }
    const WIDTH: u8 = 16;
    const HEIGHT: u8 = 16;
    const SELECT_LIMIT: u8 = 8;
    const SWAP_COST: u16 = 1;
    const SELECT_COST: u16 = 8;
    let mut rng = rand::thread_rng();

    let grid = Grid::new(WIDTH, HEIGHT);
    let circular = gen_circular(grid, &mut rng);
    let mut case = vec![];
    for pair in circular.windows(2) {
        case.push((pair[0], pair[1]));
    }
    case.push((*circular.last().unwrap(), *circular.first().unwrap()));

    let Nodes { mut nodes, .. } = Nodes::new(grid, &case);
    eprintln!("before: {:#?}", nodes);

    let result = resolve(grid, &case, SELECT_LIMIT, SWAP_COST, SELECT_COST);

    let finder = BoardFinder::new(grid);
    eprintln!("operations: {:?}", result);
    for Operation { select, movements } in result {
        let mut current = select;
        for movement in movements {
            let to_swap = finder.move_pos_to(current, movement);
            nodes.swap(current, to_swap);
            current = to_swap;
        }
    }
    eprintln!("after: {:#?}", nodes);
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
}
