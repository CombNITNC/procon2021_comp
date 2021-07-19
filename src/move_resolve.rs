use self::edges_nodes::EdgesNodes;
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
struct State {
    grid: Grid,
    field: VecOnGrid<'static, Pos>,
    selecting: Pos,
}

fn h1(state: &State) -> u64 {
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

fn heuristic(state: &State) -> u64 {
    // h1 = 隣接マスどうしのマンハッタン距離が 1 かつ全頂点がゴール位置に無い集合の数
    // h2 = ゴール位置と異なる位置のマスの数
    let h1: u64 = h1(state);
    let h2: u64 = state
        .grid
        .all_pos()
        .filter(|&pos| pos != state.field[pos])
        .count() as u64;
    h1 + h2
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(grid: &Grid, movements: &[(Pos, Pos)]) -> Vec<Operation> {
    let edges_nodes = EdgesNodes::new(grid, &movements);

    todo!()
}
