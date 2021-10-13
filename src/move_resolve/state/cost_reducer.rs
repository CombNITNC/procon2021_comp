use std::{collections::HashMap, hash::Hash, iter::Sum, ops::Add, sync::Arc};

use crate::{
    grid::{board::Board, Grid, Pos},
    move_resolve::{beam_search::BeamSearchState, ResolveParam},
};

use super::GridAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SqManhattan(u32);

impl SqManhattan {
    fn pre_calc(grid: Grid) -> HashMap<(Pos, Pos), Self> {
        let mut map = HashMap::new();
        for from in grid.all_pos() {
            for to in grid.all_pos() {
                let dist = grid.looping_manhattan_dist(from, to);
                map.insert((from, to), Self(dist * dist));
            }
        }
        map
    }
}

impl Add<SqManhattan> for SqManhattan {
    type Output = Self;

    fn add(self, rhs: SqManhattan) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sum for SqManhattan {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0), |acc, x| acc + x)
    }
}

#[derive(Debug)]
pub(crate) struct CostReducer {
    board: Board,
    initial_dist: SqManhattan,
    dist: SqManhattan,
    pre_calc: Arc<HashMap<(Pos, Pos), SqManhattan>>,
    param: ResolveParam,
}

impl CostReducer {
    pub(crate) fn new(board: Board, param: ResolveParam) -> Self {
        let pre_calc = Arc::new(SqManhattan::pre_calc(board.grid()));
        let dist = board
            .field()
            .iter_with_pos()
            .map(|(pos, &cell)| pre_calc[&(pos, cell)])
            .sum();
        Self {
            board,
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
        // dist <= 0.8 * initial_dist
        // => dist <= 8 / 10 * initial_dist
        // => dist * 10 / 8 <= initial_dist
        // => dist * 5 / 4 <= initial_dist
        self.dist.0 * 5 / 4 <= self.initial_dist.0
    }

    type C = u64;
    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.param.swap_cost as u64,
            GridAction::Select(_) => self.param.select_cost as u64,
        }
    }

    fn enrichment_key(&self) -> usize {
        self.param.select_limit as usize
    }
}
