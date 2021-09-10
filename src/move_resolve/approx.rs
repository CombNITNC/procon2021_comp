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
    row_estimate: Option<RowSolveEstimate>,
}

impl Solver {
    pub(super) fn solve(&mut self, select: Pos, field: &VecOnGrid<Pos>) -> Vec<GridAction> {
        let mut board = Board::new(select, field.clone());
        let mut actions = vec![];
        loop {
            if board.grid().height() < board.grid().width() {
                board = board.rotate_to(3);
            }
            if board.grid().width() <= 3 && board.grid().height() <= 3 {
                break;
            }
            let target_row = self.next_row(&board);
            if board.selected().y() == target_row {
                board = board.rotate_to(3);
                continue;
            }
            let mut moves = self.solve_row(&board, target_row);
            for &mov in &moves {
                match mov {
                    GridAction::Swap(mov) => {
                        let to_swap = board.grid().move_pos_to(board.selected(), mov);
                        board.swap_to(to_swap);
                    }
                    GridAction::Select(sel) => {
                        board.select(sel);
                    }
                }
            }
            actions.append(&mut moves);
            for x in 0..board.grid().width() {
                let pos = board.grid().pos(x, target_row);
                board.lock(pos);
            }
        }
        actions
    }

    fn next_row(&self, board: &Board) -> u8 {
        for y in 0..board.grid().height() {
            if (0..board.grid().width()).any(|x| {
                let pos = board.grid().pos(x, y);
                !board.is_locked(pos)
            }) {
                return y;
            }
        }
        0
    }

    fn solve_row(&mut self, board: &Board, target_row: u8) -> Vec<GridAction> {
        let estimate = estimate_solve_row(board.clone(), target_row);
        if let Some(worst_estimate) = &self.row_estimate {
            if worst_estimate.worst_route_size < estimate.worst_route_size {
                self.row_estimate.replace(estimate);
            }
        } else {
            self.row_estimate.replace(estimate);
        }
        let estimate = self.row_estimate.as_ref().unwrap();

        let mut actions = vec![];
        if board
            .grid()
            .looping_manhattan_dist(estimate.moves[0], estimate.moves[1])
            != 1
        {
            actions.push(GridAction::Select(estimate.moves[0]));
        }
        for win in estimate.moves.windows(2) {
            if board.grid().looping_manhattan_dist(win[0], win[1]) == 1 {
                actions.push(GridAction::Swap(Movement::between_pos(win[0], win[1])));
            } else {
                actions.push(GridAction::Select(win[1]));
            }
        }
        actions
    }
}
