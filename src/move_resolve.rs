use self::{edges_nodes::Nodes, state::actions_to_operations};
use crate::{
    basis::Operation,
    grid::{
        board::{Board, BoardFinder},
        Grid, Pos,
    },
    move_resolve::{
        approx::{gen::FromOutside, Solver},
        beam_search::beam_search,
        state::{completer::Completer, cost_reducer::CostReducer, GridAction},
    },
};

pub mod approx;
pub mod beam_search;
pub mod dijkstra;
pub mod edges_nodes;
pub mod ida_star;
pub mod least_movements;
mod state;
#[cfg(test)]
mod tests;

/// [`GridAction`] 列からその選択回数と交換回数の合計を計算する.
fn actions_counts(ops: &[GridAction]) -> (usize, usize) {
    ops.iter()
        .fold((0, 0), |(selects, swaps), action| match action {
            GridAction::Swap(_) => (selects, swaps + 1),
            GridAction::Select(_) => (selects + 1, swaps),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolveParam {
    pub select_limit: u8,
    pub swap_cost: u16,
    pub select_cost: u16,
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を複数求める.
///
/// ```
/// use procon2021_comp::basis::Operation;
/// use procon2021_comp::grid::{Grid, VecOnGrid};
/// use procon2021_comp::move_resolve::{ResolveParam, resolve};
/// // 10 00
/// let grid = Grid::new(2, 1);
/// let mut field = VecOnGrid::with_init(grid, grid.pos(0, 0));
/// field[grid.pos(0, 0)] = grid.pos(1, 0);
/// field[grid.pos(1, 0)] = grid.pos(0, 0);
/// let path = resolve(
///     grid,
///     &[
///         (grid.pos(0, 0), grid.pos(1, 0)),
///         (grid.pos(1, 0), grid.pos(0, 0)),
///     ],
///     ResolveParam {
///         select_limit: 1,
///         swap_cost: 1,
///         select_cost: 1,
///     },
/// ).next().expect("the solution must be found");
///
/// use procon2021_comp::basis::Movement;
/// assert_eq!(path.len(), 1);
/// assert_eq!(
///     Operation {
///         select: grid.pos(1, 0),
///         movements: vec![Movement::Left],
///     },
///     path[0]
/// );
/// ```
pub fn resolve(
    grid: Grid,
    movements: &'_ [(Pos, Pos)],
    param: ResolveParam,
) -> impl Iterator<Item = Vec<Operation>> + '_ {
    phase1(grid, movements, param, 200)
        .flat_map(phase2)
        .flat_map(phase3(param, 50))
}

fn phase1(
    grid: Grid,
    movements: &[(Pos, Pos)],
    param: ResolveParam,
    beam_width: usize,
) -> impl Iterator<Item = (Vec<GridAction>, Board)> {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let empty = Board::new(None, nodes.clone());
    let phase1 = empty.clone();
    let chain = empty.clone();

    beam_search(CostReducer::new(empty, param), beam_width, 2000)
        .map(move |(actions, _)| {
            let mut board = phase1.clone();
            apply_actions(&mut board, &actions);
            (actions, board)
        })
        .chain(
            grid.all_pos()
                .filter(move |&p| p != nodes[p])
                .map(move |select| {
                    let mut board = chain.clone();
                    board.select(select);
                    (vec![GridAction::Select(select)], board)
                }),
        )
}

fn phase2((mut actions, mut board): (Vec<GridAction>, Board)) -> Option<(Vec<GridAction>, Board)> {
    let grid = board.grid();
    if grid.width() <= 4 && grid.height() <= 4 {
        return Some((actions, board));
    }
    let mut solver = Solver {
        threshold_x: 2,
        threshold_y: 3,
        targets_gen: FromOutside,
    };
    let second_actions = solver.solve(board.clone())?;
    apply_actions(&mut board, &second_actions);
    actions.extend(second_actions.into_iter());
    Some((actions, board))
}

fn phase3(
    param: ResolveParam,
    beam_width: usize,
) -> impl FnMut((Vec<GridAction>, Board)) -> Option<Vec<Operation>> {
    let mut min_cost = 1_000_000_u64;
    move |(mut actions, mut board): (Vec<GridAction>, Board)| {
        let mut param = param;
        let (selects, swaps) = actions_counts(&actions);
        param.select_limit -= selects as u8;
        let cost_until_2nd =
            { selects as u64 * param.select_cost as u64 + swaps as u64 * param.swap_cost as u64 };
        beam_search(
            Completer::new(board.clone(), param, actions.last().copied()),
            beam_width,
            min_cost - cost_until_2nd,
        )
        .next()
        .and_then(|(third_actions, cost)| {
            apply_actions(&mut board, &third_actions);
            debug_assert!(
                board
                    .field()
                    .iter_with_pos()
                    .all(|(pos, &cell)| pos == cell),
                "the board must be completed"
            );

            let cost = cost_until_2nd + cost;
            if cost < min_cost {
                min_cost = cost;
                actions.extend(third_actions.into_iter());
                eprintln!("{:?}", actions);
                Some(actions_to_operations(actions))
            } else {
                None
            }
        })
    }
}

fn apply_actions(board: &mut Board, ops: &[GridAction]) {
    for &op in ops {
        match op {
            GridAction::Swap(mov) => {
                let finder = BoardFinder::new(board.grid());
                let moved = finder.move_pos_to(board.selected().unwrap(), mov);
                board.swap_to(moved);
            }
            GridAction::Select(sel) => board.select(sel),
        }
    }
}
