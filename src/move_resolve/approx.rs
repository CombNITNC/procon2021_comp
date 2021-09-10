use self::estimate::{estimate_solve_row, RowSolveEstimate};
use super::GridAction;
use crate::{
    basis::Movement,
    grid::{
        board::{Board, BoardFinder},
        Pos, VecOnGrid,
    },
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
        let mut finder = BoardFinder::new(&board);
        let mut actions = vec![];
        loop {
            if finder.height() < finder.width() {
                finder.rotate_to(3);
            }
            if finder.width() <= 3 && finder.height() <= 3 {
                break;
            }
            let targets = self.next_targets(&board, &finder);
            if targets.contains(&board.selected()) {
                finder.rotate_to(3);
                continue;
            }
            eprintln!("targets: {:?}", targets);

            if !targets.is_empty() {
                let mut moves = self.solve_row(&board, &finder, &targets);
                for &mov in &moves {
                    match mov {
                        GridAction::Swap(mov) => {
                            let to_swap = board.move_pos_to(board.selected(), mov);
                            board.swap_to(to_swap);
                        }
                        GridAction::Select(sel) => {
                            board.select(sel);
                        }
                    }
                }
                actions.append(&mut moves);
            }
            for pos in targets {
                board.lock(pos);
            }
            finder.slice_up(&board);
        }
        actions
    }

    fn next_targets(&self, board: &Board, finder: &BoardFinder) -> Vec<Pos> {
        let mut targets: Vec<_> = finder.iter().filter(|&pos| !board.is_locked(pos)).collect();
        targets.sort_unstable();
        targets
    }

    fn solve_row(
        &mut self,
        board: &Board,
        finder: &BoardFinder,
        targets: &[Pos],
    ) -> Vec<GridAction> {
        let estimate = estimate_solve_row(board.clone(), finder, targets);
        if let Some(worst_estimate) = &self.row_estimate {
            if worst_estimate.worst_route_size < estimate.worst_route_size {
                self.row_estimate.replace(estimate);
            }
        } else {
            self.row_estimate.replace(estimate);
        }
        let estimate = self.row_estimate.as_ref().unwrap();

        let mut actions = vec![];
        if board.looping_manhattan_dist(estimate.moves[0], estimate.moves[1]) != 1 {
            actions.push(GridAction::Select(estimate.moves[0]));
        }
        for win in estimate.moves.windows(2) {
            if board.looping_manhattan_dist(win[0], win[1]) == 1 {
                actions.push(GridAction::Swap(Movement::between_pos(win[0], win[1])));
            } else {
                actions.push(GridAction::Select(win[1]));
            }
        }
        actions
    }
}
