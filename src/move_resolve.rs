use self::edges_nodes::EdgesNodes;
use crate::{
    basis::Operation,
    grid::{Grid, Pos},
};

mod edges_nodes;
mod ida_star;
#[cfg(test)]
mod tests;

fn heuristic() -> u64 {
    // h1 = 隣接マスどうしのマンハッタン距離が 1 かつ全頂点がゴール位置に無い集合の数
    // h2 = ゴール位置と異なる位置のマスの数
    let h1: u64 = todo!();
    let h2: u64 = todo!();
    h1 + h2
}

/// 完成形から `movements` のとおりに移動されているとき, それを解消する移動手順を求める.
pub(crate) fn resolve(grid: &Grid, movements: &[(Pos, Pos)]) -> Vec<Operation> {
    let edges_nodes = EdgesNodes::new(grid, &movements);

    todo!()
}
