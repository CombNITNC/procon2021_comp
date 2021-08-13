use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, State},
};
use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, VecOnGrid},
};

mod edges_nodes;
mod ida_star;
#[cfg(test)]
mod tests;

#[derive(Clone)]
struct GridState<'grid> {
    grid: &'grid Grid,
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    different_cells: u8,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
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
        let different_cells = self.grid.all_pos().filter(|&p| p != self.field[p]);
        if history.len() <= 1 {
            return different_cells
                .map(|next_select| Self {
                    selecting: Some(next_select),
                    remaining_select: self.remaining_select - 1,
                    ..self.clone()
                })
                .collect();
        }
        let selecting = self.selecting.unwrap();
        let prev_prev = &history[history.len() - 2];
        let swapping_states = self
            .grid
            .around_of(selecting)
            .into_iter()
            .filter(|&around| around != self.selecting.unwrap())
            .map(|next_swap| {
                let different_cells = (self.different_cells as i32
                    + if next_swap == self.field[next_swap] {
                        1
                    } else if selecting == self.field[next_swap] {
                        -1
                    } else {
                        0
                    }) as u8;
                let mut new_field = self.field.clone();
                new_field.swap(selecting, next_swap);
                Self {
                    selecting: Some(next_swap),
                    field: new_field,
                    different_cells,
                    ..self.clone()
                }
            });
        let moved_in_prev = self
            .field
            .iter()
            .zip(prev_prev.field.iter())
            .any(|(a, b)| a != b);
        if moved_in_prev && 1 <= self.remaining_select {
            let selecting_states = different_cells
                .filter(|&p| p != selecting)
                .map(|next_select| Self {
                    selecting: Some(next_select),
                    remaining_select: self.remaining_select - 1,
                    ..self.clone()
                });
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        self.different_cells == 0
    }

    fn heuristic(&self) -> u64 {
        self.different_cells as u64
    }

    fn cost_between(&self, next: &Self) -> u64 {
        (if (&self.field)
            .into_iter()
            .zip((&next.field).into_iter())
            .all(|(a, b)| a == b)
        {
            self.select_cost
        } else {
            self.swap_cost
        }) as u64
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
    let (path, _total_cost) = ida_star(GridState {
        grid,
        field: nodes.clone(),
        selecting: None,
        different_cells: grid
            .all_pos()
            .zip(nodes.iter())
            .filter(|&(p, &n)| p != n)
            .count() as u8,
        swap_cost,
        select_cost,
        remaining_select: select_limit,
    });
    path_to_operations(path)
}
