use std::hash::Hash;
use std::ops::Deref;

use rayon::iter::{ParallelBridge, ParallelIterator};

use self::{approx::NextTargetsGenerator, edges_nodes::Nodes};
use crate::{
    basis::Operation,
    grid::{
        board::{Board, BoardFinder},
        Grid, Pos, VecOnGrid,
    },
    move_resolve::{
        approx::Solver,
        ida_star::ida_star,
        state::{actions_to_operations, completer::Completer, GridAction},
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

/// フィールドにあるマスのゴール位置までの距離の合計.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct DifferentCells(u64);

impl std::fmt::Debug for DifferentCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DifferentCells {
    fn new(nodes: &VecOnGrid<Pos>) -> Self {
        let mut distances: Vec<_> = nodes
            .iter_with_pos()
            .map(|(p, &n)| nodes.grid.looping_manhattan_dist(p, n) as u64)
            .collect();
        distances.sort_unstable();
        Self(distances.iter().sum())
    }

    /// a の位置と b の位置のマスを入れ替えた場合を計算する.
    fn on_swap(self, field: impl Deref<Target = VecOnGrid<Pos>>, a: Pos, b: Pos) -> Self {
        let before = (field.grid.looping_manhattan_dist(field[a], a)
            + field.grid.looping_manhattan_dist(field[b], b)) as i64;
        let after = (field.grid.looping_manhattan_dist(field[a], b)
            + field.grid.looping_manhattan_dist(field[b], a)) as i64;
        let diff = self.0 as i64 - before + after;
        Self(diff as _)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResolveParam {
    pub select_limit: u8,
    pub swap_cost: u16,
    pub select_cost: u16,
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の最適解を求める.
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
pub(crate) fn resolve(grid: Grid, movements: &[(Pos, Pos)], param: ResolveParam) -> Vec<Operation> {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let different_cells = DifferentCells::new(&nodes);

    let (path, cost) = grid
        .all_pos()
        .par_bridge()
        .map(|pos| {
            ida_star(
                Completer::new(Board::new(pos, nodes.clone()), param, None),
                different_cells.0,
            )
        })
        .min_by(|a, b| a.1.cmp(&b.1))
        .unwrap();
    println!("move_resolve(strict): cost was {}", cost);
    actions_to_operations(path)
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を求める.
pub(crate) fn resolve_approximately<G: NextTargetsGenerator + Clone + Send + Sync>(
    grid: Grid,
    movements: &[(Pos, Pos)],
    param: ResolveParam,
    (threshold_x, threshold_y): (u8, u8),
    max_cost: u32,
    targets_gen: G,
) -> Option<(Vec<Operation>, u32)> {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let operations_cost = |ops: &[Operation]| -> u32 {
        ops.iter()
            .map(|op| op.movements.len() as u32 * param.swap_cost as u32 + param.select_cost as u32)
            .sum()
    };
    let result = grid
        .all_pos()
        .par_bridge()
        .flat_map(move |pos| {
            resolve_on_select(
                grid,
                nodes.clone(),
                param,
                pos,
                max_cost,
                Solver {
                    threshold_x,
                    threshold_y,
                    targets_gen: targets_gen.clone(),
                },
            )
        })
        .min_by(|a, b| operations_cost(a).cmp(&operations_cost(b)))?;
    let cost = operations_cost(&result);
    println!("move_resolve(approx): cost was {}", cost);
    Some((result, cost))
}

fn resolve_on_select<G: NextTargetsGenerator>(
    grid: Grid,
    mut nodes: VecOnGrid<Pos>,
    mut param: ResolveParam,
    init_select: Pos,
    _max_cost: u32,
    mut solver: Solver<G>,
) -> Option<Vec<Operation>> {
    let mut all_actions = vec![];
    let mut selection = init_select;

    let mut actions = solver.solve(init_select, &nodes)?;
    for &action in &actions {
        match action {
            GridAction::Swap(mov) => {
                let to = BoardFinder::new(grid).move_pos_to(selection, mov);
                nodes.swap(selection, to);
                selection = to;
            }
            GridAction::Select(sel) => {
                selection = sel;
                param.select_limit -= 1;
            }
        }
    }
    all_actions.append(&mut actions);

    let (mut actions, _total_cost) = ida_star(
        Completer::new(
            Board::new(selection, nodes),
            param,
            all_actions.last().copied(),
        ),
        100,
    );
    all_actions.append(&mut actions);
    Some(actions_to_operations(all_actions))
}
