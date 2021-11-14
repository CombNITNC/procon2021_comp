use std::{hash::Hash, sync::Arc};

use fxhash::FxHashMap as HashMap;

use crate::{
    basis::Movement,
    grid::{
        board::{Board, BoardFinder},
        Pos,
    },
    move_resolve::{beam_search::BeamSearchState, ResolveParam},
};

use super::{GridAction, SqManhattan};

#[derive(Debug)]
pub struct CostReducer {
    board: Board,
    prev_action: Option<GridAction>,
    initial_dist: SqManhattan,
    dist: SqManhattan,
    pre_calc: Arc<HashMap<(Pos, Pos), SqManhattan>>,
    param: ResolveParam,
}

impl CostReducer {
    pub fn new(board: Board, param: ResolveParam) -> Self {
        let pre_calc: Arc<HashMap<_, _>> = Arc::new(SqManhattan::pre_calc(board.grid()).collect());
        let dist = board
            .field()
            .iter_with_pos()
            .map(|(pos, &cell)| pre_calc[&(pos, cell)])
            .sum();
        Self {
            board,
            prev_action: None,
            initial_dist: dist,
            dist,
            pre_calc,
            param,
        }
    }
}

impl Clone for CostReducer {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            pre_calc: Arc::clone(&self.pre_calc),
            ..*self
        }
    }
}

impl PartialEq for CostReducer {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
    }
}

impl Eq for CostReducer {}

impl Hash for CostReducer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.param.select_limit.hash(state);
    }
}

impl BeamSearchState for CostReducer {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        let mut cloned = self.clone();
        cloned.prev_action.replace(action);

        match action {
            GridAction::Swap(mov) => {
                let selected = self.board.selected().unwrap();
                let finder = BoardFinder::new(self.board.grid());
                let next_swap = finder.move_pos_to(selected, mov);

                cloned.board.swap_to(next_swap);
                cloned.dist =
                    self.dist
                        .swap_on((selected, next_swap), &self.board.field(), &self.pre_calc);
            }
            GridAction::Select(sel) => {
                cloned.board.select(sel);
                cloned.param.select_limit -= 1;
            }
        }
        cloned
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

    fn is_goal(&self) -> bool {
        // dist <= 0.90 * initial_dist
        // => dist <= 90 / 100 * initial_dist
        // => dist * 100 / 90 <= initial_dist
        // => dist * 10 / 9 <= initial_dist
        self.dist.as_u32() * 10 / 9 <= self.initial_dist.as_u32()
    }

    type C = u64;
    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.param.swap_cost as u64,
            GridAction::Select(_) => self.param.select_cost as u64,
        }
    }

    fn max_cost(&self) -> Self::C {
        let cost_limit = self.param.select_cost as u64 + self.param.swap_cost as u64 * 3;
        cost_limit.min(self.initial_dist.as_u32() as u64 / 10)
    }

    fn enrichment_key(&self) -> usize {
        self.param.select_limit as usize
    }
}
