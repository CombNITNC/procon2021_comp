use std::hash::Hash;

use crate::{grid::board::Board, move_resolve::beam_search::BeamSearchState};

use super::GridAction;

#[derive(Debug, Clone)]
pub struct CostReducer {
    board: Board,
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
    }
}

impl BeamSearchState for CostReducer {
    type A = GridAction;
    fn apply(&self, _action: Self::A) -> Self {
        todo!()
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        todo!()
    }

    fn is_goal(&self) -> bool {
        todo!()
    }

    type C = u64;
    fn cost_on(&self, _action: Self::A) -> Self::C {
        todo!()
    }

    fn enrich(_states: &mut [Self]) {
        todo!()
    }
}
