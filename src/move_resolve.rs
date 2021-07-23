use self::{edges_nodes::EdgesNodes, ida_star::State};
use crate::{
    basis::Operation,
    grid::{Grid, Pos, VecOnGrid},
};
use petgraph::{
    algo::kosaraju_scc,
    graph::{node_index, IndexType, UnGraph},
};

mod edges_nodes;
mod ida_star;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
struct GridState {
    grid: Grid,
    field: VecOnGrid<'static, Pos>,
    selecting: Pos,
    swap_cost: u64,
    select_cost: u64,
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

impl State<Vec<GridState>, u64> for GridState {
    fn next_states(&self) -> Vec<GridState> {
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
        if self.selecting.manhattan_distance(next.selecting) == 1 {
            return self.swap_cost;
        }
        self.swap_cost + self.select_cost
    }
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(grid: &Grid, movements: &[(Pos, Pos)]) -> Vec<Operation> {
    let edges_nodes = EdgesNodes::new(grid, &movements);

    todo!()
}
