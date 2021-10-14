use self::edges_nodes::Nodes;
use crate::{
    basis::Operation,
    grid::{
        board::{Board, BoardFinder},
        Grid, Pos,
    },
    move_resolve::{
        beam_search::beam_search,
        state::{cost_reducer::CostReducer, GridAction},
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

    beam_search(
        CostReducer::new(Board::new(None, nodes.clone()), param),
        4000,
        2000,
    )
    .map(move |(ops, _)| {
        let mut select = None;
        let mut nodes = nodes.clone();
        for &op in &ops {
            match op {
                GridAction::Swap(mov) => {
                    let finder = BoardFinder::new(grid);
                    let moved = finder.move_pos_to(select.unwrap(), mov);
                    nodes.swap(select.unwrap(), moved);
                    select.replace(moved);
                }
                GridAction::Select(sel) => {
                    select.replace(sel);
                }
            }
        }
        (ops, Board::new(select, nodes))
    })
    .map(|(actions, board): (Vec<GridAction>, Board)| {
        //
        todo!("second phase");
    })
    .map(|(actions, board): (Vec<GridAction>, Board)| {
        //
        todo!("third phase");
    })
}
