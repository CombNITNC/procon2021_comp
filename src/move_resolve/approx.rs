use self::estimate::{estimate_solve_row, RowSolveEstimate};
use super::GridAction;
use crate::{
    basis::Movement,
    grid::{board::Board, Pos, VecOnGrid},
};

mod estimate;
mod route;

#[derive(Debug, Default)]
pub(crate) struct Solver {
    estimate: Option<RowSolveEstimate>,
}

impl Solver {
    pub(super) fn solve_row(
        &mut self,
        select: Pos,
        field: &VecOnGrid<Pos>,
        target_row: u8,
    ) -> Vec<GridAction> {
        let estimate = estimate_solve_row(Board::new(select, field.clone()), target_row);
        if let Some(worst_estimate) = &self.estimate {
            if worst_estimate.worst_route_size < estimate.worst_route_size {
                self.estimate.replace(estimate);
            }
        } else {
            self.estimate.replace(estimate);
        }
        let estimate = self.estimate.as_ref().unwrap();

        let mut actions = vec![];
        if field
            .grid
            .looping_manhattan_dist(estimate.moves[0], estimate.moves[1])
            != 1
        {
            actions.push(GridAction::Select(estimate.moves[0]));
        }
        for win in estimate.moves.windows(2) {
            if field.grid.looping_manhattan_dist(win[0], win[1]) == 1 {
                actions.push(GridAction::Swap(Movement::between_pos(win[0], win[1])));
            } else {
                actions.push(GridAction::Select(win[1]));
            }
        }
        actions
    }
}
