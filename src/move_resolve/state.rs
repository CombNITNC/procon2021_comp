use std::{collections::HashMap, iter::Sum, ops};

use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, VecOnGrid},
};

pub mod completer;
pub mod cost_reducer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SqManhattan(u32);

impl SqManhattan {
    pub fn pre_calc(grid: Grid) -> HashMap<(Pos, Pos), Self> {
        let mut map = HashMap::new();
        for from in grid.all_pos() {
            for to in grid.all_pos() {
                let dist = grid.looping_manhattan_dist(from, to);
                map.insert((from, to), Self(dist * dist));
            }
        }
        map
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn swap_on(
        self,
        pair: (Pos, Pos),
        field: &VecOnGrid<Pos>,
        pre_calc: &HashMap<(Pos, Pos), Self>,
    ) -> Self {
        let prev = pre_calc[&(pair.0, field[pair.0])] + pre_calc[&(pair.1, field[pair.1])];
        let next = pre_calc[&(pair.0, field[pair.1])] + pre_calc[&(pair.1, field[pair.0])];
        self + next - prev
    }
}

impl ops::Add<SqManhattan> for SqManhattan {
    type Output = Self;

    fn add(self, rhs: SqManhattan) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Sub<SqManhattan> for SqManhattan {
    type Output = Self;

    fn sub(self, rhs: SqManhattan) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sum for SqManhattan {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0), |acc, x| acc + x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridAction {
    Swap(Movement),
    Select(Pos),
}

/// 操作の履歴 Vec<GridAction> を Vec<Operation> に変換する.
pub fn actions_to_operations(actions: Vec<GridAction>) -> Vec<Operation> {
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
