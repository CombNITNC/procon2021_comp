use std::hash::Hash;
use std::ops::Deref;

use rayon::iter::{ParallelBridge, ParallelIterator};

use self::{
    approx::NextTargetsGenerator,
    edges_nodes::Nodes,
    ida_star::{ida_star, IdaStarState},
};
use crate::{
    basis::{Movement, Operation},
    grid::{
        board::{Board, BoardFinder},
        Grid, Pos, VecOnGrid,
    },
    move_resolve::approx::{gen::FromOutside, Solver},
};

pub mod approx;
pub mod dijkstra;
pub mod edges_nodes;
pub mod ida_star;
pub mod least_movements;
#[cfg(test)]
mod tests;

/// フィールドにあるマスのゴール位置までの距離の合計.
#[derive(Clone, Copy)]
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

#[derive(Clone)]
struct GridCompleter<'b> {
    board: Board<'b>,
    prev_action: Option<GridAction>,
    different_cells: DifferentCells,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridCompleter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("board", &self.board)
            .field("different_cells", &self.different_cells)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GridAction {
    Swap(Movement),
    Select(Pos),
}

impl IdaStarState for GridCompleter<'_> {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selected = self.board.selected();
                let finder = BoardFinder::new(self.board.grid());
                let next_swap = finder.move_pos_to(selected, mov);
                let mut new_board = self.board.clone();
                new_board.swap_to(next_swap);
                Self {
                    board: new_board,
                    different_cells: self.different_cells.on_swap(
                        self.board.field(),
                        selected,
                        next_swap,
                    ),
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => {
                let mut new_board = self.board.clone();
                new_board.select(sel);
                Self {
                    board: new_board,
                    remaining_select: self.remaining_select - 1,
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
        }
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        // 揃っているマスどうしは入れ替えない
        let field = self.board.field();
        let different_cells = field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if self.prev_action.is_none() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selected = self.board.selected();
        let prev = self.prev_action.unwrap();
        let swapping_states = self
            .board
            .around_of(selected)
            .map(|to| Movement::between_pos(selected, to))
            .filter(|&around| {
                if let GridAction::Swap(dir) = prev {
                    around != dir.opposite()
                } else {
                    true
                }
            })
            .map(GridAction::Swap);
        if matches!(prev, GridAction::Swap(_)) && 1 <= self.remaining_select {
            let selecting_states = different_cells
                .filter(|&p| p != selected)
                .map(GridAction::Select);
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        self.different_cells.0 == 0
    }

    type C = u64;
    fn heuristic(&self) -> Self::C {
        (self.different_cells.0 as f64).powf(1.0 + 41.0 / 256.0) as u64
    }

    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.swap_cost as u64,
            GridAction::Select(_) => self.select_cost as u64,
        }
    }
}

/// 操作の履歴 Vec<GridAction> を Vec<Operation> に変換する.
fn actions_to_operations(actions: Vec<GridAction>) -> Vec<Operation> {
    if actions.is_empty() {
        return vec![];
    }
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    for state in actions {
        match state {
            GridAction::Swap(mov) => {
                current_operation.as_mut().unwrap().movements.push(mov);
            }
            GridAction::Select(select) => {
                if let Some(op) = current_operation.replace(Operation {
                    select,
                    movements: vec![],
                }) {
                    operations.push(op);
                }
            }
        }
    }
    if let Some(op) = current_operation {
        operations.push(op);
    }
    operations
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
pub(crate) fn resolve(
    grid: Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let different_cells = DifferentCells::new(&nodes);
    let lower_bound = different_cells.0;

    let (path, cost) = grid
        .all_pos()
        .par_bridge()
        .map(|pos| {
            ida_star(
                GridCompleter {
                    board: Board::new(pos, nodes.clone()),
                    prev_action: None,
                    different_cells,
                    swap_cost,
                    select_cost,
                    remaining_select: select_limit,
                },
                lower_bound,
            )
        })
        .min_by(|a, b| a.1.cmp(&b.1))
        .unwrap();
    println!("move_resolve(strict): cost was {}", cost);
    actions_to_operations(path)
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を求める.
pub(crate) fn resolve_approximately(
    grid: Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
    (threshold_x, threshold_y): (u8, u8),
) -> (Vec<Operation>, u32) {
    let Nodes { nodes, .. } = Nodes::new(grid, movements);
    let operations_cost = |ops: &[Operation]| -> u32 {
        ops.iter()
            .map(|op| op.movements.len() as u32 * swap_cost as u32 + select_cost as u32)
            .sum()
    };
    let result = grid
        .all_pos()
        .par_bridge()
        .map(|pos| {
            resolve_on_select(
                grid,
                nodes.clone(),
                swap_cost,
                select_cost,
                select_limit,
                pos,
                Solver {
                    threshold_x,
                    threshold_y,
                    targets_gen: FromOutside,
                },
            )
        })
        .flatten()
        .min_by(|a, b| operations_cost(a).cmp(&operations_cost(b)))
        .unwrap();
    let cost = operations_cost(&result);
    println!("move_resolve(approx): cost was {}", cost);
    (result, cost)
}

fn resolve_on_select<G: NextTargetsGenerator>(
    grid: Grid,
    mut nodes: VecOnGrid<Pos>,
    swap_cost: u16,
    select_cost: u16,
    mut select_limit: u8,
    init_select: Pos,
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
                select_limit -= 1;
            }
        }
    }
    all_actions.append(&mut actions);

    let different_cells = DifferentCells::new(&nodes);
    let (mut actions, _total_cost) = ida_star(
        GridCompleter {
            board: Board::new(selection, nodes),
            prev_action: all_actions.last().copied(),
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        different_cells.0,
    );
    all_actions.append(&mut actions);
    Some(actions_to_operations(all_actions))
}
