use std::{
    cmp::Ordering,
    collections::{hash_map::DefaultHasher, BinaryHeap, HashSet},
    hash::{Hash, Hasher},
    ops::Add,
};

use super::SearchState;

pub(crate) fn beam_search<S, A, C>(initial_state: S, beam_width: usize) -> (Vec<A>, C)
where
    S: SearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    let mut heap = BinaryHeap::new();
    let mut visited = HashSet::new();

    heap.push(Node {
        state: initial_state,
        answer: vec![],
        answer_hasher: DefaultHasher::new(),
        score: C::default(),
    });

    while let Some(Node {
        state,
        answer,
        answer_hasher,
        score,
    }) = heap.pop()
    {
        if state.is_goal() {
            return (answer, score);
        }
        visited.insert(answer_hasher.finish());

        let mut next_heap = BinaryHeap::new();

        for _ in 0..beam_width.min(heap.len()) {
            for action in state.next_actions() {
                let mut next_hasher = answer_hasher.clone();
                action.hash(&mut next_hasher);
                if !visited.contains(&next_hasher.finish()) {
                    let next_state = state.apply(action);
                    let mut next_answer = answer.clone();
                    next_answer.push(action);
                    let next_score = state.cost_on(action);
                    next_heap.push(Node {
                        state: next_state,
                        answer: next_answer,
                        score: next_score,
                        answer_hasher: next_hasher,
                    });
                }
            }
        }
        heap = next_heap;
    }
    (vec![], C::default())
}

struct Node<S, A, C> {
    state: S,
    answer: Vec<A>,
    score: C,
    answer_hasher: DefaultHasher,
}

impl<S, A, C: PartialEq> PartialEq for Node<S, A, C> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<S, A, C: PartialEq + Eq> Eq for Node<S, A, C> {}

impl<S, A, C: PartialOrd> PartialOrd for Node<S, A, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<S, A, C: PartialOrd + PartialEq + Eq + Ord> Ord for Node<S, A, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
