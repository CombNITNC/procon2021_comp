use self::{
    edges_nodes::EdgesNodes,
    ida_star::{ida_star, State},
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
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
struct GridState<'grid> {
    grid: &'grid Grid,
    field: VecOnGrid<'grid, Pos>,
    selecting: Pos,
    swap_cost: u16,
    select_cost: u16,
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
            eprintln!("--- {:?}", tree);
            tree.iter().any(|&idx| {
                eprintln!("{:?} {:?}", <Pos as IndexType>::new(idx.index()), g[idx]);
                <Pos as IndexType>::new(idx.index()) != g[idx]
            })
        })
        .count() as u64
}

impl<'grid> State<u64> for GridState<'grid> {
    type NextStates = Vec<GridState<'grid>>;
    fn next_states(&self) -> Vec<GridState<'grid>> {
        self.grid
            .around_of(self.selecting)
            .into_iter()
            .map(|next_swap| {
                let mut new_field = self.field.clone();
                new_field.swap(self.selecting, next_swap);
                Self {
                    selecting: next_swap,
                    field: new_field,
                    ..self.clone()
                }
            })
            .chain(
                self.grid
                    .all_pos()
                    .filter(|&p| p != self.selecting)
                    .map(|next_select| Self {
                        selecting: next_select,
                        ..self.clone()
                    }),
            )
            .collect()
    }

    fn is_goal(&self) -> bool {
        self.grid
            .all_pos()
            .map(|pos| (pos, self.field[pos]))
            .all(|(pos, cell)| pos == cell)
    }

    fn heuristic(&self) -> u64 {
        let h1: u64 = h1(self);
        let cells_different_to_goal = self
            .grid
            .all_pos()
            .filter(|&pos| pos != self.field[pos])
            .count() as u64;
        h1 + cells_different_to_goal
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
    for state in path {
        let is_adj = |op: &Operation| op.select.manhattan_distance(state.selecting) == 1;
        if current_operation.as_ref().map_or(false, is_adj) {
            let movement =
                Movement::between_pos(current_operation.as_ref().unwrap().select, state.selecting);
            current_operation.as_mut().unwrap().movements.push(movement);
        } else if let Some(op) = current_operation.replace(Operation {
            select: state.selecting,
            movements: vec![],
        }) {
            operations.push(op);
        }
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

    let moved_nodes: HashSet<Pos> = movements.iter().flat_map(|&(a, b)| [a, b]).collect();
    let initial_states = moved_nodes.into_iter().flat_map(|node| {
        grid.around_of(node).into_iter().map(|p| GridState {
            grid,
            field: nodes.clone(),
            selecting: p,
            swap_cost,
            select_cost,
        })
    });

    initial_states
        .into_iter()
        .map(ida_star)
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(path, cost)| (path_to_operations(path), cost))
        .unwrap()
        .0
}
