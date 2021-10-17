use std::rc::Rc;

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
        ida_star::ida_star,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResolveParam {
    pub select_limit: u8,
    pub swap_cost: u16,
    pub select_cost: u16,
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を複数求める.
///
/// ```
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
///     1,
///     1,
///     1,
/// );
/// assert_eq!(path.len(), 1);
/// assert_eq!(
///     Operation {
///         select: grid.pos(1, 0),
///         movements: vec![Right],
///     },
///     path[0]
/// );
/// ```
pub(crate) fn resolve(
    grid: Grid,
    movements: &'_ [(Pos, Pos)],
    param: ResolveParam,
) -> impl Iterator<Item = Vec<Operation>> + '_ {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let empty = Rc::new(Board::new(None, nodes));
    let phase1 = Rc::clone(&empty);
    let chain = Rc::clone(&empty);

    beam_search(CostReducer::new(empty.as_ref().clone(), param), 4000, 2000)
        .map(move |(actions, _)| {
            let mut board = phase1.as_ref().clone();
            apply_actions(&mut board, &actions);
            (actions, board)
        })
        .chain(grid.all_pos().map(move |select| {
            let mut board = chain.as_ref().clone();
            board.select(select);
            (vec![GridAction::Select(select)], board)
        }))
        .flat_map(|(mut actions, mut board): (Vec<GridAction>, Board)| {
            let mut solver = Solver {
                threshold_x: 3,
                threshold_y: 3,
                targets_gen: FromOutside,
            };
            let second_actions = solver.solve(board.clone())?;
            actions.extend(second_actions.into_iter());
            apply_actions(&mut board, &actions);
            Some((actions, board))
        })
        .map(move |(mut actions, board): (Vec<GridAction>, Board)| {
            let mut param = param;
            for &action in &actions {
                if let GridAction::Select(_) = action {
                    param.select_limit -= 1;
                }
            }
            let (third_actions, _cost) =
                ida_star(Completer::new(board, param, actions.last().copied()), 0);
            actions.extend(third_actions.into_iter());
            actions_to_operations(actions)
        })
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
