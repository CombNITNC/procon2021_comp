use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    hash::Hash,
    ops::Add,
};

use super::SearchState;

pub(crate) fn beam_search<S, A, C>(
    initial_state: S,
    beam_width: usize,
    max_cost: C,
) -> Option<(Vec<A>, C)>
where
    S: SearchState<C = C, A = A> + Hash + Eq,
    A: Copy + std::fmt::Debug + Hash + Eq,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    if initial_state.is_goal() {
        return Some((vec![], C::default()));
    }

    let mut heap = BinaryHeap::with_capacity(beam_width);
    let mut visited = HashSet::new();

    visited.insert(initial_state.clone());
    heap.push(Node {
        state: initial_state,
        answer: vec![],
        cost: C::default(),
    });

    while !heap.is_empty() {
        let mut next_heap = BinaryHeap::with_capacity(beam_width);

        for _ in 0..beam_width.min(heap.len()) {
            let Node {
                state,
                answer,
                cost,
            } = heap.pop().unwrap();

            for action in state.next_actions() {
                let next_state = state.apply(action);
                if visited.insert(next_state.clone()) {
                    let next_cost = cost + state.cost_on(action);

                    if max_cost <= next_cost {
                        continue;
                    }

                    let mut next_answer = answer.clone();
                    next_answer.push(action);
                    if next_state.is_goal() {
                        return Some((next_answer, next_cost));
                    }

                    next_heap.push(Node {
                        state: next_state,
                        answer: next_answer,
                        cost: next_cost,
                    });
                }
            }
        }
        heap = next_heap;
    }
    None
}

struct Node<S, A, C> {
    state: S,
    answer: Vec<A>,
    cost: C,
}

impl<S, A, C: PartialEq> PartialEq for Node<S, A, C> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl<S, A, C: PartialEq + Eq> Eq for Node<S, A, C> {}

impl<S, A, C: PartialOrd> PartialOrd for Node<S, A, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

impl<S, A, C: PartialOrd + PartialEq + Eq + Ord> Ord for Node<S, A, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}
