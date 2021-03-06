use crate::{
    basis::Movement,
    grid::{
        board::{Board, BoardFinder},
        Pos,
    },
};

use super::route::{route_select_to_target, route_target_to_pos};

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct RowSolveEstimate {
    pub(super) moves: Vec<Pos>,
    pub(super) worst_route_size: usize,
    pub(super) worst_swap_pos: Pos,
}

pub(super) fn estimate_solve_row(
    mut board: Board,
    finder: &BoardFinder,
    targets: &[Pos],
) -> Option<RowSolveEstimate> {
    debug_assert_eq!(
        board.looping_manhattan_dist(targets[targets.len() - 2], *targets.last().unwrap()),
        1,
        "board: {:#?}, targets: {:?}",
        board,
        targets
    );

    let mut estimate = RowSolveEstimate::default();

    let without_corner = &targets[..targets.len() - 2];
    let mut line_proc = estimate_line_without_corner(board.clone(), without_corner)
        .expect("the route must be found");
    board.swap_many_to(&line_proc.moves);
    estimate.moves.append(&mut line_proc.moves);
    for &p in without_corner {
        debug_assert_eq!(p, board.forward(p), "{:#?}", board);
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
            finder,
            (targets[targets.len() - 2], targets[targets.len() - 1]),
        );
        let edge_ld_estimate = estimate_edge_then_left_down(
            &board,
            finder,
            (targets[targets.len() - 2], targets[targets.len() - 1]),
        );
        let mut edge_estimate = match (edge_rd_estimate, edge_ld_estimate) {
            (None, None) => return None,
            (None, Some(ld)) => ld,
            (Some(rd), None) => rd,
            (Some(rd), Some(ld)) => {
                if rd.len() < ld.len() {
                    rd
                } else {
                    ld
                }
            }
        };
        estimate.moves.append(&mut edge_estimate);
    }
    estimate.moves.dedup();
    Some(estimate)
}

fn estimate_line_without_corner(mut board: Board, targets: &[Pos]) -> Option<RowSolveEstimate> {
    let mut estimate = RowSolveEstimate::default();
    for &target in targets {
        let pos = board.reverse(target);
        if target == pos {
            board.lock(pos);
            continue;
        }
        let route = route_target_to_pos(&board, pos, target)?;
        let mut route_size = 0;
        for win in route.windows(2) {
            let way = win[0];
            let next = win[1];
            board.lock(way);
            let mut route = route_select_to_target(&board, next)?;
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
    Some(estimate)
}

/// ```text
/// ... ??? a
/// ... ** b
/// ```
/// ?????????????????????????????? `Right` ??? `Down` ????????????????????????????????????????????????
fn estimate_edge_then_right_down(
    board: &Board,
    finder: &BoardFinder,
    (a, b): (Pos, Pos),
) -> Option<Vec<Pos>> {
    let mut board = board.clone();
    let mut ret = vec![];

    let a_pos = board.reverse(a);
    let a_goal = finder.move_pos_to(a, Movement::Right);
    move_target_to_pos(&mut board, a_pos, a_goal, &mut ret)?;
    board.lock(a_goal);

    let b_pos = board.reverse(b);
    let b_goal = finder.move_pos_to(b, Movement::Down);
    move_target_to_pos(&mut board, b_pos, b_goal, &mut ret)?;
    board.lock(b_goal);

    let select_goal = a;
    move_select_to_target(&mut board, select_goal, &mut ret)?;

    ret.push(b);
    ret.push(b_goal);
    Some(ret)
}

/// ```text
/// ... b ???
/// ... a **
/// ```
/// ?????????????????????????????? `Left` ??? `Down` ????????????????????????????????????????????????
fn estimate_edge_then_left_down(
    board: &Board,
    finder: &BoardFinder,
    (a, b): (Pos, Pos),
) -> Option<Vec<Pos>> {
    let mut board = board.clone();
    let mut ret = vec![];

    let b_pos = board.reverse(b);
    let b_goal = finder.move_pos_to(b, Movement::Left);
    move_target_to_pos(&mut board, b_pos, b_goal, &mut ret)?;
    board.lock(b_goal);

    let a_pos = board.reverse(a);
    let a_goal = finder.move_pos_to(a, Movement::Down);
    move_target_to_pos(&mut board, a_pos, a_goal, &mut ret)?;
    board.lock(a_goal);

    let select_goal = b;
    move_select_to_target(&mut board, select_goal, &mut ret)?;

    ret.push(a);
    ret.push(a_goal);
    Some(ret)
}

#[must_use]
fn move_target_to_pos(board: &mut Board, target: Pos, pos: Pos, ret: &mut Vec<Pos>) -> Option<()> {
    let route = route_target_to_pos(board, target, pos)?;

    for win in route.windows(2) {
        let way = win[0];
        let next = win[1];
        board.lock(way);

        let mut route = route_select_to_target(board, next)?;
        board.swap_many_to(&route);
        ret.append(&mut route);
        board.unlock(way);
        board.swap_to(way);
        ret.push(way);
    }
    Some(())
}

#[must_use]
fn move_select_to_target(board: &mut Board, target: Pos, ret: &mut Vec<Pos>) -> Option<()> {
    let mut route = route_select_to_target(board, target)?;
    board.swap_many_to(&route);
    ret.append(&mut route);
    Some(())
}
