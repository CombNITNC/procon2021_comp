use std::{collections::BinaryHeap, ops};

use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, IdaStarState},
};
use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, RangePos, VecOnGrid},
};

pub mod edges_nodes;
pub mod ida_star;
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
    fn on_swap(self, field: &VecOnGrid<Pos>, a: Pos, b: Pos) -> Self {
        let before = (field.grid.looping_manhattan_dist(field[a], a)
            + field.grid.looping_manhattan_dist(field[b], b)) as i64;
        let after = (field.grid.looping_manhattan_dist(field[a], b)
            + field.grid.looping_manhattan_dist(field[b], a)) as i64;
        let diff = self.0 as i64 - before + after;
        Self(diff as _)
    }
}

#[derive(Clone)]
struct GridCompleter<'grid> {
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    prev_action: Option<GridAction>,
    different_cells: DifferentCells,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridCompleter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .field("different_cells", &self.different_cells)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridAction {
    Swap(Movement),
    Select(Pos),
}

impl<'grid> IdaStarState for GridCompleter<'grid> {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selecting = self.selecting.unwrap();
                let next_swap = self.field.grid.move_pos_to(selecting, mov);
                let mut new_field = self.field.clone();
                new_field.swap(selecting, next_swap);
                Self {
                    selecting: Some(next_swap),
                    field: new_field,
                    different_cells: self.different_cells.on_swap(
                        &self.field,
                        selecting,
                        next_swap,
                    ),
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => Self {
                selecting: Some(sel),
                remaining_select: self.remaining_select - 1,
                prev_action: Some(action),
                ..self.clone()
            },
        }
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self
            .field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if self.selecting.is_none() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selecting = self.selecting.unwrap();
        let prev = self.prev_action.unwrap();
        let swapping_states = [
            Movement::Up,
            Movement::Right,
            Movement::Down,
            Movement::Left,
        ]
        .iter()
        .copied()
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
                .filter(|&p| p != selecting)
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
/// let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
/// field[grid.pos(0, 0)] = grid.pos(1, 0);
/// field[grid.pos(1, 0)] = grid.pos(0, 0);
/// let path = resolve(
///     &grid,
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
    grid: &Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    if 36 <= grid.width() * grid.height() {
        return resolve_approximately(grid, movements, select_limit, swap_cost, select_cost);
    }
    let EdgesNodes { nodes, .. } = EdgesNodes::new(grid, movements);
    let different_cells = DifferentCells::new(&nodes);
    let lower_bound = different_cells.0;
    let (path, _total_cost) = ida_star(
        GridCompleter {
            field: nodes.clone(),
            selecting: None,
            prev_action: None,
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        lower_bound,
    );
    actions_to_operations(path)
}

fn least_movements((dx, dy): (i32, i32)) -> u32 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let dx = dx.abs();
    let dy = dy.abs();
    let d = (dx - dy).unsigned_abs();
    let min = dx.min(dy) as u32;
    let mut ret = 5 * d + 6 * min - 4;
    if dx == dy {
        ret += 2;
    }
    ret
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LeastMovements(u32);

impl LeastMovements {
    fn new(field: &VecOnGrid<Pos>) -> Self {
        Self(
            field
                .iter_with_pos()
                .map(|(pos, &to)| field.grid.looping_min_vec(pos, to))
                .map(least_movements)
                .sum(),
        )
    }

    fn move_on(self, field: &VecOnGrid<Pos>, from: Pos, to: Pos) -> Self {
        let before = least_movements(field.grid.looping_min_vec(from, field[from]));
        let after = least_movements(field.grid.looping_min_vec(to, field[from]));
        Self(4 + self.0 - before + after)
    }
}

impl ops::Add for LeastMovements {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MoveNode {
    pos: Pos,
    cost: LeastMovements,
}
impl PartialOrd for MoveNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}
impl Ord for MoveNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

fn path_to_move_select_around_target(
    field: &VecOnGrid<Pos>,
    target: Pos,
    select: Pos,
) -> Vec<GridAction> {
    // ダイクストラ法で select を target の隣へ動かす経路を決定する.
    // コストは各マスの必要最低手数の合計.
    let mut shortest_cost = VecOnGrid::with_init(field.grid, LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(field.grid, None);

    let mut heap = BinaryHeap::new();
    heap.push(MoveNode {
        pos: select,
        cost: LeastMovements(0),
    });
    shortest_cost[select] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.pos] != pick.cost {
            continue;
        }
        if field.grid.looping_manhattan_dist(pick.pos, target) == 1 {
            return extract_back_path(pick.pos, back_path);
        }
        for next in field.grid.around_of(pick.pos) {
            let next_cost = pick.cost.move_on(field, pick.pos, next) + LeastMovements(1);
            if shortest_cost[next] <= next_cost {
                continue;
            }
            shortest_cost[next] = next_cost;
            back_path[next] = Some(pick.pos);
            heap.push(MoveNode {
                pos: next,
                cost: next_cost,
            });
        }
    }
    vec![]
}

fn extract_back_path(mut pos: Pos, back_path: VecOnGrid<Option<Pos>>) -> Vec<GridAction> {
    let mut history = vec![pos];
    while let Some(back) = back_path[pos] {
        history.push(back);
        pos = back;
    }
    history.reverse();
    history
        .windows(2)
        .map(|mov| Movement::between_pos(mov[0], mov[1]))
        .map(GridAction::Swap)
        .collect()
}

fn path_to_move_target_to_goal(field: &VecOnGrid<Pos>, target: Pos) -> Vec<GridAction> {
    // ダイクストラ法で target をゴール位置へ動かす経路を決定する.
    // コストは各マスの必要最低手数の合計.
    todo!()
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を求める.
fn resolve_approximately(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    mut select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { mut nodes, .. } = EdgesNodes::new(grid, movements);
    let mut all_actions = vec![];
    let mut selection = None;

    let mut row_to_sort: Vec<_> = (0..grid.height()).collect();
    for _ in 0..grid.height() - 1 {
        let cost_to_sort_row = |y: u8| -> u32 {
            (0..grid.width())
                .map(move |x| grid.clamping_pos(x, y))
                .map(|pos| grid.looping_manhattan_dist(pos, nodes[pos]))
                .sum()
        };
        row_to_sort.sort_by(|&a, &b| cost_to_sort_row(a).cmp(&cost_to_sort_row(b)));
        let y = row_to_sort.pop().unwrap();

        for x in 0..grid.width() {
            let target = grid.clamping_pos(x, y);
            if target != nodes[target] {
                let mut actions = path_to_move_target_to_goal(&nodes, target);
                all_actions.append(&mut actions);
            }
        }

        eprintln!("sort result: {:?}", nodes);
    }
    let different_cells = DifferentCells::new(&nodes);
    let (mut actions, _total_cost) = ida_star(
        GridCompleter {
            field: nodes.clone(),
            selecting: selection,
            prev_action: all_actions.last().copied(),
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        different_cells.0,
    );
    all_actions.append(&mut actions);
    actions_to_operations(all_actions)
}
