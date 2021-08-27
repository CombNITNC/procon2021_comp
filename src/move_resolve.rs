use std::time::Instant;

use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, State},
};
use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, VecOnGrid},
};

pub mod edges_nodes;
pub mod ida_star;
#[cfg(test)]
mod tests;

/// フィールドにあるマスのゴール位置までの距離の合計.
#[derive(Clone, Copy)]
struct DifferentCells(u64);

impl std::fmt::Debug for DifferentCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DifferentCells {
    /// a の位置と b の位置のマスを入れ替えた場合を計算する.
    fn on_swap(self, field: &VecOnGrid<Pos>, a: Pos, b: Pos) -> Self {
        let before = (field[a].manhattan_distance(a) + field[b].manhattan_distance(b)) as i64;
        let after = (field[a].manhattan_distance(b) + field[b].manhattan_distance(a)) as i64;
        let diff = self.0 as i64 - before + after;
        Self(diff as _)
    }
}

#[derive(Debug, Clone)]
enum StatePhase {
    _1 {
        x_amount: i8,
        x_schedule: Vec<u8>,
        y_amount: i8,
        y_schedule: Vec<u8>,
        remaining_move: Option<(Movement, u8)>,
    },
    _2 {
        different_cells: DifferentCells,
    },
}

#[derive(Clone)]
struct GridState<'grid> {
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    phase: StatePhase,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .field("phase", &self.phase)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

impl PartialEq for GridState<'_> {
    fn eq(&self, other: &Self) -> bool {
        (&self.field)
            .into_iter()
            .zip(&other.field)
            .all(|(a, b)| a == b)
            && self.selecting == other.selecting
    }
}

impl<'grid> State<u64> for GridState<'grid> {
    type NextStates = Vec<GridState<'grid>>;
    fn next_states(&self, history: &[Self]) -> Vec<GridState<'grid>> {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self
            .field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if history.len() <= 1 {
            return different_cells
                .map(|next_select| self.with_next_select(next_select))
                .collect();
        }
        if let StatePhase::_1 {
            remaining_move: Some((dir, _)),
            ..
        } = &self.phase
        {
            let mut field = self.field.clone();
            let sel = self.selecting.unwrap();
            match dir {
                Movement::Up => field.swap(sel, self.field.grid.up_of(sel)),
                Movement::Right => field.swap(sel, self.field.grid.right_of(sel)),
                Movement::Down => field.swap(sel, self.field.grid.down_of(sel)),
                Movement::Left => field.swap(sel, self.field.grid.left_of(sel)),
            }
            return vec![Self {
                field,
                ..self.clone()
            }];
        }
        let selecting = self.selecting.unwrap();
        let prev_prev = &history[history.len() - 2];
        let around = self.field.grid.around_of(selecting);
        let swapping_states = around
            .iter()
            .cloned()
            .filter(|&around| {
                prev_prev
                    .selecting
                    .map_or(true, |selected| around != selected)
            })
            .map(|next_swap| self.with_next_swap(next_swap));
        if self.is_moved_from(prev_prev) && 1 <= self.remaining_select {
            let selecting_states = different_cells
                .filter(|&p| p != selecting)
                .map(|next_select| self.with_next_select(next_select));
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        match &self.phase {
            StatePhase::_1 {
                x_schedule,
                y_schedule,
                ..
            } => x_schedule.is_empty() && y_schedule.is_empty(),
            StatePhase::_2 { different_cells } => different_cells.0 == 0,
        }
    }

    fn heuristic(&self) -> u64 {
        match &self.phase {
            StatePhase::_1 {
                x_schedule,
                y_schedule,
                ..
            } => (x_schedule.len() + y_schedule.len()) as _,
            StatePhase::_2 { different_cells } => different_cells.0,
        }
    }

    fn cost_between(&self, next: &Self) -> u64 {
        if self.selecting.is_none() {
            return self.swap_cost as u64;
        }
        (if next.is_moved_from(self) {
            self.swap_cost
        } else {
            self.select_cost
        }) as u64
    }
}

impl GridState<'_> {
    #[inline]
    fn with_next_select(&self, next_select: Pos) -> Self {
        Self {
            selecting: Some(next_select),
            remaining_select: self.remaining_select - 1,
            ..self.clone()
        }
    }

    #[inline]
    fn with_next_swap(&self, next_swap: Pos) -> Self {
        let selecting = self.selecting.unwrap();
        let mut new_field = self.field.clone();
        new_field.swap(selecting, next_swap);
        Self {
            selecting: Some(next_swap),
            field: new_field,
            phase: match &self.phase {
                StatePhase::_1 {
                    x_amount,
                    x_schedule,
                    y_amount,
                    y_schedule,
                    remaining_move,
                } => {
                    let mut x_schedule = x_schedule.clone();
                    let mut y_schedule = y_schedule.clone();
                    if let Some((mov, remaining)) = *remaining_move {
                        StatePhase::_1 {
                            remaining_move: (1 <= remaining).then(|| (mov, remaining - 1)),
                            x_amount: *x_amount,
                            x_schedule,
                            y_amount: *y_amount,
                            y_schedule,
                        }
                    } else if let Some(x) = y_schedule.pop() {
                        let mov = Movement::between_pos(selecting, next_swap);
                        StatePhase::_1 {
                            remaining_move: Some((mov, *x_amount as u8)),
                            x_amount: *x_amount,
                            x_schedule,
                            y_amount: *y_amount,
                            y_schedule,
                        }
                    } else if let Some(y) = x_schedule.pop() {
                        let mov = Movement::between_pos(selecting, next_swap);
                        StatePhase::_1 {
                            remaining_move: Some((mov, *y_amount as u8)),
                            x_amount: *x_amount,
                            x_schedule,
                            y_amount: *y_amount,
                            y_schedule,
                        }
                    } else {
                        self.phase.clone()
                    }
                }
                StatePhase::_2 { different_cells } => StatePhase::_2 {
                    different_cells: different_cells.on_swap(&self.field, selecting, next_swap),
                },
            },
            ..self.clone()
        }
    }

    #[inline]
    fn is_moved_from(&self, prev: &Self) -> bool {
        prev.selecting.map_or(true, |prev_selecting| {
            prev.field[prev_selecting] == self.field[self.selecting.unwrap()]
        })
    }
}

/// 状態の履歴 Vec<GridState> を Vec<Operation> に変換する.
fn path_to_operations(path: Vec<GridState>) -> Vec<Operation> {
    if path.is_empty() {
        return vec![];
    }
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    let mut prev = &path[0];
    for state in &path[1..] {
        let is_swapped = (&prev.field)
            .into_iter()
            .zip(&state.field)
            .any(|(a, b)| a != b);
        if is_swapped {
            let movement = Movement::between_pos(prev.selecting.unwrap(), state.selecting.unwrap());
            current_operation.as_mut().unwrap().movements.push(movement);
        } else if let Some(op) = current_operation.replace(Operation {
            select: state.selecting.unwrap(),
            movements: vec![],
        }) {
            operations.push(op);
        }
        prev = state;
    }
    if let Some(op) = current_operation {
        operations.push(op);
    }
    operations
}

fn min_shift(field: &mut VecOnGrid<Pos>) -> (isize, isize) {
    let mut min = (300, (0, 0));
    for y_shift in 0..field.grid.height() as isize {
        for x_shift in 0..field.grid.width() as isize {
            let count = field
                .iter_with_pos()
                .filter(|&(pos, &cell)| pos != cell)
                .count();
            if count < min.0 {
                min = (count, (x_shift, y_shift));
            }
            field.rotate_x(1);
        }
        field.rotate_y(1);
    }
    let (_, (mut x, mut y)) = min;
    if field.grid.width() as isize / 2 <= x {
        x -= field.grid.width() as isize;
    }
    if field.grid.height() as isize / 2 <= y {
        y -= field.grid.height() as isize;
    }
    (x, y)
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { mut nodes, .. } = EdgesNodes::new(grid, movements);
    let (x, y) = min_shift(&mut nodes);
    let rotated = {
        let mut ns = nodes.clone();
        ns.rotate_x(x);
        ns.rotate_y(y);
        ns
    };
    let hint = rotated
        .iter_with_pos()
        .filter(|&(p, &r)| p != r)
        .map(|(p, _)| p)
        .next();
    let mut x_schedule: Vec<_> = (0..grid.height()).collect();
    let mut y_schedule: Vec<_> = (0..grid.width()).collect();
    if let Some(hint) = hint {
        // 最後に hint の位置を入れ替えると効率的
        let move_on_last = x_schedule.remove(hint.y() as usize);
        x_schedule.insert(0, move_on_last);
        let move_on_last = y_schedule.remove(hint.x() as usize);
        y_schedule.insert(0, move_on_last);
    }
    let lower_bound = {
        let distances: Vec<_> = rotated
            .iter_with_pos()
            .map(|(p, &n)| {
                (p.manhattan_distance(n) as u64).min(
                    (p.x() as i64 + grid.width() as i64 - n.x() as i64).unsigned_abs()
                        + (p.y() as i64 + grid.height() as i64 - n.y() as i64).unsigned_abs(),
                )
            })
            .collect();
        distances.iter().sum()
    };
    let different_cells = DifferentCells(lower_bound);
    let mut min = (vec![], 1 << 60);
    const SEARCH_TIMEOUT: u64 = 10 * 60;
    let start_instant = Instant::now();
    for (total_path, total_cost) in ida_star(
        GridState {
            field: nodes.clone(),
            selecting: None,
            phase: StatePhase::_1 {
                x_amount: x as i8,
                x_schedule,
                y_amount: y as i8,
                y_schedule,
                remaining_move: None,
            },
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        lower_bound,
    )
    .map(|(mut phase1_path, phase1_cost)| {
        let selected1 = phase1_path
            .windows(2)
            .filter(|win| {
                win[0].selecting.map_or(true, |selecting_0| {
                    win[0].field[selecting_0] != win[1].field[win[1].selecting.unwrap()]
                })
            })
            .count();
        let phase1_last = phase1_path.pop().unwrap();
        ida_star(
            GridState {
                field: phase1_last.field.clone(),
                selecting: phase1_last.selecting,
                phase: StatePhase::_2 { different_cells },
                swap_cost,
                select_cost,
                remaining_select: select_limit - selected1 as u8,
            },
            lower_bound,
        )
        .map(move |(mut phase2_path, phase2_cost)| {
            let mut path = phase1_path.clone();
            path.append(&mut phase2_path);
            (path, phase1_cost + phase2_cost)
        })
        .next()
        .unwrap()
    }) {
        if total_cost < min.1 {
            min = (total_path, total_cost);
        } else {
            break;
        }
        if SEARCH_TIMEOUT <= Instant::now().duration_since(start_instant).as_secs() {
            break;
        }
    }
    path_to_operations(min.0)
}
