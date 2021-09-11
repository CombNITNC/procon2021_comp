use self::estimate::estimate_solve_row;
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
pub(crate) struct Solver {}

impl Solver {
    pub(super) fn solve(&mut self, select: Pos, field: &VecOnGrid<Pos>) -> Vec<GridAction> {
        let mut board = Board::new(select, field.clone());
        let mut finder = BoardFinder::new(field.grid);
        let mut actions = vec![GridAction::Select(select)];
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
                let estimate = estimate_solve_row(board.clone(), &finder, &targets);
                for &pos in &estimate.moves {
                    board.swap_to(pos);
                }
                for win in estimate.moves.windows(2) {
                    let mov = Movement::between_pos(win[0], win[1]);
                    actions.push(GridAction::Swap(mov));
                }
            }
            for pos in targets {
                debug_assert_eq!(pos, board.forward(pos), "{:#?}", board);
                board.lock(pos);
            }
            finder.slice_up();
        }
        actions
    }

    fn next_targets(&self, board: &Board, finder: &BoardFinder) -> Vec<Pos> {
        finder.iter().filter(|&pos| !board.is_locked(pos)).collect()
    }
}
