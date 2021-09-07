use std::ops;

use self::estimate::{estimate_solve_row, RowSolveEstimate};
use super::{dijkstra::DijkstraCost, GridAction};
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

fn least_movements((dx, dy): (i32, i32)) -> u32 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let dx = dx.abs();
    let dy = dy.abs();
    let d = (dx - dy).unsigned_abs();
    let min = dx.min(dy) as u32;
    let mut ret = 5 * d + 6 * min - 4;
    if dx == dy {
        ret += 2;
    }
    ret
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LeastMovements(u32);

impl LeastMovements {
    fn new(field: &VecOnGrid<Pos>) -> Self {
        Self(
            field
                .iter_with_pos()
                .map(|(pos, &to)| field.grid.looping_min_vec(pos, to))
                .map(least_movements)
                .sum(),
        )
    }

    fn swap_on(self, field: &VecOnGrid<Pos>, from: Pos, to: Pos) -> Self {
        let before = least_movements(field.grid.looping_min_vec(from, field[from]))
            + least_movements(field.grid.looping_min_vec(to, field[to]));
        let after = least_movements(field.grid.looping_min_vec(to, field[from]))
            + least_movements(field.grid.looping_min_vec(from, field[to]));
        Self(4 + self.0 + after - before)
    }
}

impl ops::Add for LeastMovements {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign for LeastMovements {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl DijkstraCost for LeastMovements {
    const IDENTITY: Self = Self(1_000_000_000);

    fn op(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
