use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, IdaStarState},
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
    fn new(nodes: &VecOnGrid<Pos>) -> Self {
        let mut distances: Vec<_> = nodes
            .iter_with_pos()
            .map(|(p, &n)| nodes.grid.looping_manhattan_dist(p, n) as u64)
            .collect();
        distances.sort_unstable();
        Self(distances.iter().sum())
    }

    /// a の位置と b の位置のマスを入れ替えた場合を計算する.
    fn on_swap(self, field: &VecOnGrid<Pos>, a: Pos, b: Pos) -> Self {
        let before = (field.grid.looping_manhattan_dist(field[a], a)
            + field.grid.looping_manhattan_dist(field[b], b)) as i64;
        let after = (field.grid.looping_manhattan_dist(field[a], b)
            + field.grid.looping_manhattan_dist(field[b], a)) as i64;
        let diff = self.0 as i64 - before + after;
        Self(diff as _)
    }
}

#[derive(Clone)]
struct GridCompleter<'grid> {
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    prev_action: Option<GridAction>,
    different_cells: DifferentCells,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for GridCompleter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .field("different_cells", &self.different_cells)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GridAction {
    Swap(Movement),
    Select(Pos),
}

impl<'grid> IdaStarState for GridCompleter<'grid> {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selecting = self.selecting.unwrap();
                let next_swap = self.field.grid.move_pos_to(selecting, mov);
                let mut new_field = self.field.clone();
                new_field.swap(selecting, next_swap);
                Self {
                    selecting: Some(next_swap),
                    field: new_field,
                    different_cells: self.different_cells.on_swap(
                        &self.field,
                        selecting,
                        next_swap,
                    ),
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => Self {
                selecting: Some(sel),
                remaining_select: self.remaining_select - 1,
                prev_action: Some(action),
                ..self.clone()
            },
        }
    }

    type AS = Vec<GridAction>;
    fn next_actions(&self) -> Self::AS {
        // 揃っているマスどうしは入れ替えない
        let different_cells = self
            .field
            .iter_with_pos()
            .filter(|&(pos, &cell)| pos != cell)
            .map(|(_, &cell)| cell);
        if self.selecting.is_none() {
            return different_cells.map(GridAction::Select).collect();
        }
        let selecting = self.selecting.unwrap();
        let prev = self.prev_action.unwrap();
        let swapping_states = [
            Movement::Up,
            Movement::Right,
            Movement::Down,
            Movement::Left,
        ]
        .iter()
        .copied()
        .filter(|&around| {
            if let GridAction::Swap(dir) = prev {
                around != dir.opposite()
            } else {
                true
            }
        })
        .map(GridAction::Swap);
        if matches!(prev, GridAction::Swap(_)) && 1 <= self.remaining_select {
            let selecting_states = different_cells
                .filter(|&p| p != selecting)
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
        (self.different_cells.0 as f64).powf(1.0 + 41.0 / 256.0) as u64
    }

    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.swap_cost as u64,
            GridAction::Select(_) => self.select_cost as u64,
        }
    }
}

/// 操作の履歴 Vec<GridAction> を Vec<Operation> に変換する.
fn actions_to_operations(actions: Vec<GridAction>) -> Vec<Operation> {
    if actions.is_empty() {
        return vec![];
    }
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    for state in actions {
        match state {
            GridAction::Swap(mov) => {
                current_operation.as_mut().unwrap().movements.push(mov);
            }
            GridAction::Select(select) => {
                if let Some(op) = current_operation.replace(Operation {
                    select,
                    movements: vec![],
                }) {
                    operations.push(op);
                }
            }
        }
    }
    if let Some(op) = current_operation {
        operations.push(op);
    }
    operations
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の最適解を求める.
///
/// ```
/// // 10 00
/// let grid = Grid::new(2, 1);
/// let mut field = VecOnGrid::with_init(&grid, grid.pos(0, 0));
/// field[grid.pos(0, 0)] = grid.pos(1, 0);
/// field[grid.pos(1, 0)] = grid.pos(0, 0);
/// let path = resolve(
///     &grid,
///     &[
///         (grid.pos(0, 0), grid.pos(1, 0)),
///         (grid.pos(1, 0), grid.pos(0, 0)),
///     ],
///     1,
///     1,
///     1,
/// );
/// assert_eq!(path.len(), 1);
/// assert_eq!(
///     Operation {
///         select: grid.pos(1, 0),
///         movements: vec![Right],
///     },
///     path[0]
/// );
/// ```
pub(crate) fn resolve(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    if 36 <= grid.width() * grid.height() {
        return resolve_approximately(grid, movements, select_limit, swap_cost, select_cost);
    }
    let EdgesNodes { nodes, .. } = EdgesNodes::new(grid, movements);
    let different_cells = DifferentCells::new(&nodes);
    let lower_bound = different_cells.0;
    let (path, _total_cost) = ida_star(
        GridCompleter {
            field: nodes.clone(),
            selecting: None,
            prev_action: None,
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        lower_bound,
    );
    actions_to_operations(path)
}

#[derive(Clone)]
struct RowCompleter<'grid> {
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    prev_action: Option<GridAction>,
    target_row: u8,
    swap_cost: u16,
    select_cost: u16,
    remaining_select: u8,
}

impl std::fmt::Debug for RowCompleter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .field("target_row", &self.target_row)
            .field("remaining_select", &self.remaining_select)
            .finish()
    }
}

impl IdaStarState for RowCompleter<'_> {
    type A = GridAction;
    fn apply(&self, action: Self::A) -> Self {
        match action {
            GridAction::Swap(mov) => {
                let selecting = self.selecting.unwrap();
                let next_swap = self.field.grid.move_pos_to(selecting, mov);
                let mut new_field = self.field.clone();
                new_field.swap(selecting, next_swap);
                Self {
                    selecting: Some(next_swap),
                    field: new_field,
                    prev_action: Some(action),
                    ..self.clone()
                }
            }
            GridAction::Select(sel) => Self {
                selecting: Some(sel),
                remaining_select: self.remaining_select - 1,
                prev_action: Some(action),
                ..self.clone()
            },
        }
    }

    type AS = Vec<Self::A>;
    fn next_actions(&self) -> Self::AS {
        // Y 座標が target_row であるマスを揃える
        // Y が target_row でないマスを選択して, Y が target_row であるマスを揃えていく
        let grid = self.field.grid;
        let different = grid
            .all_pos()
            .map(|pos| self.field[pos])
            .filter(|&pos| pos.y() != self.target_row)
            .filter(|&pos| pos != self.field[pos]);
        if self.prev_action.is_none() {
            return different.map(GridAction::Select).collect();
        }
        let selecting = self.selecting.unwrap();
        let prev = self.prev_action.unwrap();
        let swapping_states = [
            Movement::Up,
            Movement::Right,
            Movement::Down,
            Movement::Left,
        ]
        .iter()
        .copied()
        .filter(|&around| {
            if let GridAction::Swap(dir) = prev {
                around != dir.opposite()
            } else {
                true
            }
        })
        .filter(|&mov| {
            let next_to_swap = grid.move_pos_to(selecting, mov);
            if self.field[next_to_swap].y() != self.target_row {
                return true;
            }
            // 入れ替え対象が target_row に属する場合
            // next_to_swap がマンハッタン経路を辿るようにする
            let min_vec = grid.looping_min_vec(next_to_swap, self.field[next_to_swap]);
            let preferred_dir = match min_vec {
                (min_x, min_y) if 0 < min_x && 0 < min_y => [Movement::Right, Movement::Down],
                (min_x, min_y) if 0 < min_x && min_y < 0 => [Movement::Right, Movement::Up],
                (min_x, min_y) if min_x < 0 && 0 < min_y => [Movement::Left, Movement::Down],
                _ => [Movement::Left, Movement::Up],
            };
            let dir = Movement::between_pos(next_to_swap, selecting);
            preferred_dir.iter().any(|&preferred| preferred == dir)
        })
        .map(GridAction::Swap);
        if matches!(prev, GridAction::Swap(_)) && 2 < self.remaining_select {
            let selecting_states = different
                .filter(|&p| p != selecting)
                .map(GridAction::Select);
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }
    fn is_goal(&self) -> bool {
        let grid = self.field.grid;
        (0..grid.width())
            .map(|x| grid.clamping_pos(x, self.target_row))
            .all(|pos| pos == self.field[pos])
    }

    type C = u64;
    fn heuristic(&self) -> Self::C {
        let grid = self.field.grid;
        (0..grid.width())
            .map(|x| grid.clamping_pos(x, self.target_row))
            .filter(|&pos| pos != self.field[pos])
            .count() as u64
    }
    fn cost_on(&self, action: Self::A) -> Self::C {
        match action {
            GridAction::Swap(_) => self.swap_cost as u64,
            GridAction::Select(_) => self.select_cost as u64,
        }
    }
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順の近似解を求める.
fn resolve_approximately(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    mut select_limit: u8,
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { mut nodes, .. } = EdgesNodes::new(grid, movements);
    let mut all_actions = vec![];
    let mut selection = None;

    let mut row_to_sort: Vec<_> = (0..grid.height()).collect();
    for _ in 0..grid.height() - 1 {
        let cost_to_sort_row = |y: u8| -> u32 {
            (0..grid.width())
                .map(move |x| grid.clamping_pos(x, y))
                .map(|pos| grid.looping_manhattan_dist(pos, nodes[pos]))
                .sum()
        };
        row_to_sort.sort_by(|&a, &b| cost_to_sort_row(b).cmp(&cost_to_sort_row(a)));
        let y = row_to_sort.pop().unwrap();
        eprintln!("start to sort the row: {}", y);
        let row_completer = RowCompleter {
            field: nodes.clone(),
            selecting: selection,
            target_row: y,
            prev_action: all_actions.last().copied(),
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        };
        let (mut actions, _cost) = ida_star(row_completer, 0);
        for &action in &actions {
            match action {
                GridAction::Swap(mov) => {
                    let sel = selection.unwrap();
                    let swap_to = grid.move_pos_to(sel, mov);
                    nodes.swap(sel, swap_to);
                }
                GridAction::Select(sel) => {
                    selection = Some(sel);
                    select_limit -= 1;
                }
            }
        }
        eprintln!("sort result: {:?}", actions);
        all_actions.append(&mut actions);
    }
    let different_cells = DifferentCells::new(&nodes);
    let (mut actions, _total_cost) = ida_star(
        GridCompleter {
            field: nodes.clone(),
            selecting: selection,
            prev_action: all_actions.last().copied(),
            different_cells,
            swap_cost,
            select_cost,
            remaining_select: select_limit,
        },
        different_cells.0,
    );
    all_actions.append(&mut actions);
    actions_to_operations(all_actions)
}
