use crate::grid::{board::Board, Pos};

use super::route::{route_select_to_target, route_target_to_pos};

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct RowSolveEstimate {
    pub(super) moves: Vec<Pos>,
    pub(super) worst_route_size: usize,
    pub(super) worst_swap_pos: Pos,
}

pub(super) fn estimate_solve_row(mut board: Board, targets: &[Pos]) -> RowSolveEstimate {
    debug_assert_eq!(
        board
            .grid()
            .looping_manhattan_dist(targets[targets.len() - 2], *targets.last().unwrap()),
        1,
        "board: {:#?}, targets: {:?}",
        board,
        targets
    );

    let mut estimate = RowSolveEstimate::default();

    let mut line_proc = estimate_line_without_edge(board.clone(), &targets[..targets.len() - 2]);
    board.swap_many_to(&line_proc.moves);
    estimate.moves.append(&mut line_proc.moves);
    for &p in &targets[..targets.len() - 2] {
        board.lock(p);
    }
    if estimate.worst_route_size < line_proc.worst_route_size {
        estimate.worst_route_size = line_proc.worst_route_size;
        estimate.worst_swap_pos = line_proc.worst_swap_pos;
    }
    if (&targets[targets.len() - 2..])
        .iter()
        .any(|&p| p != board.forward(p))
    {
        let edge_rd_estimate = estimate_edge_then_right_down(
            &board,
            (targets[targets.len() - 2], targets[targets.len() - 1]),
        );
        let edge_ld_estimate = estimate_edge_then_left_down(
            &board,
            (targets[targets.len() - 2], targets[targets.len() - 1]),
        );
        let mut edge_estimate = if edge_rd_estimate.len() < edge_ld_estimate.len() {
            edge_rd_estimate
        } else {
            edge_ld_estimate
        };
        estimate.moves.append(&mut edge_estimate);
    }
    estimate.moves.dedup();
    estimate
}

fn estimate_line_without_edge(mut board: Board, targets: &[Pos]) -> RowSolveEstimate {
    let mut estimate = RowSolveEstimate::default();
    for &target in targets {
        let pos = board.reverse(target);
        let route = route_target_to_pos(&board, pos, target).expect("the route must be found");
        let mut route_size = 0;
        for win in route.windows(2) {
            let way = win[0];
            let next = win[1];
            board.lock(way);
            let mut route = route_select_to_target(&board, next);
            board.swap_many_to(&route);
            estimate.moves.append(&mut route);
            route_size += route.len();
            board.unlock(way);
            estimate.moves.push(way);
            board.swap_to(way);
        }
        if estimate.worst_route_size < route_size {
            estimate.worst_route_size = route_size;
            estimate.worst_swap_pos = target;
        }
        board.lock(target);
    }
    estimate.moves.dedup();
    estimate
}

/// ```text
/// ... 選 a
/// ... ** b
/// ```
/// この形に変形してから `Right` → `Down` して行を完成させる経路を見積もる
fn estimate_edge_then_right_down(board: &Board, (a, b): (Pos, Pos)) -> Vec<Pos> {
    let mut board = board.clone();
    let mut ret = vec![];

    let a_pos = board.reverse(a);
    let a_goal = board.grid().right_of(a);
    move_target_to_pos(&mut board, a_pos, a_goal, &mut ret);

    let b_pos = board.reverse(b);
    let b_goal = board.grid().down_of(b);
    move_target_to_pos(&mut board, b_pos, b_goal, &mut ret);

    let select = board.selected();
    let select_goal = a;
    move_target_to_pos(&mut board, select, select_goal, &mut ret);

    ret.push(b);
    ret.push(b_goal);
    ret
}

/// ```text
/// ... b 選
/// ... a **
/// ```
/// この形に変形してから `Left` → `Down` して行を完成させる経路を見積もる
fn estimate_edge_then_left_down(board: &Board, (a, b): (Pos, Pos)) -> Vec<Pos> {
    let mut board = board.clone();
    let mut ret = vec![];

    let a_pos = board.reverse(a);
    let a_goal = board.grid().down_of(a);
    move_target_to_pos(&mut board, a_pos, a_goal, &mut ret);

    let b_pos = board.reverse(b);
    let b_goal = board.grid().left_of(b);
    move_target_to_pos(&mut board, b_pos, b_goal, &mut ret);

    let select = board.selected();
    let select_goal = b;
    move_target_to_pos(&mut board, select, select_goal, &mut ret);

    ret.push(b);
    ret.push(b_goal);
    ret
}

fn move_target_to_pos(board: &mut Board, target: Pos, pos: Pos, ret: &mut Vec<Pos>) {
    let route = route_target_to_pos(board, target, pos).unwrap();

    for win in route.windows(2) {
        let way = win[0];
        let next = win[1];
        board.lock(way);

        let mut route = route_select_to_target(board, next);
        board.swap_many_to(&route);
        ret.append(&mut route);
        board.unlock(way);
        board.swap_to(way);
        ret.push(way);
    }
}
