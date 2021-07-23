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
    graph::{node_index, IndexType, UnGraph},
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
    for pos in state.grid.all_pos() {
        if let Some(right) = state.grid.right_of(pos) {
            if state.field[pos].manhattan_distance(state.field[right]) == 1 {
                edges.push((state.field[pos], state.field[right]));
            }
        }
        if let Some(down) = state.grid.down_of(pos) {
            if state.field[pos].manhattan_distance(state.field[down]) == 1 {
                edges.push((state.field[pos], state.field[down]));
            }
        }
    }
    let mut g = UnGraph::<Pos, (), Pos>::from_edges(edges);
    for pos in state.grid.all_pos() {
        if let Some(weight) = g.node_weight_mut(node_index(state.field[pos].index())) {
            *weight = pos;
        }
    }
    let forest = kosaraju_scc(&g);
    forest
        .iter()
        .filter(|tree| {
            tree.iter()
                .map(|&idx| -> (Pos, Pos) { (idx.into(), *g.node_weight(idx).unwrap()) })
                .any(|(pos, cell)| cell != pos)
        })
        .count() as u64
}

impl<'grid> State<Vec<GridState<'grid>>, u64> for GridState<'grid> {
    fn next_states(&self) -> Vec<GridState<'grid>> {
        self.grid
            .all_pos()
            .map(|next_select| Self {
                selecting: next_select,
                ..self.clone()
            })
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
        (if self.selecting.manhattan_distance(next.selecting) == 1 {
            self.swap_cost
        } else {
            self.swap_cost + self.select_cost
        }) as u64
    }
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(
    grid: &Grid,
    movements: &[(Pos, Pos)],
    swap_cost: u16,
    select_cost: u16,
) -> Vec<Operation> {
    let EdgesNodes { nodes, .. } = EdgesNodes::new(grid, &movements);

    let moved_nodes: HashSet<Pos> = movements.iter().flat_map(|&(a, b)| [a, b]).collect();
    let initial_states = moved_nodes.into_iter().flat_map(|node| {
        grid.around_of(node).into_iter().map(|p| GridState {
            grid: &grid,
            field: nodes.clone(),
            selecting: p,
            swap_cost,
            select_cost,
        })
    });

    let mut path = initial_states
        .into_iter()
        .map(ida_star)
        .min_by(|a, b| a.1.cmp(&b.1))
        .unwrap()
        .0;

    let mut current_operation = Operation {
        select: path.pop().unwrap().selecting,
        movements: vec![],
    };
    let mut operations = vec![];
    for state in path {
        if current_operation.select.manhattan_distance(state.selecting) == 1 {
            current_operation.movements.push(Movement::between_pos(
                current_operation.select,
                state.selecting,
            ));
        } else {
            operations.push(current_operation);
            current_operation = Operation {
                select: state.selecting,
                movements: vec![],
            };
        }
    }
    operations
}
