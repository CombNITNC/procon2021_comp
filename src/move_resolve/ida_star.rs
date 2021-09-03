use std::ops::Add;

/// A* で探索する状態が実装するべき trait.
pub trait IdaStarState: PartialEq + Clone + std::fmt::Debug {
    type A: Copy + std::fmt::Debug;
    fn apply(&self, action: Self::A) -> Self;

    type AS: IntoIterator<Item = Self::A>;
    fn next_actions(&self, history: &[Self::A]) -> Self::AS;

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

fn find<V, A, C>(node: V, history: &mut Vec<A>, distance: C, bound: C) -> FindResult<C>
where
    V: IdaStarState<C = C, A = A>,
    A: Copy + std::fmt::Debug,
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
    for action in node.next_actions(history) {
        history.push(action);
        let next_distance = distance + node.cost_on(action);
        match find(node.apply(action), history, next_distance, bound) {
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
    match min {
        Some(cost) => FindResult::Deeper(cost),
        None => FindResult::None,
    }
}

/// 反復深化 A* アルゴリズムの実装.
pub fn ida_star<V, A, C>(start: V, lower_bound: C) -> (Vec<A>, C)
where
    V: IdaStarState<C = C, A = A>,
    A: Copy + std::fmt::Debug,
    C: PartialOrd + Add<Output = C> + Default + Copy + std::fmt::Debug,
{
    let mut history = vec![];
    let mut bound = lower_bound;
    loop {
        match find(start.clone(), &mut history, C::default(), bound) {
            FindResult::Found => return (history, bound),
            FindResult::Deeper(cost) => bound = cost,
            FindResult::None => return (vec![], C::default()),
        }
    }
}
