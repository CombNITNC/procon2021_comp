use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    hash::Hash,
    iter::FromIterator,
    ops::Add,
    sync::Mutex,
};

use rayon::iter::{ParallelBridge, ParallelIterator};

use super::SearchState;

pub(crate) fn beam_search<S, A, C>(
    initial_state: S,
    beam_width: usize,
    max_cost: C,
) -> Option<(Vec<A>, C)>
where
    S: SearchState<C = C, A = A> + Hash + Eq + Send,
    A: Copy + std::fmt::Debug + Hash + Eq + Send,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug + Send + Sync,
    <<S as SearchState>::AS as IntoIterator>::IntoIter: Send,
{
    if initial_state.is_goal() {
        return Some((vec![], C::default()));
    }

    let mut heap = BinaryHeap::with_capacity(beam_width);
    let visited: Mutex<_> = HashSet::new().into();

    visited.lock().unwrap().insert(initial_state.clone());
    heap.push(Node {
        state: initial_state,
        answer: vec![],
        cost: C::default(),
    });

    while !heap.is_empty() {
        let heap_len = heap.len();
        let nexts: Vec<_> = heap
            .into_iter()
            .take(beam_width.min(heap_len))
            .par_bridge()
            .flat_map(
                |Node {
                     state,
                     answer,
                     cost,
                 }| {
                    let mut next_states = vec![];
                    for action in state.next_actions() {
                        let next_state = state.apply(action);
                        if !visited.lock().unwrap().contains(&next_state) {
                            let next_cost = cost + state.cost_on(action);

                            if max_cost <= next_cost {
                                continue;
                            }

                            let mut next_answer = answer.clone();
                            next_answer.push(action);

                            visited.lock().unwrap().insert(next_state.clone());
                            next_states.push(Node {
                                state: next_state,
                                answer: next_answer,
                                cost: next_cost,
                            });
                        }
                    }
                    next_states
                },
            )
            .collect();

        heap = BinaryHeap::from_iter(nexts.into_iter());
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
