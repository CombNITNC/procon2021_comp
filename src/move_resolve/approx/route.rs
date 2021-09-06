use std::collections::BinaryHeap;

use super::LeastMovements;
use crate::grid::{board::Board, Pos, RangePos, VecOnGrid};

#[cfg(test)]
mod tests;

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

/// `target` 位置のマスをそのゴール位置へ動かす実際の手順を決定する.
pub(super) fn moves_to_swap_target_to_goal(
    board: &Board,
    target: Pos,
    range: RangePos,
) -> Option<Vec<Pos>> {
    let route = route_target_to_goal(board, target, range)?;
    let mut board = board.clone();
    let mut current = target;
    let mut ret = vec![board.select()];
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
    Some(ret)
}

/// `target` 位置のマスを `pos` の位置へ移動させる最短経路を求める.
pub(super) fn route_target_to_pos(board: &Board, target: Pos, pos: Pos) -> Option<Vec<Pos>> {
    pub(super) fn route_target_around_pos(
        board: &Board,
        target: Pos,
        pos: Pos,
    ) -> Option<(Vec<Pos>, LeastMovements)> {
        let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
        let mut back_path = VecOnGrid::with_init(board.grid(), None);

        let mut heap = BinaryHeap::new();
        let cost = LeastMovements::new(board.field());
        heap.push(TargetNode { target, cost });
        shortest_cost[target] = cost;
        while let Some(pick) = heap.pop() {
            if shortest_cost[pick.target] != pick.cost {
                continue;
            }
            if board.grid().looping_manhattan_dist(pick.target, pos) == 1 {
                return Some((extract_back_path(pick.target, back_path), pick.cost));
            }
            for next_pos in board.around_of(pick.target) {
                let next_cost =
                    pick.cost.swap_on(board.field(), pick.target, next_pos) + LeastMovements(1);
                if shortest_cost[next_pos] <= next_cost {
                    continue;
                }
                shortest_cost[next_pos] = next_cost;
                back_path[next_pos] = Some(pick.target);
                heap.push(TargetNode {
                    target: next_pos,
                    cost: next_cost,
                });
            }
        }
        None
    }
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    let cost = LeastMovements::new(board.field());
    heap.push(RowCompleteNode {
        target,
        cost,
        board: board.clone(),
    });
    shortest_cost[target] = cost;
    while let Some(mut pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if pos == pick.target {
            return Some(extract_back_path(pick.target, back_path));
        }
        pick.board.lock(pick.target);
        let pick = pick;
        for next_pos in pick.board.around_of(pick.target) {
            if shortest_cost[next_pos] <= pick.cost {
                continue;
            }
            let route = route_target_around_pos(&pick.board, pick.board.select(), next_pos);
            if route.is_none() {
                continue;
            }
            let (route, cost) = route.unwrap();
            let mut next_node = pick.clone();
            for mov in route {
                next_node.board.swap_to(mov);
            }
            next_node.cost += cost;
            assert_eq!(
                pick.board
                    .grid()
                    .looping_manhattan_dist(pick.target, next_node.board.select()),
                1,
                "{:#?}",
                next_node
            );
            next_node.cost = next_node
                .cost
                .swap_on(next_node.board.field(), pick.target, next_pos)
                + LeastMovements(1);

            if shortest_cost[next_pos] <= next_node.cost {
                continue;
            }
            shortest_cost[next_pos] = next_node.cost;
            next_node.board.unlock(pick.target);
            next_node.board.swap_to(pick.target);
            next_node.target = next_pos;
            back_path[next_pos] = Some(pick.target);
            heap.push(next_node);
        }
    }
    None
}

/// `target` 位置のマスを `range` の範囲内に収める最短経路を求める.
fn route_into_range(board: &Board, target: Pos, range: RangePos) -> Option<Vec<Pos>> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    let cost = LeastMovements::new(board.field());
    heap.push(TargetNode { target, cost });
    shortest_cost[target] = cost;
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if range.is_in(pick.target) {
            return Some(extract_back_path(pick.target, back_path));
        }
        for next in board.around_of(pick.target) {
            let next_cost = pick.cost.swap_on(board.field(), pick.target, next) + LeastMovements(1);
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
    None
}

/// `board` の `select` を `target` へ動かす最短経路を決定する.
pub(super) fn route_select_to_target(board: &Board, target: Pos) -> Vec<Pos> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    let cost = LeastMovements::new(board.field());
    heap.push(TargetNode {
        target: board.select(),
        cost,
    });
    shortest_cost[board.select()] = cost;
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if pick.target == target {
            return extract_back_path(pick.target, back_path);
        }
        for next in board.around_of(pick.target) {
            let next_cost = pick.cost.swap_on(board.field(), pick.target, next);
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

/// `board` が選択しているマスを `target` の隣へ動かす最短経路を決定する.
fn route_select_around_target(board: &Board, target: Pos) -> Option<(Vec<Pos>, LeastMovements)> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    let cost = LeastMovements::new(board.field());
    heap.push(TargetNode {
        target: board.select(),
        cost,
    });
    shortest_cost[board.select()] = cost;
    while let Some(pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if board.grid().looping_manhattan_dist(pick.target, target) == 1 {
            return Some((extract_back_path(pick.target, back_path), pick.cost));
        }
        for next in board.around_of(pick.target) {
            // target とは入れ替えない
            if next == target {
                continue;
            }
            let next_cost = pick.cost.swap_on(board.field(), pick.target, next) + LeastMovements(1);
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
    None
}

/// `target` 位置のマスをそのゴール位置へ動かす最短経路を決定する.
fn route_target_to_goal(board: &Board, target: Pos, range: RangePos) -> Option<Vec<Pos>> {
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), LeastMovements(1_000_000_000));
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    let mut heap = BinaryHeap::new();
    let cost = LeastMovements::new(board.field());
    heap.push(RowCompleteNode {
        target,
        cost,
        board: board.clone(),
    });
    shortest_cost[target] = cost;
    while let Some(mut pick) = heap.pop() {
        if shortest_cost[pick.target] != pick.cost {
            continue;
        }
        if range.is_in(pick.target) {
            return Some(extract_back_path(pick.target, back_path));
        }
        pick.board.lock(pick.target);
        let pick = pick;
        for next_pos in pick.board.around_of(pick.target) {
            if shortest_cost[next_pos] <= pick.cost {
                continue;
            }
            let (moves_to_around, cost) = route_select_around_target(&pick.board, pick.target)?;
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
                    .looping_manhattan_dist(pick.target, next_node.board.select()),
                1,
                "{:#?}",
                next_node
            );
            // コストだけ先に計算
            next_node.cost = next_node
                .cost
                .swap_on(next_node.board.field(), pick.target, next_pos)
                + LeastMovements(1);
            if shortest_cost[next_pos] <= next_node.cost {
                continue;
            }
            // この手順がより短かったので適用
            shortest_cost[next_pos] = next_node.cost;
            back_path[next_pos] = Some(pick.target);
            next_node.board.unlock(pick.target);
            next_node.board.swap_to(pick.target);
            next_node.target = next_pos;
            heap.push(next_node);
        }
    }
    None
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
