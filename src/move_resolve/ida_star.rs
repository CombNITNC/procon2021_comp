use std::ops::Add;

#[derive(Debug)]
enum FindResult<C> {
    Found,
    Deeper(C),
    None,
}

fn find<V, N, C>(
    history: &mut Vec<V>,
    neighbors: &mut impl FnMut(&V) -> N,
    is_goal: &mut impl FnMut(&V) -> bool,
    heuristic: &mut impl FnMut(&V) -> C,
    step_cost: &mut impl FnMut(&V, &V) -> C,
    distance: C,
    bound: C,
) -> FindResult<C>
where
    V: PartialEq + Clone,
    N: IntoIterator<Item = V>,
    C: PartialOrd + Add<Output = C> + Copy,
{
    let visiting = history.last().cloned().unwrap();
    let total_estimated = distance + heuristic(&visiting);
    if bound < total_estimated {
        return FindResult::Deeper(total_estimated);
    }
    if is_goal(&visiting) {
        return FindResult::Found;
    }
    let mut min = None;
    for neighbor in neighbors(&visiting) {
        if !history.contains(&neighbor) {
            history.push(neighbor.clone());
            let next_distance = distance + step_cost(&visiting, &neighbor);
            match find(
                history,
                neighbors,
                is_goal,
                heuristic,
                step_cost,
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
            history.pop();
        }
    }
    match min {
        Some(cost) => FindResult::Deeper(cost),
        None => FindResult::None,
    }
}

// 反復深化 A* アルゴリズムの実装.
pub fn ida_star<V, N, C>(
    start: V,
    mut neighbors: impl FnMut(&V) -> N,
    mut is_goal: impl FnMut(&V) -> bool,
    mut heuristic: impl FnMut(&V) -> C,
    mut step_cost: impl FnMut(&V, &V) -> C,
) -> (Vec<V>, C)
where
    V: PartialEq + Clone,
    N: IntoIterator<Item = V>,
    C: PartialOrd + Default + Add<Output = C> + Copy,
{
    let mut history = vec![start];
    let mut bound = C::default();
    loop {
        match find(
            &mut history,
            &mut neighbors,
            &mut is_goal,
            &mut heuristic,
            &mut step_cost,
            C::default(),
            bound,
        ) {
            FindResult::Found => return (history, bound),
            FindResult::Deeper(cost) => bound = cost,
            FindResult::None => return (vec![], C::default()),
        }
    }
}
