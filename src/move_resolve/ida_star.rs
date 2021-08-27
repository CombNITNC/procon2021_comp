use std::ops::Add;

/// A* で探索する状態が実装するべき trait.
pub trait State<C> {
    type NextStates: IntoIterator<Item = Self>;
    fn next_states(&self, history: &[Self]) -> Self::NextStates
    where
        Self: Sized;

    fn is_goal(&self) -> bool;

    fn heuristic(&self) -> C;

    fn cost_between(&self, next: &Self) -> C;
}

#[derive(Debug)]
enum FindResult<C> {
    Found,
    Deeper(C),
    None,
}

fn find<V, N, C>(history: &mut Vec<V>, distance: C, bound: C) -> FindResult<C>
where
    V: PartialEq + Clone + State<C, NextStates = N> + std::fmt::Debug,
    N: IntoIterator<Item = V>,
    C: PartialOrd + Add<Output = C> + Copy + std::fmt::Debug,
{
    let visiting = history.last().cloned().unwrap();
    let total_estimated = distance + visiting.heuristic();
    if bound < total_estimated {
        return FindResult::Deeper(total_estimated);
    }
    if visiting.is_goal() {
        return FindResult::Found;
    }
    let mut min = None;
    for neighbor in visiting.next_states(history) {
        if !history.contains(&neighbor) {
            history.push(neighbor.clone());
            let next_distance = distance + visiting.cost_between(&neighbor);
            match find(history, next_distance, bound) {
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
pub fn ida_star<V, N, C>(start: V, lower_bound: C) -> impl Iterator<Item = (Vec<V>, C)>
where
    V: PartialEq + Clone + State<C, NextStates = N> + std::fmt::Debug,
    N: IntoIterator<Item = V>,
    C: PartialOrd + Default + Add<Output = C> + Copy + std::fmt::Debug,
{
    let mut history = vec![start];
    let mut bound = lower_bound;
    std::iter::from_fn(move || loop {
        match find(&mut history, C::default(), bound) {
            FindResult::Found => return Some((history.clone(), bound)),
            FindResult::Deeper(cost) => bound = cost,
            FindResult::None => return None,
        }
    })
}
