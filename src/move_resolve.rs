use self::ida_star::{ida_star, State};
use crate::{
    basis::{Movement, Operation},
    grid::{Cycles, Grid, Pos},
};

pub mod edges_nodes;
pub mod ida_star;
#[cfg(test)]
mod tests;

#[derive(Clone)]
struct GridState<'grid> {
    cycles: Cycles<'grid>,
    selecting: Option<Pos>,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("cycles", &self.cycles)
            .field("selecting", &self.selecting)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

impl PartialEq for GridState<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cycles == other.cycles && self.selecting == other.selecting
    }
}

impl<'grid> State<u64> for GridState<'grid> {
    type NextStates = Vec<GridState<'grid>>;
    fn next_states(&self, history: &[Self]) -> Vec<GridState<'grid>> {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self.cycles.different_cells();
        if history.len() <= 1 {
            return different_cells
                .map(|next_select| self.with_next_select(next_select))
                .collect();
        }
        let selecting = self.selecting.unwrap();
        let prev_prev = &history[history.len() - 2];
        let around = self.cycles.grid().around_of(selecting);
        let swapping_states = around
            .iter()
            .cloned()
            .filter(|&around| {
                selecting != around
                    && prev_prev
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
        self.cycles.scatter_amount() == 0
    }

    fn heuristic(&self) -> u64 {
        self.cycles.scatter_amount()
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
        let mut new_cycles = self.cycles.clone();
        new_cycles.on_swap(selecting, next_swap);
        Self {
            selecting: Some(next_swap),
            cycles: new_cycles,
            ..self.clone()
        }
    }

    #[inline]
    fn is_moved_from(&self, prev: &Self) -> bool {
        prev.cycles.tree_count() != self.cycles.tree_count()
    }
}

/// 状態の履歴 Vec<GridState> を Vec<Operation> に変換する.
fn path_to_operations(path: Vec<GridState>) -> Vec<Operation> {
    if path.is_empty() {
        return vec![];
    }
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    let mut prev = &path[0];
    for state in &path[1..] {
        let is_swapped = prev.cycles != state.cycles;
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
    let mut min = (vec![], 1 << 60);
    for (total_path, total_cost) in ida_star(vec![GridState {
        cycles: Cycles::new(grid, movements),
        selecting: None,
        swap_cost,
        select_cost,
        remaining_select: select_limit,
    }]) {
        if !total_path.is_empty() && total_cost < min.1 {
            min = (total_path, total_cost);
        } else {
            break;
        }
    }
    path_to_operations(min.0)
}
