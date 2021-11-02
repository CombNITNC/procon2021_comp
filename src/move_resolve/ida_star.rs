use std::{hash::Hash, ops::Add};

use fxhash::FxHashSet as HashSet;

/// IDA* 探索する状態が実装するべき trait.
pub trait IdaSearchState: Hash + Eq + Clone + std::fmt::Debug {
    type A: Copy + std::fmt::Debug;
    fn apply(&self, action: Self::A) -> Self;

    type AS: IntoIterator<Item = Self::A>;
    fn next_actions(&self) -> Self::AS;

    fn is_goal(&self) -> bool;

    type C: Copy + Ord + std::fmt::Debug;
    fn heuristic(&self) -> Self::C;
    fn cost_on(&self, action: Self::A) -> Self::C;
}

#[derive(Debug)]
enum FindResult<C> {
    Found,
    Deeper(C),
    None,
}

fn find<V, A, C>(
    node: V,
    history: &mut Vec<A>,
    visited: &mut HashSet<V>,
    distance: C,
    bound: C,
    limit_cost: C,
) -> FindResult<C>
where
    V: IdaSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug,
    C: PartialOrd + Add<Output = C> + Copy + std::fmt::Debug,
{
    if limit_cost <= distance {
        return FindResult::None;
    }
    let total_estimated = distance + node.heuristic();
    if bound < total_estimated {
        return FindResult::Deeper(total_estimated);
    }
    if node.is_goal() {
        return FindResult::Found;
    }
    let mut min = None;
    for action in node.next_actions() {
        let next_state = node.apply(action);
        if visited.insert(next_state.clone()) {
            history.push(action);
            let next_distance = distance + node.cost_on(action);
            match find(
                next_state,
                history,
                visited,
                next_distance,
                bound,
                limit_cost,
            ) {
                FindResult::Found => return FindResult::Found,
                FindResult::Deeper(cost) => {
                    if min.map_or(true, |c| cost < c) {
                        min.replace(cost);
                    }
                }
                _ => {}
            }
            history.pop();
        }
    }
    match min {
        Some(cost) => FindResult::Deeper(cost),
        None => FindResult::None,
    }
}

/// 反復深化 A* アルゴリズムの実装.
pub fn ida_star<V, A, C>(start: V, lower_bound: C, limit_cost: C) -> Option<(Vec<A>, C)>
where
    V: IdaSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq,
    C: PartialOrd + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    let mut history = vec![];
    let mut bound = lower_bound;
    let mut visited = HashSet::default();
    loop {
        match find(
            start.clone(),
            &mut history,
            &mut visited,
            C::default(),
            bound,
            limit_cost,
        ) {
            FindResult::Found => return Some((history, bound)),
            FindResult::Deeper(cost) => {
                visited.clear();
                bound = cost;
            }
            FindResult::None => return None,
        }
    }
}
