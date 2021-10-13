use std::hash::Hash;

use crate::{
    basis::Movement,
    grid::board::{Board, BoardFinder},
    move_resolve::{ida_star::IdaSearchState, DifferentCells, ResolveParam},
};

use super::GridAction;

#[derive(Clone, Eq)]
pub(crate) struct Completer {
    board: Board,
    prev_action: Option<GridAction>,
    different_cells: DifferentCells,
    param: ResolveParam,
}

impl Completer {
    pub(crate) fn new(board: Board, param: ResolveParam, prev_action: Option<GridAction>) -> Self {
        let different_cells = DifferentCells::new(&board.field());
        Self {
            board,
            prev_action,
            different_cells,
            param,
        }
    }
}

impl std::fmt::Debug for Completer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("board", &self.board)
            .field("different_cells", &self.different_cells)
            .field("remaining_select", &self.param.select_limit)
            .finish()
    }
}

impl PartialEq for Completer {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.different_cells == other.different_cells
            && self.param.select_limit == other.param.select_limit
    }
}

impl Hash for Completer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.different_cells.hash(state);
        self.param.select_limit.hash(state);
    }
}

impl IdaSearchState for Completer {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selected = self.board.selected();
                let finder = BoardFinder::new(self.board.grid());
                let next_swap = finder.move_pos_to(selected, mov);
                let mut new_board = self.board.clone();
                new_board.swap_to(next_swap);
                Self {
                    board: new_board,
                    different_cells: self.different_cells.on_swap(
                        self.board.field(),
                        selected,
                        next_swap,
                    ),
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => {
                let mut new_board = self.board.clone();
                new_board.select(sel);
                let mut param = self.param;
                param.select_limit -= 1;
                Self {
                    board: new_board,
                    param,
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
        }
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        // 揃っているマスどうしは入れ替えない
        let field = self.board.field();
        let different_cells = field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if self.prev_action.is_none() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selected = self.board.selected();
        let prev = self.prev_action.unwrap();
        let swapping_states = self
            .board
            .around_of(selected)
            .map(|to| Movement::between_pos(selected, to))
            .filter(|&around| {
                if let GridAction::Swap(dir) = prev {
                    around != dir.opposite()
                } else {
                    true
                }
            })
            .map(GridAction::Swap);
        if matches!(prev, GridAction::Swap(_)) && 1 <= self.param.select_limit {
            let selecting_states = different_cells
                .filter(|&p| p != selected)
                .map(GridAction::Select);
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        self.different_cells.0 == 0
    }

    type C = u64;
    fn heuristic(&self) -> Self::C {
        self.board
            .field()
            .iter_with_pos()
            .map(|(p, &e)| self.board.grid().looping_manhattan_dist(p, e).pow(2) as u64)
            .sum()
    }

    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.param.swap_cost as u64,
            GridAction::Select(_) => self.param.select_cost as u64,
        }
    }
}
