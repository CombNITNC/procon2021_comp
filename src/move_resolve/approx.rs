use self::estimate::estimate_solve_row;
use super::GridAction;
use crate::{
    basis::Movement,
    grid::{
        board::{Board, BoardFinder},
        Pos,
    },
};

mod estimate;
pub(crate) mod gen;
mod route;

pub(crate) trait NextTargetsGenerator {
    fn next_targets(&mut self, finder: &BoardFinder) -> Vec<Pos>;
}

#[derive(Debug, Default)]
pub(crate) struct Solver<G> {
    pub(crate) threshold_x: u8,
    pub(crate) threshold_y: u8,
    pub(crate) targets_gen: G,
}

impl<G: NextTargetsGenerator> Solver<G> {
    pub(super) fn solve(&mut self, mut board: Board) -> Option<Vec<GridAction>> {
        let mut finder = BoardFinder::new(board.grid());
        let mut actions = vec![];
        loop {
            if finder.height() < finder.width() {
                finder.rotate_to(3);
            }
            if finder.width() <= self.threshold_x && finder.height() <= self.threshold_y {
                break;
            }
            let targets: Vec<_> = self
                .targets_gen
                .next_targets(&finder)
                .into_iter()
                .filter(|&p| !board.is_locked(p))
                .collect();
            if targets.is_empty() || targets.contains(&board.forward(board.selected().unwrap())) {
                finder.rotate_to(3);
                continue;
            }

            let estimate = estimate_solve_row(board.clone(), &finder, &targets)?;
            for &pos in &estimate.moves {
                board.swap_to(pos);
            }
            for win in estimate.moves.windows(2) {
                let mov = Movement::between_pos(win[0], win[1]);
                actions.push(GridAction::Swap(mov));
            }
            for pos in targets {
                debug_assert_eq!(pos, board.forward(pos), "{:#?}", board);
                board.lock(pos);
            }
            finder.slice_up();
        }
        Some(actions)
    }
}
