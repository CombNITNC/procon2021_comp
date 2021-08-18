use super::{edges_nodes::EdgesNodes, resolve, DifferentCells};
use crate::{
    basis::{Movement::*, Operation},
    grid::{Grid, Pos, VecOnGrid},
};

#[test]
fn test_different_cells() {
    // (0, 0) (1, 1)
    // (1, 0) (0, 1)
    let grid = Grid::new(2, 2);
    let case = &[
        (grid.pos(0, 1), grid.pos(1, 1)),
        (grid.pos(1, 0), grid.pos(0, 1)),
        (grid.pos(1, 1), grid.pos(1, 0)),
    ];
    let EdgesNodes { nodes: field, .. } = EdgesNodes::new(&grid, case);

    let diff = DifferentCells(3);
    assert_eq!(diff.on_swap(&field, grid.pos(0, 1), grid.pos(1, 1)).0, 2);
    assert_eq!(diff.on_swap(&field, grid.pos(0, 1), grid.pos(0, 0)).0, 4);
}

#[test]
fn smallest_case() {
    // 10 00
    let grid = Grid::new(2, 1);
    let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(1, 0);
    field[grid.pos(1, 0)] = grid.pos(0, 0);

    let path = resolve(
        &grid,
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
        path[0],
        Operation {
            select: grid.pos(0, 0),
            movements: vec![Right],
        }
    );
}

#[test]
fn simple_case() {
    // 00 11
    // 10 01
    let grid = Grid::new(2, 2);
    let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
    field[grid.pos(0, 0)] = grid.pos(0, 0);
    field[grid.pos(1, 0)] = grid.pos(1, 1);
    field[grid.pos(0, 1)] = grid.pos(1, 0);
    field[grid.pos(1, 1)] = grid.pos(0, 1);

    let path = resolve(
        &grid,
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
            select: grid.pos(2, 0),
            movements: vec![Down],
        },
        Operation {
            select: grid.pos(0, 1),
            movements: vec![Left, Left, Up, Left],
        },
    ];
    let actual = resolve(&grid, case, 2, 1, 2);
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
            movements: vec![Down],
        },
        Operation {
            select: grid.pos(3, 1),
            movements: vec![Up, Right],
        },
    ];
    let actual = resolve(&grid, case, 2, 1, 1);
    test_vec(expected, actual);
}

#[test]
fn rand_case() {
    fn gen_circular(grid: &Grid, rng: &mut rand::rngs::ThreadRng) -> Vec<Pos> {
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
    const SELECT_LIMIT: u8 = 3;
    const SWAP_COST: u16 = 1;
    const SELECT_COST: u16 = 2;
    let mut rng = rand::thread_rng();

    let grid = Grid::new(WIDTH, HEIGHT);
    let circular = gen_circular(&grid, &mut rng);
    let mut case = vec![];
    for pair in circular.windows(2) {
        case.push((pair[0], pair[1]));
    }
    case.push((*circular.last().unwrap(), *circular.first().unwrap()));
    let result = resolve(&grid, &case, SELECT_LIMIT, SWAP_COST, SELECT_COST);

    let EdgesNodes { mut nodes, .. } = EdgesNodes::new(&grid, &case);
    for Operation { select, movements } in result {
        let mut current = select;
        for movement in movements {
            let to_swap = match movement {
                Up => grid.up_of(current),
                Right => grid.right_of(current),
                Down => grid.down_of(current),
                Left => grid.left_of(current),
            };
            nodes.swap(current, to_swap);
            current = to_swap;
        }
    }
    assert!(grid.all_pos().zip(nodes.into_iter()).all(|(p, n)| p == n));
}
