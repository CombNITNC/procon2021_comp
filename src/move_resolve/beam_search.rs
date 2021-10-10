use std::{
    cmp::Ordering,
    collections::{hash_map::DefaultHasher, BinaryHeap, HashSet},
    hash::{Hash, Hasher},
    ops::Add,
};

use super::SearchState;

pub(crate) fn beam_search<S, A, C>(
    initial_state: S,
    beam_width: usize,
    max_cost: C,
) -> Option<(Vec<A>, C)>
where
    S: SearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    if initial_state.is_goal() {
        return Some((vec![], C::default()));
    }

    let mut heap = BinaryHeap::with_capacity(beam_width);
    let mut visited = HashSet::new();

    heap.push(Node {
        state: initial_state,
        answer: vec![],
        answer_hasher: DefaultHasher::new(),
        cost: C::default(),
    });

    while !heap.is_empty() {
        let mut next_heap = BinaryHeap::with_capacity(beam_width);

        for _ in 0..beam_width.min(heap.len()) {
            let Node {
                state,
                answer,
                cost,
                answer_hasher,
            } = heap.pop().unwrap();

            for action in state.next_actions() {
                let mut next_hasher = answer_hasher.clone();
                action.hash(&mut next_hasher);
                if !visited.contains(&next_hasher.finish()) {
                    let next_cost = cost + state.cost_on(action);

                    if max_cost <= next_cost {
                        continue;
                    }

                    let next_state = state.apply(action);
                    let mut next_answer = answer.clone();
                    next_answer.push(action);
                    if next_state.is_goal() {
                        return Some((next_answer, next_cost));
                    }
                    visited.insert(answer_hasher.finish());

                    next_heap.push(Node {
                        state: next_state,
                        answer: next_answer,
                        cost: next_cost,
                        answer_hasher: next_hasher,
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
    answer_hasher: DefaultHasher,
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
