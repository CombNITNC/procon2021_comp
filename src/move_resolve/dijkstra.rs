use std::{cmp::Reverse, collections::BinaryHeap};

use crate::grid::{board::Board, Pos, VecOnGrid};

pub(crate) trait DijkstraCost: Copy + Ord + std::fmt::Debug {
    const IDENTITY: Self;

    fn op(self, other: Self) -> Self;
}

pub(crate) trait DijkstraState: Clone + std::fmt::Debug {
    type C: DijkstraCost;
    fn cost(&self) -> Self::C;

    fn as_pos(&self) -> Pos;

    fn is_goal(&self) -> bool;

    type AS: IntoIterator<Item = Pos>;
    fn next_actions(&mut self) -> Self::AS;

    fn apply(&self, new_pos: Pos) -> Option<Self>;
}

struct DijkstraNode<S: DijkstraState>(S);

impl<S: DijkstraState> PartialEq for DijkstraNode<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.cost() == other.0.cost()
    }
}
impl<S: DijkstraState> Eq for DijkstraNode<S> {}
impl<S: DijkstraState> PartialOrd for DijkstraNode<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.cost().partial_cmp(&other.0.cost())
    }
}
impl<S: DijkstraState> Ord for DijkstraNode<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cost().cmp(&other.0.cost())
    }
}

pub(crate) fn dijkstra<S, C>(board: &Board, start: S) -> Option<(Vec<Pos>, C)>
where
    S: DijkstraState<C = C>,
    C: DijkstraCost,
{
    let mut shortest_cost = VecOnGrid::with_init(board.grid(), C::IDENTITY);
    let mut back_path = VecOnGrid::with_init(board.grid(), None);

    shortest_cost[start.as_pos()] = start.cost();
    let mut heap = BinaryHeap::new();
    heap.push(Reverse(DijkstraNode(start)));
    while let Some(Reverse(DijkstraNode(mut pick))) = heap.pop() {
        if shortest_cost[pick.as_pos()] != pick.cost() {
            continue;
        }
        if pick.is_goal() {
            return Some((extract_back_path(pick.as_pos(), back_path), pick.cost()));
        }
        for next in pick.next_actions() {
            if shortest_cost[next] <= pick.cost() {
                continue;
            }
            if let Some(applied) = pick.apply(next) {
                if shortest_cost[applied.as_pos()] <= applied.cost() {
                    continue;
                }
                shortest_cost[applied.as_pos()] = applied.cost();
                back_path[applied.as_pos()] = Some(pick.as_pos());
                heap.push(Reverse(DijkstraNode(applied)));
            }
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
