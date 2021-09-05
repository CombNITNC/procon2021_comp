use super::{
    route::{route_select_to_target, route_target_to_goal},
    Board,
};
use crate::grid::Pos;

#[derive(Debug, Default)]
pub(super) struct RowSolveEstimate {
    pub(super) moves: Vec<Pos>,
    pub(super) worst_route_size: usize,
    pub(super) worst_swap_pos: Pos,
}

pub(super) fn estimate_solve_row(mut board: Board, targets: &[Pos]) -> RowSolveEstimate {
    let mut estimate = RowSolveEstimate::default();

    let line_proc = estimate_line_without_edge(board.clone(), targets);
    for mov in line_proc.moves {
        estimate.moves.push(mov);
        board.swap_to(mov);
    }
    for &p in &targets[..targets.len() - 2] {
        board.lock(p);
    }
    if estimate.worst_route_size < line_proc.worst_route_size {
        estimate.worst_route_size = line_proc.worst_route_size;
        estimate.worst_swap_pos = line_proc.worst_swap_pos;
    }

    let edge_rd_estimate = estimate_edge_then_right_down(&board, targets);
    let edge_ld_estimate = estimate_edge_then_left_down(&board, targets);
    let mut edge_estimate = if edge_rd_estimate.len() < edge_ld_estimate.len() {
        edge_rd_estimate
    } else {
        edge_ld_estimate
    };
    estimate.moves.append(&mut edge_estimate);
    estimate
}

fn estimate_line_without_edge(mut board: Board, targets: &[Pos]) -> RowSolveEstimate {
    let mut estimate = RowSolveEstimate::default();
    for &target in &targets[..targets.len() - 2] {
        let mut pos = board.field[target];
        let route = route_target_to_goal(&board, target, board.grid().all_pos())
            .expect("the route must be found");
        let mut route_size = 0;
        for way in route {
            board.lock(pos);
            let mut route = route_select_to_target(&board, way);
            for &way in &route {
                board.swap_to(way);
            }
            estimate.moves.append(&mut route);
            route_size += route.len();
            board.unlock(pos);
            estimate.moves.push(pos);
            board.swap_to(pos);
            pos = way;
        }
        if estimate.worst_route_size < route_size {
            estimate.worst_route_size = route_size;
            estimate.worst_swap_pos = target;
        }
        board.lock(target);
    }
    estimate
}

/// ```text
/// ... 選 a
/// ... ** b
/// ```
/// この形に変形してから `Right` → `Down` して行を完成させる経路を見積もる
fn estimate_edge_then_right_down(board: &Board, targets: &[Pos]) -> Vec<Pos> {
    let a = targets[targets.len() - 1];
    let b = targets.last().unwrap();
    todo!()
}

/// ```text
/// ... b 選
/// ... a **
/// ```
/// この形に変形してから `Left` → `Down` して行を完成させる経路を見積もる
fn estimate_edge_then_left_down(board: &Board, targets: &[Pos]) -> Vec<Pos> {
    let a = targets[targets.len() - 1];
    let b = targets.last().unwrap();
    todo!()
}
