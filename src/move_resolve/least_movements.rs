use std::ops::{self, Deref};

use super::dijkstra::DijkstraCost;
use crate::grid::{Pos, VecOnGrid};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LeastMovements(u32);

impl LeastMovements {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn swap_on(
        self,
        field: impl Deref<Target = VecOnGrid<Pos>> + std::fmt::Debug,
        from: Pos,
        to: Pos,
    ) -> Self {
        let before_min_vec = field.grid.looping_min_vec(from, field[from]);
        let before = least_movements(before_min_vec);
        let after_min_vec = field.grid.looping_min_vec(to, field[from]);
        let after = least_movements(after_min_vec);
        let res = 5 + self.0 as i32 + after as i32 - before as i32;
        if res < 0 {
            eprintln!("{:?} -> {:?}", before_min_vec, after_min_vec);
            eprintln!("5 + {} + {} - {} = {}", self.0, after, before, res);
            panic!("invalid swap on: {:?} -> {:?}\n{:#?}", from, to, field);
        }
        Self(res as u32)
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
