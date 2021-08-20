use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, State},
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
    /// a の位置と b の位置のマスを入れ替えた場合を計算する.
    fn on_swap(self, field: &VecOnGrid<Pos>, a: Pos, b: Pos) -> Self {
        let before = unsafe {
            field.get_unchecked(a).manhattan_distance(a)
                + field.get_unchecked(b).manhattan_distance(b)
        } as i64;
        let after = unsafe {
            field.get_unchecked(a).manhattan_distance(b)
                + field.get_unchecked(b).manhattan_distance(a)
        } as i64;
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

impl<'grid> State<u64> for GridState<'grid> {
    type NextStates = Vec<GridState<'grid>>;
    fn next_states(&self, history: &[Self]) -> Vec<GridState<'grid>> {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self
            .field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if history.len() <= 1 {
            return different_cells
                .map(|next_select| self.with_next_select(next_select))
                .collect();
        }
        let selecting = self.selecting.unwrap();
        let prev_prev = unsafe { history.get_unchecked(history.len() - 2) };
        let around = self.grid.around_of(selecting);
        let swapping_states = around
            .iter()
            .cloned()
            .filter(|&around| {
                prev_prev
                    .selecting
                    .map_or(true, |selected| around != selected)
            })
            .map(|next_swap| self.with_next_swap(next_swap));
        if self.is_moved_from(prev_prev) && 1 <= self.remaining_select {
            let selecting_states = different_cells
                .filter(|&p| p != selecting)
                .map(|next_select| self.with_next_select(next_select));
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        self.different_cells.0 == 0
    }

    fn heuristic(&self) -> u64 {
        self.different_cells.0
    }

    fn cost_between(&self, next: &Self) -> u64 {
        if self.selecting.is_none() {
            return self.swap_cost as u64;
        }
        (if next.is_moved_from(self) {
            self.swap_cost
        } else {
            self.select_cost
        }) as u64
    }
}

impl GridState<'_> {
    #[inline]
    fn with_next_select(&self, next_select: Pos) -> Self {
        Self {
            selecting: Some(next_select),
            remaining_select: self.remaining_select - 1,
            ..self.clone()
        }
    }

    #[inline]
    fn with_next_swap(&self, next_swap: Pos) -> Self {
        let selecting = self.selecting.unwrap();
        let mut new_field = self.field.clone();
        new_field.swap(selecting, next_swap);
        Self {
            selecting: Some(next_swap),
            field: new_field,
            different_cells: self
                .different_cells
                .on_swap(&self.field, selecting, next_swap),
            ..self.clone()
        }
    }

    #[inline]
    fn is_moved_from(&self, prev: &Self) -> bool {
        prev.selecting.map_or(true, |prev_selecting| unsafe {
            prev.field.get_unchecked(prev_selecting)
                == self.field.get_unchecked(self.selecting.unwrap())
        })
    }
}

/// 状態の履歴 Vec<GridState> を Vec<Operation> に変換する.
fn path_to_operations(path: Vec<GridState>) -> Vec<Operation> {
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    let mut prev = &path[0];
    for state in &path[1..] {
        let is_swapped = (&prev.field)
            .into_iter()
            .zip(&state.field)
            .any(|(a, b)| a != b);
        if is_swapped {
            let movement = Movement::between_pos(prev.selecting.unwrap(), state.selecting.unwrap());
            current_operation.as_mut().unwrap().movements.push(movement);
        } else if let Some(op) = current_operation.replace(Operation {
            select: state.selecting.unwrap(),
            movements: vec![],
        }) {
            operations.push(op);
        }
        prev = state;
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
    let lower_bound = {
        let mut distances: Vec<_> = nodes
            .iter_with_pos()
            .map(|(p, &n)| p.manhattan_distance(n) as u64)
            .collect();
        distances.sort_unstable();
        distances.iter().sum()
    };
    let different_cells = DifferentCells(lower_bound);
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
    path_to_operations(path)
}
