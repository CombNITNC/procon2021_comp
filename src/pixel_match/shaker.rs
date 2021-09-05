use super::{average_distance, find_and_remove, find_with, DiffEntry};
use crate::{
    basis::Dir,
    fragment::{Edge, Fragment},
};

fn find_by_single_side(fragments: &[Fragment], reference_edge: &Edge) -> DiffEntry {
    find_with(fragments, move |fragment| {
        fragment.edges.iter().map(move |edge| DiffEntry {
            pos: fragment.pos,
            dir: edge.dir,
            score: average_distance(reference_edge.pixels.iter(), edge.pixels.iter().rev()),
        })
    })
}

/// root_ref から left_dir と left_dir.opposite() 方向に探索して、スコアが良い順に採用する。
pub(super) fn shaker_fill(
    num_fragment: u8,
    fragments: &mut Vec<Fragment>,
    left_dir: Dir,
    root_ref: &Fragment,
) -> (Vec<Fragment>, Vec<Fragment>) {
    let right_dir = left_dir.opposite();
    let (mut left, mut right) = (vec![], vec![]);
    let (mut left_fragment_ref, mut right_fragment_ref) = (root_ref, root_ref);

    while right.len() + left.len() + 1 != num_fragment as usize {
        let right_score = find_by_single_side(fragments, right_fragment_ref.edges.edge(right_dir));
        let left_score = find_by_single_side(fragments, left_fragment_ref.edges.edge(left_dir));

        if right_score.score < left_score.score {
            let mut fragment = find_and_remove(fragments, right_score.pos).unwrap();
            fragment.rotate(right_dir.calc_rot(right_score.dir));
            right.push(fragment);
            right_fragment_ref = right.last().unwrap();
        } else {
            let mut fragment = find_and_remove(fragments, left_score.pos).unwrap();
            fragment.rotate(left_dir.calc_rot(left_score.dir));
            left.push(fragment);
            left_fragment_ref = left.last().unwrap();
        }
    }

    (left, right)
}
