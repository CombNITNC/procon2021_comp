use super::{edges_nodes::Nodes, resolve, DifferentCells};
use crate::{
    basis::{Movement::*, Operation},
    grid::{
        board::{Board, BoardFinder},
        Grid, Pos, VecOnGrid,
    },
    move_resolve::{ida_star::ida_star, GridAction, GridCompleter},
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
fn completer_case1() {
    // (00) (10) (32) (41) (31) (50) (60) (70) (80) (90)
    // (01) (11) (40) (21) (42) (51) (61) (71) (81) (91)
    // (02) (12) (30) (20) (22) (52) (62) (72) (82) (92)
    // (03) (13) (23) (33) (43) (53) (63) (73) (83) (93)
    let grid = Grid::new(10, 4);
    let case = &[
        (grid.pos(3, 2), grid.pos(2, 0)),
        (grid.pos(4, 1), grid.pos(3, 0)),
        (grid.pos(3, 1), grid.pos(4, 0)),
        (grid.pos(4, 0), grid.pos(2, 1)),
        (grid.pos(2, 1), grid.pos(3, 1)),
        (grid.pos(4, 2), grid.pos(4, 1)),
        (grid.pos(3, 0), grid.pos(2, 2)),
        (grid.pos(2, 0), grid.pos(3, 2)),
        (grid.pos(2, 2), grid.pos(4, 2)),
    ];
    let Nodes { mut nodes, .. } = Nodes::new(grid, case);
    const SELECT_LIMIT: u8 = 10;
    const SWAP_COST: u16 = 10;
    const SELECT_COST: u16 = 4;

    let selection = grid.pos(3, 2);
    let different_cells = DifferentCells::new(&nodes);
    let mut board = Board::new(selection, nodes.clone());
    grid.range(grid.pos(0, 0), grid.pos(1, 3))
        .chain(grid.range(grid.pos(5, 0), grid.pos(9, 3)))
        .for_each(|pos| {
            board.lock(pos);
        });

    let (actions, _total_cost) = ida_star(
        GridCompleter {
            board,
            prev_action: Some(GridAction::Select(selection)),
            different_cells,
            swap_cost: SWAP_COST,
            select_cost: SELECT_COST,
            remaining_select: SELECT_LIMIT,
        },
        different_cells.0,
    );

    let mut selection = selection;
    let finder = BoardFinder::new(grid);
    for action in actions {
        match action {
            GridAction::Swap(mov) => {
                let dst = finder.move_pos_to(selection, mov);
                nodes.swap(selection, dst);
                selection = dst;
            }
            GridAction::Select(pos) => selection = pos,
        }
    }
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
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
            movements: vec![Right],
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
            movements: vec![Left, Up, Left, Left],
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
