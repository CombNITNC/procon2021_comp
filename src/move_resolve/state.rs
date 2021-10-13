use crate::{
    basis::{Movement, Operation},
    grid::Pos,
};

pub mod completer;
pub mod cost_reducer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GridAction {
    Swap(Movement),
    Select(Pos),
}

/// 操作の履歴 Vec<GridAction> を Vec<Operation> に変換する.
pub(crate) fn actions_to_operations(actions: Vec<GridAction>) -> Vec<Operation> {
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
