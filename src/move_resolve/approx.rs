use std::{collections::HashSet, ops};

use crate::grid::{Grid, Pos, VecOnGrid};

mod estimate;
mod route;

#[derive(Debug, Clone)]
struct Board<'grid> {
    select: Pos,
    field: VecOnGrid<'grid, Pos>,
    locked: HashSet<Pos>,
}

impl Board<'_> {
    fn grid(&self) -> &Grid {
        self.field.grid
    }

    fn swap_to(&mut self, to_swap: Pos) {
        if self.locked.contains(&to_swap) || self.locked.contains(&self.select) {
            return;
        }
        self.field.swap(self.select, to_swap);
        self.select = to_swap;
    }

    fn around_of(&self, pos: Pos) -> Vec<Pos> {
        self.grid()
            .around_of(pos)
            .iter()
            .copied()
            .filter(|pos| !self.locked.contains(&pos))
            .collect()
    }

    fn lock(&mut self, pos: Pos) -> bool {
        self.locked.insert(pos)
    }

    fn unlock(&mut self, pos: Pos) -> bool {
        self.locked.remove(&pos)
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
        let before = least_movements(field.grid.looping_min_vec(from, field[from]));
        let after = least_movements(field.grid.looping_min_vec(to, field[from]));
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
