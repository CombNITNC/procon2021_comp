use std::{cmp::Ordering, collections::BinaryHeap, hash::Hash, ops::Add, sync::Mutex};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use rayon::iter::{ParallelBridge, ParallelIterator};

/// ビームサーチする状態が実装するべき trait.
pub trait BeamSearchState: Clone + std::fmt::Debug + Hash + Eq + Send + Sync {
    type A: Copy + std::fmt::Debug + Send;
    fn apply(&self, action: Self::A) -> Self;

    type AS: IntoIterator<Item = Self::A> + Send;
    fn next_actions(&self) -> Self::AS;

    fn is_goal(&self) -> bool;

    type C: Copy + Ord + std::fmt::Debug + Send + Sync;
    fn cost_on(&self, action: Self::A) -> Self::C;

    fn max_cost(&self) -> Self::C;

    fn enrichment_key(&self) -> usize;
}

pub fn beam_search<S, A, C>(
    initial_state: S,
    beam_width: usize,
) -> impl Iterator<Item = (Vec<A>, C)>
where
    S: BeamSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq + Send,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug + Send + Sync,
    <<S as BeamSearchState>::AS as IntoIterator>::IntoIter: Send,
{
    let max_cost = initial_state.max_cost();

    let mut heap = BinaryHeap::with_capacity(beam_width);
    let mut visited_goals = HashSet::default();

    std::iter::from_fn(move || {
        if initial_state.is_goal() {
            return Some((vec![], C::default()));
        }

        let mut visited = HashSet::default();

        visited.insert(initial_state.clone());
        visited.extend(visited_goals.iter().cloned());

        heap.push(Node {
            state: initial_state.clone(),
            answer: vec![],
            cost: C::default(),
        });

        'search: loop {
            let nexts = Mutex::new(HashMap::default());
            nexts.lock().unwrap().reserve(beam_width);
            heap.clone()
                .into_iter()
                .par_bridge()
                .for_each(search_nexts(beam_width, max_cost, &visited, &nexts));
            let nexts = nexts.into_inner().unwrap();
            if nexts.is_empty() {
                break None;
            }
            for next in nexts.values().flat_map(|heap| heap.iter()) {
                if next.state.is_goal() {
                    visited_goals.insert(next.state.clone());
                    break 'search Some((next.answer.clone(), next.cost));
                }
                visited.insert(next.state.clone());
            }
            let kinds_of_key = nexts.len();
            let take_len = beam_width / kinds_of_key;
            let mut new_heap = BinaryHeap::with_capacity(beam_width);
            for mut next in nexts.into_values().take(take_len) {
                new_heap.append(&mut next);
            }
            heap = new_heap;
        }
    })
}

type NextsMap<S, A, C> = HashMap<usize, BinaryHeap<Node<S, A, C>>>;

fn search_nexts<'a, S, A, C>(
    beam_width: usize,
    max_cost: C,
    visited: &'a HashSet<S>,
    nexts: &'a Mutex<NextsMap<S, A, C>>,
) -> impl Fn(Node<S, A, C>) + 'a
where
    S: BeamSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq + Send,
    C: Ord + Add<Output = C> + Default + Copy + std::fmt::Debug + Send + Sync,
    <<S as BeamSearchState>::AS as IntoIterator>::IntoIter: Send,
{
    move |Node {
              state,
              answer,
              cost,
          }| {
        if max_cost <= cost {
            return;
        }

        for action in state.next_actions() {
            let next_cost = cost + state.cost_on(action);
            let next_state = state.apply(action);
            if !visited.contains(&next_state) {
                let mut next_answer = answer.clone();
                next_answer.push(action);
                nexts
                    .lock()
                    .unwrap()
                    .entry(next_state.enrichment_key())
                    .or_insert_with(|| BinaryHeap::with_capacity(beam_width))
                    .push(Node {
                        state: next_state,
                        answer: next_answer,
                        cost: next_cost,
                    });
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Node<S, A, C> {
    state: S,
    answer: Vec<A>,
    cost: C,
}

impl<S: Hash, A, C> Hash for Node<S, A, C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
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
