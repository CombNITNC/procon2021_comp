use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Add,
};

/// IDA* 探索する状態が実装するべき trait.
pub(crate) trait IdaSearchState: Clone + std::fmt::Debug {
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
    hasher: impl Hasher + Clone,
    distance: C,
    bound: C,
) -> FindResult<C>
where
    V: IdaSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Eq + Hash,
    C: PartialOrd + Add<Output = C> + Copy + std::fmt::Debug,
{
    let total_estimated = distance + node.heuristic();
    if bound < total_estimated {
        return FindResult::Deeper(total_estimated);
    }
    if node.is_goal() {
        return FindResult::Found;
    }
    let mut min = None;
    for action in node.next_actions() {
        history.push(action);
        let next_distance = distance + node.cost_on(action);
        let mut next_hasher = hasher.clone();
        action.hash(&mut next_hasher);
        if hasher.finish() != next_hasher.finish() {
            match find(
                node.apply(action),
                history,
                next_hasher,
                next_distance,
                bound,
            ) {
                FindResult::Found => return FindResult::Found,
                FindResult::Deeper(cost) => {
                    if min.map_or(true, |c| cost < c) {
                        min.replace(cost);
                    }
                }
                _ => {}
            }
        }
        history.pop();
    }
    match min {
        Some(cost) => FindResult::Deeper(cost),
        None => FindResult::None,
    }
}

/// 反復深化 A* アルゴリズムの実装.
pub(crate) fn ida_star<V, A, C>(start: V, lower_bound: C) -> (Vec<A>, C)
where
    V: IdaSearchState<C = C, A = A>,
    A: Copy + std::fmt::Debug + Hash + Eq,
    C: PartialOrd + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    let mut history = vec![];
    let mut bound = lower_bound;
    loop {
        let hasher = DefaultHasher::new();
        match find(start.clone(), &mut history, hasher, C::default(), bound) {
            FindResult::Found => return (history, bound),
            FindResult::Deeper(cost) => bound = cost,
            FindResult::None => return (vec![], C::default()),
        }
    }
}
