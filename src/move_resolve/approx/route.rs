use std::collections::BinaryHeap;

use super::LeastMovements;
use crate::{
    grid::{board::Board, Pos, RangePos, VecOnGrid},
    move_resolve::dijkstra::{dijkstra, DijkstraState},
};

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
                    .looping_manhattan_dist(next_pos, next_node.board.select()),
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

fn route_target_around_pos(
    board: &Board,
    target: Pos,
    pos: Pos,
) -> Option<(Vec<Pos>, LeastMovements)> {
    #[derive(Debug, Clone)]
    struct RouteTargetAroundPos<'b> {
        node: TargetNode,
        board: &'b Board<'b>,
        pos: Pos,
    }
    impl DijkstraState for RouteTargetAroundPos<'_> {
        type C = LeastMovements;
        fn cost(&self) -> Self::C {
            self.node.cost
        }

        fn as_pos(&self) -> Pos {
            self.node.target
        }

        fn is_goal(&self) -> bool {
            self.board
                .grid()
                .looping_manhattan_dist(self.as_pos(), self.pos)
                == 1
        }

        type AS = Vec<Pos>;
        fn next_actions(&self) -> Self::AS {
            self.board.around_of(self.as_pos())
        }

        fn apply(&self, new_pos: Pos) -> Option<Self> {
            let new_cost = self
                .cost()
                .swap_on(self.board.field(), self.as_pos(), new_pos)
                + LeastMovements(1);
            Some(Self {
                node: TargetNode {
                    cost: new_cost,
                    target: new_pos,
                },
                ..self.clone()
            })
        }
    }
    dijkstra(
        board,
        RouteTargetAroundPos {
            node: TargetNode {
                target,
                cost: LeastMovements::new(board.field()),
            },
            board,
            pos,
        },
    )
}

/// `target` 位置のマスを `range` の範囲内に収める最短経路を求める.
fn route_into_range(board: &Board, target: Pos, range: RangePos) -> Option<Vec<Pos>> {
    #[derive(Debug, Clone)]
    struct RouteIntoRange<'b> {
        node: TargetNode,
        board: &'b Board<'b>,
        range: RangePos,
    }
    impl DijkstraState for RouteIntoRange<'_> {
        type C = LeastMovements;
        fn cost(&self) -> Self::C {
            self.node.cost
        }

        fn as_pos(&self) -> Pos {
            self.node.target
        }

        fn is_goal(&self) -> bool {
            self.range.is_in(self.as_pos())
        }

        type AS = Vec<Pos>;
        fn next_actions(&self) -> Self::AS {
            self.board.around_of(self.as_pos())
        }

        fn apply(&self, new_pos: Pos) -> Option<Self> {
            let new_cost = self
                .cost()
                .swap_on(self.board.field(), self.as_pos(), new_pos)
                + LeastMovements(1);
            Some(Self {
                node: TargetNode {
                    cost: new_cost,
                    target: new_pos,
                },
                ..self.clone()
            })
        }
    }
    dijkstra(
        board,
        RouteIntoRange {
            node: TargetNode {
                target,
                cost: LeastMovements::new(board.field()),
            },
            board,
            range,
        },
    )
    .map(|res| res.0)
}

/// `board` の `select` を `target` へ動かす最短経路を決定する.
pub(super) fn route_select_to_target(board: &Board, target: Pos) -> Vec<Pos> {
    #[derive(Debug, Clone)]
    struct RouteSelectToTarget<'b> {
        node: TargetNode,
        board: &'b Board<'b>,
        target: Pos,
    }
    impl DijkstraState for RouteSelectToTarget<'_> {
        type C = LeastMovements;
        fn cost(&self) -> Self::C {
            self.node.cost
        }

        fn as_pos(&self) -> Pos {
            self.node.target
        }

        fn is_goal(&self) -> bool {
            self.as_pos() == self.target
        }

        type AS = Vec<Pos>;
        fn next_actions(&self) -> Self::AS {
            self.board.around_of(self.as_pos())
        }

        fn apply(&self, new_pos: Pos) -> Option<Self> {
            let new_cost = self
                .cost()
                .swap_on(self.board.field(), self.as_pos(), new_pos);
            Some(Self {
                node: TargetNode {
                    target: new_pos,
                    cost: new_cost,
                },
                ..self.clone()
            })
        }
    }
    dijkstra(
        board,
        RouteSelectToTarget {
            node: TargetNode {
                target: board.select(),
                cost: LeastMovements::new(board.field()),
            },
            board,
            target,
        },
    )
    .unwrap()
    .0
}

/// `board` が選択しているマスを `target` の隣へ動かす最短経路を決定する.
fn route_select_around_target(board: &Board, target: Pos) -> Option<(Vec<Pos>, LeastMovements)> {
    #[derive(Debug, Clone)]
    struct RouteSelectAroundTarget<'b> {
        node: TargetNode,
        board: &'b Board<'b>,
        target: Pos,
    }
    impl DijkstraState for RouteSelectAroundTarget<'_> {
        type C = LeastMovements;
        fn cost(&self) -> Self::C {
            self.node.cost
        }

        fn as_pos(&self) -> Pos {
            self.node.target
        }

        fn is_goal(&self) -> bool {
            self.board
                .grid()
                .looping_manhattan_dist(self.as_pos(), self.target)
                == 1
        }

        type AS = Vec<Pos>;
        fn next_actions(&self) -> Self::AS {
            self.board.around_of(self.as_pos())
        }

        fn apply(&self, new_pos: Pos) -> Option<Self> {
            // target とは入れ替えない
            if new_pos == self.as_pos() {
                return None;
            }
            let new_cost = self
                .node
                .cost
                .swap_on(self.board.field(), self.as_pos(), new_pos)
                + LeastMovements(1);
            Some(Self {
                node: TargetNode {
                    target: new_pos,
                    cost: new_cost,
                },
                ..self.clone()
            })
        }
    }
    dijkstra(
        board,
        RouteSelectAroundTarget {
            node: TargetNode {
                target: board.select(),
                cost: LeastMovements::new(board.field()),
            },
            board,
            target,
        },
    )
}

/// `target` 位置のマスをそのゴール位置へ動かす最短経路を決定する.
fn route_target_to_goal(board: &Board, target: Pos, range: RangePos) -> Option<Vec<Pos>> {
    #[derive(Debug, Clone)]
    struct RouteSelectAroundTarget<'b> {
        node: RowCompleteNode<'b>,
        range: &'b RangePos,
        target: Pos,
    }
    impl DijkstraState for RouteSelectAroundTarget<'_> {
        type C = LeastMovements;
        fn cost(&self) -> Self::C {
            self.node.cost
        }

        fn as_pos(&self) -> Pos {
            self.node.target
        }

        fn is_goal(&self) -> bool {
            self.range.is_in(self.as_pos())
        }

        type AS = Vec<Pos>;
        fn next_actions(&self) -> Self::AS {
            self.node.board.around_of(self.as_pos())
        }

        fn apply(&self, new_pos: Pos) -> Option<Self> {
            let (moves_to_around, cost) =
                route_select_around_target(&self.node.board, self.target)?;
            let mut new_node = self.node.clone();
            new_node.cost += cost;
            for to in moves_to_around {
                new_node.board.swap_to(to);
            }
            // 隣に移動していなければならない
            assert_eq!(
                new_node
                    .board
                    .grid()
                    .looping_manhattan_dist(self.as_pos(), new_node.board.select()),
                1,
                "{:#?}",
                new_node
            );
            // コストだけ先に計算
            new_node.cost = new_node
                .cost
                .swap_on(new_node.board.field(), self.as_pos(), new_pos)
                + LeastMovements(1);
            new_node.board.unlock(self.as_pos());
            new_node.board.swap_to(self.as_pos());
            Some(Self {
                node: new_node,
                ..self.clone()
            })
        }
    }
    dijkstra(
        board,
        RouteSelectAroundTarget {
            node: RowCompleteNode {
                target,
                cost: LeastMovements::new(board.field()),
                board: board.clone(),
            },
            target,
            range: &range,
        },
    )
    .map(|res| res.0)
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
