use super::{edges_nodes::Nodes, resolve};
use crate::{
    basis::{Movement::*, Operation},
    grid::{board::BoardFinder, Grid, Pos, VecOnGrid},
    move_resolve::{state::SqManhattan, ResolveParam},
};

#[test]
fn test_sq_manhattan() {
    let grid = Grid::new(2, 2);
    let pre_calc = SqManhattan::pre_calc(grid);
    assert_eq!(pre_calc[&(grid.pos(0, 1), grid.pos(1, 1))].as_u32(), 1);
    assert_eq!(pre_calc[&(grid.pos(0, 0), grid.pos(1, 1))].as_u32(), 4);
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
        ResolveParam {
            select_limit: 1,
            swap_cost: 1,
            select_cost: 1,
        },
    )
    .next()
    .unwrap();
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
    let movements = &[
        (grid.pos(1, 0), grid.pos(0, 1)),
        (grid.pos(0, 1), grid.pos(1, 1)),
        (grid.pos(1, 1), grid.pos(1, 0)),
    ];

    let actual = resolve(
        grid,
        movements,
        ResolveParam {
            select_limit: 1,
            swap_cost: 1,
            select_cost: 1,
        },
    );
    test_answers(1, 2, actual);
}

fn test_answers(
    select_count: usize,
    swap_count: usize,
    actual_gen: impl Iterator<Item = Vec<Operation>>,
) {
    assert!(actual_gen.into_iter().any(|actual| {
        actual.into_iter().fold((0, 0), |(selects, swaps), op| {
            (selects + 1, swaps + op.movements.len())
        }) == (select_count, swap_count)
    }));
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

    let actual = resolve(
        grid,
        case,
        ResolveParam {
            select_limit: 2,
            swap_cost: 1,
            select_cost: 2,
        },
    );
    test_answers(2, 5, actual);
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

    let actual = resolve(
        grid,
        case,
        ResolveParam {
            select_limit: 2,
            swap_cost: 1,
            select_cost: 1,
        },
    );
    test_answers(2, 3, actual);
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

    let actual = resolve(
        grid,
        case,
        ResolveParam {
            select_limit: 2,
            swap_cost: 2,
            select_cost: 3,
        },
    );
    test_answers(2, 5, actual);
}

#[test]
fn case4() {
    // 00 11 02
    // 01 20 21
    // 12 10 22
    let grid = Grid::new(3, 3);
    let case = &[
        (grid.pos(1, 1), grid.pos(1, 0)),
        (grid.pos(0, 2), grid.pos(2, 0)),
        (grid.pos(2, 0), grid.pos(1, 1)),
        (grid.pos(1, 2), grid.pos(0, 2)),
        (grid.pos(1, 0), grid.pos(1, 2)),
    ];

    let actual = resolve(
        grid,
        case,
        ResolveParam {
            select_limit: 8,
            swap_cost: 1,
            select_cost: 8,
        },
    );
    test_answers(1, 8, actual);
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
    const PARAM: ResolveParam = ResolveParam {
        select_limit: 3,
        swap_cost: 1,
        select_cost: 8,
    };

    let result = resolve(grid, case, PARAM).next().unwrap();

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
    const PARAM: ResolveParam = ResolveParam {
        select_limit: 3,
        swap_cost: 1,
        select_cost: 8,
    };

    let result = resolve(grid, case, PARAM).next().unwrap();

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
    const PARAM: ResolveParam = ResolveParam {
        select_limit: 10,
        swap_cost: 10,
        select_cost: 4,
    };

    let result = resolve(grid, case, PARAM).next().unwrap();

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
    const PARAM: ResolveParam = ResolveParam {
        select_limit: 8,
        swap_cost: 1,
        select_cost: 8,
    };
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

    let result = resolve(grid, &case, PARAM).next().unwrap();

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
