use std::{
    collections::{BinaryHeap, HashSet},
    ops,
};

use crate::{
    basis::Movement,
    grid::{Grid, Pos, RangePos, VecOnGrid},
};

use super::GridAction;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TargetNode {
    target: Pos,
    cost: LeastMovements,
}
impl PartialOrd for TargetNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}
impl Ord for TargetNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

fn moves_to_sort(board: &Board, targets: &[Pos], range: RangePos) -> Vec<Pos> {
    let mut board = board.clone();
    let mut res = vec![];
    for &target in targets {
        if range.is_in(target) {
            board.lock(target);
            continue;
        }
        let mut way = moves_to_swap_target_to_goal(&board, target, range.clone());
        if way.is_empty() {
            return vec![];
        }
        for &mov in &way {
            board.swap_to(mov);
        }
        res.append(&mut way);
        board.lock(target);
    }
    let mut way = route_into_range(&board, board.select, range);
    if way.is_empty() {
        return vec![];
    }
    res.append(&mut way);
    res
}

/// target 位置のマスをそのゴール位置へ動かす実際の手順を決定する.
fn moves_to_swap_target_to_goal(board: &Board, target: Pos, range: RangePos) -> Vec<Pos> {
    let route = route_target_to_goal(board, target, range);
    if route.is_empty() {
        return vec![];
    }
    let mut board = board.clone();
    let mut current = target;
    let mut ret = vec![board.select];
    for way in route {
        board.lock(current);
        let route_to_arrive = route_select_to_target(&board, way);
        for way in route_to_arrive {
            board.swap_to(way);
            ret.push(way);
        }
        board.unlock(current);
        board.swap_to(current);
        ret.push(current);
        current = way;
    }
    ret
}

/// target 位置のマスを range の範囲内に収める最短経路を求める.
fn route_into_range(board: &Board, target: Pos, range: RangePos) -> Vec<Pos> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    heap.push(TargetNode {
        target,
        cost: LeastMovements(0),
    });
    shortest_cost[target] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if range.is_in(pick.target) {
            return extract_back_path(pick.target, back_path);
        }
        for next in board.around_of(pick.target) {
            let next_cost = pick.cost.swap_on(&board.field, pick.target, next) + LeastMovements(1);
            if shortest_cost[next] <= next_cost {
                continue;
            }
            shortest_cost[next] = next_cost;
            back_path[next] = Some(pick.target);
            heap.push(TargetNode {
                target: next,
                cost: next_cost,
            });
        }
    }
    vec![]
}

/// select を target へ動かす最短経路を決定する.
fn route_select_to_target(board: &Board, target: Pos) -> Vec<Pos> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    heap.push(TargetNode {
        target: board.select,
        cost: LeastMovements(0),
    });
    shortest_cost[board.select] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if pick.target == target {
            return extract_back_path(pick.target, back_path);
        }
        for next in board.around_of(pick.target) {
            let next_cost = pick.cost.swap_on(&board.field, pick.target, next);
            if shortest_cost[next] <= next_cost {
                continue;
            }
            shortest_cost[next] = next_cost;
            back_path[next] = Some(pick.target);
            heap.push(TargetNode {
                target: next,
                cost: next_cost,
            });
        }
    }
    vec![]
}

/// board が選択しているマスを target の隣へ動かす最短経路を決定する.
fn route_select_around_target(board: &Board, target: Pos) -> (Vec<Pos>, LeastMovements) {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    heap.push(TargetNode {
        target: board.select,
        cost: LeastMovements(0),
    });
    shortest_cost[board.select] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if board.grid().looping_manhattan_dist(pick.target, target) == 1 {
            return (extract_back_path(pick.target, back_path), pick.cost);
        }
        for next in board.around_of(pick.target) {
            // target とは入れ替えない
            if next == target {
                continue;
            }
            let next_cost = pick.cost.swap_on(&board.field, pick.target, next) + LeastMovements(1);
            if shortest_cost[next] <= next_cost {
                continue;
            }
            shortest_cost[next] = next_cost;
            back_path[next] = Some(pick.target);
            heap.push(TargetNode {
                target: next,
                cost: next_cost,
            });
        }
    }
    (vec![], LeastMovements(0))
}

/// target 位置のマスをそのゴール位置へ動かす最短経路を決定する.
fn route_target_to_goal(board: &Board, target: Pos, range: RangePos) -> Vec<Pos> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    #[derive(Debug, Clone)]
    struct RowCompleteNode<'grid> {
        target: Pos,
        cost: LeastMovements,
        board: Board<'grid>,
    }
    impl PartialEq for RowCompleteNode<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.cost == other.cost
        }
    }
    impl Eq for RowCompleteNode<'_> {}
    impl PartialOrd for RowCompleteNode<'_> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            other.cost.partial_cmp(&self.cost)
        }
    }
    impl Ord for RowCompleteNode<'_> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    let mut heap = BinaryHeap::new();
    heap.push(RowCompleteNode {
        target,
        cost: LeastMovements(0),
        board: board.clone(),
    });
    shortest_cost[target] = LeastMovements(0);
    while let Some(mut pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if range.is_in(pick.target) {
            return extract_back_path(pick.target, back_path);
        }
        pick.board.lock(pick.target);
        let pick = pick;
        for next_pos in board.around_of(pick.target) {
            if shortest_cost[next_pos] <= pick.cost {
                continue;
            }
            let (moves_to_around, cost) = route_select_around_target(&pick.board, pick.target);
            if moves_to_around.is_empty() {
                continue;
            }
            let mut next_node = pick.clone();
            next_node.cost += cost;
            for to in moves_to_around {
                next_node.board.swap_to(to);
            }
            // 隣に移動していなければならない
            assert_eq!(
                next_node
                    .board
                    .grid()
                    .looping_manhattan_dist(next_pos, next_node.board.select),
                1
            );
            // コストだけ先に計算
            next_node.cost = next_node
                .cost
                .swap_on(&next_node.board.field, pick.target, next_pos)
                + LeastMovements(1);
            if shortest_cost[next_pos] <= next_node.cost {
                continue;
            }
            // この手順がより短かったので適用
            shortest_cost[next_pos] = next_node.cost;
            next_node.board.unlock(next_pos);
            next_node.board.swap_to(next_pos);
            back_path[next_pos] = Some(pick.target);
            heap.push(next_node);
        }
    }
    vec![]
}

fn extract_back_path(mut pos: Pos, back_path: VecOnGrid<Option<Pos>>) -> Vec<Pos> {
    let mut history = vec![pos];
    while let Some(back) = back_path[pos] {
        history.push(back);
        pos = back;
    }
    history.reverse();
    history
}
