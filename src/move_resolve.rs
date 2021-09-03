use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, IdaStarState},
};
use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, VecOnGrid},
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
struct GridState<'grid> {
    grid: &'grid Grid,
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    different_cells: DifferentCells,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .field("different_cells", &self.different_cells)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

impl PartialEq for GridState<'_> {
    fn eq(&self, other: &Self) -> bool {
        (&self.field)
            .into_iter()
            .zip(&other.field)
            .all(|(a, b)| a == b)
            && self.selecting == other.selecting
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridAction {
    Swap(Movement),
    Select(Pos),
}

impl<'grid> IdaStarState for GridState<'grid> {
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
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => Self {
                selecting: Some(sel),
                remaining_select: self.remaining_select - 1,
                ..self.clone()
            },
        }
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self, history: &[Self::A]) -> Self::AS {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self
            .field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if history.is_empty() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selecting = self.selecting.unwrap();
        let prev = history.last().unwrap();
        let swapping_states = [
            Movement::Up,
            Movement::Right,
            Movement::Down,
            Movement::Left,
        ]
        .iter()
        .cloned()
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

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { nodes, .. } = EdgesNodes::new(grid, movements);
    let different_cells = DifferentCells::new(&nodes);
    let lower_bound = different_cells.0;
    // 600e8 = (WH)^select => select = 10 log 6 / log WH
    let maximum_select =
        (10.0 * 6.0f64.log2() / (grid.width() as f64 + grid.height() as f64).log2()).ceil() as u8;
    let (path, _total_cost) = ida_star(
        GridState {
            grid,
            field: nodes.clone(),
            selecting: None,
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit.min(maximum_select),
        },
        lower_bound,
    );
    actions_to_operations(path)
}
