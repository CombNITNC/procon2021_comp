use std::{hash::Hash, sync::Arc};

use fxhash::FxHashMap as HashMap;

use crate::{
    basis::Movement,
    grid::{
        board::{Board, BoardFinder},
        Pos,
    },
    move_resolve::{beam_search::BeamSearchState, ida_star::IdaSearchState, ResolveParam},
};

use super::{GridAction, SqManhattan};

#[derive(Clone, Eq)]
pub struct Completer {
    board: Board,
    prev_action: Option<GridAction>,
    dist: SqManhattan,
    pre_calc: Arc<HashMap<(Pos, Pos), SqManhattan>>,
    param: ResolveParam,
}

impl Completer {
    pub fn new(board: Board, param: ResolveParam, prev_action: Option<GridAction>) -> Self {
        let pre_calc: HashMap<_, _> = SqManhattan::pre_calc(board.grid()).collect();
        let dist = board
            .field()
            .iter_with_pos()
            .map(|(pos, &cell)| pre_calc[&(pos, cell)])
            .sum();
        Self {
            board,
            prev_action,
            dist,
            pre_calc: Arc::new(pre_calc),
            param,
        }
    }

    fn cost_on(&self, action: GridAction) -> u64 {
        match action {
            GridAction::Swap(_) => self.param.swap_cost as u64,
            GridAction::Select(_) => self.param.select_cost as u64,
        }
    }

    fn apply(&self, action: GridAction) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selected = self.board.selected().unwrap();
                let finder = BoardFinder::new(self.board.grid());
                let next_swap = finder.move_pos_to(selected, mov);
                let mut new_board = self.board.clone();
                new_board.swap_to(next_swap);
                Self {
                    board: new_board,
                    dist: self.dist.swap_on(
                        (selected, next_swap),
                        &self.board.field(),
                        &self.pre_calc,
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

    fn next_actions(&self) -> Vec<GridAction> {
        // 揃っているマスどうしは入れ替えない
        let field = self.board.field();
        let different_cells = field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if self.prev_action.is_none() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selected = self.board.selected().unwrap();
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
}

impl std::fmt::Debug for Completer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("board", &self.board)
            .field("dist", &self.dist)
            .field("remaining_select", &self.param.select_limit)
            .finish()
    }
}

impl PartialEq for Completer {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.dist == other.dist
            && self.param.select_limit == other.param.select_limit
    }
}

impl Hash for Completer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.dist.hash(state);
        self.param.select_limit.hash(state);
    }
}

impl IdaSearchState for Completer {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        self.apply(action)
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        self.next_actions()
    }

    fn is_goal(&self) -> bool {
        self.dist.0 == 0
    }

    type C = u64;
    fn heuristic(&self) -> Self::C {
        self.dist.0 as u64
    }

    fn cost_on(&self, action: Self::A) -> Self::C {
        self.cost_on(action)
    }
}

impl BeamSearchState for Completer {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        self.apply(action)
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        self.next_actions()
    }

    fn is_goal(&self) -> bool {
        self.dist.0 == 0
    }

    type C = u64;
    fn cost_on(&self, action: Self::A) -> Self::C {
        self.cost_on(action)
    }

    fn enrichment_key(&self) -> usize {
        self.param.select_limit as _
    }
}
