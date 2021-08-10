use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, State},
    seg_tree::{Monoid, SegTree},
};
use crate::{
    basis::{Movement, Operation},
    grid::{Grid, Pos, VecOnGrid},
};
use im_rc::HashSet;
use petgraph::{
    algo::kosaraju_scc,
    graph::{IndexType, UnGraph},
};

mod edges_nodes;
mod ida_star;
mod seg_tree;
#[cfg(test)]
mod tests;

#[derive(Clone)]
struct GridState<'grid> {
    grid: &'grid Grid,
    field: VecOnGrid<'grid, Pos>,
    selecting: Option<Pos>,
    swap_cost: u16,
    select_cost: u16,
}

impl std::fmt::Debug for GridState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridState")
            .field("field", &self.field)
            .field("selecting", &self.selecting)
            .finish()
    }
}

impl PartialEq for GridState<'_> {
    fn eq(&self, other: &Self) -> bool {
        (&self.field)
            .into_iter()
            .zip(&other.field)
            .all(|(a, b)| a == b)
            && self.selecting == other.selecting
    }
}

/// 隣接マスどうしのマンハッタン距離が 1 かつ全頂点がゴール位置に無い集合の数を求める.
fn h1(state: &GridState) -> u64 {
    let mut edges = vec![];
    let mut points = HashSet::new();
    for pos in state.grid.all_pos() {
        for around in state.grid.around_of(pos) {
            if state.field[pos].manhattan_distance(state.field[around]) == 1 {
                edges.push((pos, around));
                points.insert(pos);
                points.insert(around);
            }
        }
    }
    let mut g = UnGraph::<Pos, (), Pos>::from_edges(edges);
    for pos in points {
        if let Some(weight) = g.node_weight_mut(pos.into()) {
            *weight = state.field[pos];
        }
    }
    let forest = kosaraju_scc(&g);
    forest
        .iter()
        .filter(|tree| {
            tree.iter()
                .all(|p| state.grid.is_pos_valid(<Pos as IndexType>::new(p.index())))
        })
        .filter(|tree| {
            tree.iter()
                .any(|&idx| <Pos as IndexType>::new(idx.index()) != g[idx])
        })
        .count() as u64
}

/// 選択中のマスを取り除いたときの転倒数を求める.
fn h2(state: &GridState) -> u64 {
    if state.selecting.is_none() {
        return 0;
    }
    let selecting = state.selecting.unwrap();
    let nums: Vec<_> = state
        .grid
        .all_pos()
        .filter(|&p| p != selecting)
        .map(|p| p.x() + state.grid.width() * p.y())
        .collect();

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct InversionCount(u8);
    impl Monoid for InversionCount {
        fn identity() -> Self {
            InversionCount(0)
        }

        fn op(self, other: Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    let mut tree = SegTree::<InversionCount>::new(nums.len());
    let mut inversions = 0u64;
    for (i, &n) in nums.iter().enumerate() {
        inversions += i as u64 - tree.query(0..i).0 as u64;
        let plus_one = InversionCount(1).op(tree[n as usize]);
        tree.insert(n as usize, plus_one);
    }
    inversions
}

impl<'grid> State<u64> for GridState<'grid> {
    type NextStates = Vec<GridState<'grid>>;
    fn next_states(&self, history: &[Self]) -> Vec<GridState<'grid>> {
        if history.len() <= 1 {
            return self
                .grid
                .all_pos()
                .map(|next_select| Self {
                    selecting: Some(next_select),
                    ..self.clone()
                })
                .collect();
        }
        let selecting = self.selecting.unwrap();
        let prev = history.last().unwrap();
        let prev_prev = &history[history.len() - 2];
        let swapping_states = self
            .grid
            .around_of(selecting)
            .into_iter()
            .filter(|&around| around != prev.selecting.unwrap())
            .map(|next_swap| {
                let mut new_field = self.field.clone();
                new_field.swap(selecting, next_swap);
                Self {
                    selecting: Some(next_swap),
                    field: new_field,
                    ..self.clone()
                }
            });
        let selecting_states = self
            .grid
            .all_pos()
            .filter(|&p| p != selecting)
            .map(|next_select| Self {
                selecting: Some(next_select),
                ..self.clone()
            });
        let moved_in_prev = prev
            .field
            .iter()
            .zip(prev_prev.field.iter())
            .any(|(a, b)| a != b);
        if moved_in_prev {
            swapping_states.chain(selecting_states).collect()
        } else {
            swapping_states.collect()
        }
    }

    fn is_goal(&self) -> bool {
        self.grid
            .all_pos()
            .map(|pos| (pos, self.field[pos]))
            .all(|(pos, cell)| pos == cell)
    }

    fn heuristic(&self) -> u64 {
        let h1: u64 = h1(self);
        let h2: u64 = h2(self);
        let cells_different_to_goal = self
            .grid
            .all_pos()
            .filter(|&pos| pos != self.field[pos])
            .count() as u64;
        h1 + h2 + cells_different_to_goal
    }

    fn cost_between(&self, next: &Self) -> u64 {
        (if (&self.field)
            .into_iter()
            .zip((&next.field).into_iter())
            .all(|(a, b)| a == b)
        {
            self.select_cost
        } else {
            self.swap_cost
        }) as u64
    }
}

/// 状態の履歴 Vec<GridState> を Vec<Operation> に変換する.
fn path_to_operations(path: Vec<GridState>) -> Vec<Operation> {
    let mut current_operation: Option<Operation> = None;
    let mut operations = vec![];
    let mut prev = &path[0];
    for state in &path[1..] {
        let is_swapped = (&prev.field)
            .into_iter()
            .zip(&state.field)
            .any(|(a, b)| a != b);
        if is_swapped {
            let movement = Movement::between_pos(prev.selecting.unwrap(), state.selecting.unwrap());
            current_operation.as_mut().unwrap().movements.push(movement);
        } else if let Some(op) = current_operation.replace(Operation {
            select: state.selecting.unwrap(),
            movements: vec![],
        }) {
            operations.push(op);
        }
        prev = state;
    }
    if let Some(op) = current_operation {
        operations.push(op);
    }
    operations
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { nodes, .. } = EdgesNodes::new(grid, movements);
    let (path, _total_cost) = ida_star(GridState {
        grid,
        field: nodes.clone(),
        selecting: None,
        swap_cost,
        select_cost,
    });
    path_to_operations(path)
}
