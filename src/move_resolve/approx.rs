use std::{collections::BinaryHeap, ops};

use super::GridAction;
use crate::{
    basis::Movement,
    grid::{Pos, RangePos, VecOnGrid},
};

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

    fn move_on(self, field: &VecOnGrid<Pos>, from: Pos, to: Pos) -> Self {
        let before = least_movements(field.grid.looping_min_vec(from, field[from]));
        let after = least_movements(field.grid.looping_min_vec(to, field[from]));
        Self(4 + self.0 - before + after)
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
struct MoveToAroundNode {
    target: Pos,
    cost: LeastMovements,
}
impl PartialOrd for MoveToAroundNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}
impl Ord for MoveToAroundNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

fn path_to_move_select_around_target(
    field: &VecOnGrid<Pos>,
    target: Pos,
    select: Pos,
) -> (Vec<GridAction>, LeastMovements) {
    // ダイクストラ法で select を target の隣へ動かす経路を決定する.
    // コストは各マスの必要最低手数の合計.
    let mut shortest_cost = VecOnGrid::with_init(field.grid, LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(field.grid, None);

    let mut heap = BinaryHeap::new();
    heap.push(MoveToAroundNode {
        target: select,
        cost: LeastMovements(0),
    });
    shortest_cost[select] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if field.grid.looping_manhattan_dist(pick.target, target) == 1 {
            return (extract_back_path(pick.target, back_path), pick.cost);
        }
        for next in field.grid.around_of(pick.target) {
            // target とは入れ替えない
            if next == target {
                continue;
            }
            let next_cost = pick.cost.move_on(field, pick.target, next) + LeastMovements(1);
            if shortest_cost[next] <= next_cost {
                continue;
            }
            shortest_cost[next] = next_cost;
            back_path[next] = Some(pick.target);
            heap.push(MoveToAroundNode {
                target: next,
                cost: next_cost,
            });
        }
    }
    (vec![], LeastMovements(0))
}

fn extract_back_path(mut pos: Pos, back_path: VecOnGrid<Option<Pos>>) -> Vec<GridAction> {
    let mut history = vec![pos];
    while let Some(back) = back_path[pos] {
        history.push(back);
        pos = back;
    }
    history.reverse();
    history
        .windows(2)
        .map(|mov| Movement::between_pos(mov[0], mov[1]))
        .map(GridAction::Swap)
        .collect()
}

#[derive(Debug, Clone)]
struct RowCompleteNode<'grid> {
    selected: Pos,
    target: Pos,
    cost: LeastMovements,
    field: VecOnGrid<'grid, Pos>,
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

fn path_to_move_target_to_goal(
    field: &VecOnGrid<Pos>,
    selected: Pos,
    target: Pos,
    range: RangePos,
) -> Vec<GridAction> {
    // ダイクストラ法で target をゴール位置へ動かす経路を決定する.
    // コストは各マスの必要最低手数の合計.
    let mut shortest_cost = VecOnGrid::with_init(field.grid, LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(field.grid, None);

    let mut heap = BinaryHeap::new();
    heap.push(RowCompleteNode {
        selected,
        target,
        cost: LeastMovements(0),
        field: field.clone(),
    });
    shortest_cost[target] = LeastMovements(0);
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if range.is_in(pick.target) {
            return extract_back_path(pick.target, back_path);
        }
        for next_pos in pick.field.grid.around_of(pick.target) {
            if shortest_cost[next_pos] <= pick.cost {
                continue;
            }
            let (moves_to_around, cost) =
                path_to_move_select_around_target(&pick.field, pick.target, pick.selected);
            if moves_to_around.is_empty() {
                continue;
            }
            let mut next_node = pick.clone();
            next_node.cost += cost;
            for swap in moves_to_around {
                match swap {
                    GridAction::Swap(mov) => {
                        let next_swap = next_node.field.grid.move_pos_to(next_node.selected, mov);
                        next_node.field.swap(next_node.selected, next_swap);
                        next_node.selected = next_swap;
                    }
                    GridAction::Select(_) => unreachable!(),
                }
            }
            // 隣に移動していなければならない
            assert_eq!(
                next_node
                    .field
                    .grid
                    .looping_manhattan_dist(next_pos, next_node.selected),
                1
            );
            // コストだけ先に計算
            next_node.cost = next_node
                .cost
                .move_on(&next_node.field, pick.target, next_pos)
                + LeastMovements(1);
            if shortest_cost[next_pos] <= next_node.cost {
                continue;
            }
            // この手順がより短かったので適用
            shortest_cost[next_pos] = next_node.cost;
            next_node.field.swap(next_pos, next_node.selected);
            next_node.selected = next_pos;
            back_path[next_pos] = Some(pick.target);
            heap.push(next_node);
        }
    }
    todo!()
}
