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

#[allow(clippy::type_complexity)]
pub(super) fn shaker_fill(
    num_fragment: u8,
    fragments: &mut Vec<Fragment>,
    (mut left_dir, mut right_dir): (Dir, Dir),
    (mut left_fragment_ref, mut right_fragment_ref): (&Fragment, &Fragment),
) -> (Vec<(Fragment, DiffEntry)>, Vec<(Fragment, DiffEntry)>) {
    let (mut left, mut right) = (vec![], vec![]);

    while right.len() + left.len() + 1 != num_fragment as usize {
        let right_score = find_by_single_side(fragments, right_fragment_ref.edges.edge(right_dir));
        let left_score = find_by_single_side(fragments, left_fragment_ref.edges.edge(left_dir));

        if right_score.score < left_score.score {
            let fragment = find_and_remove(fragments, right_score.pos).unwrap();
            right_dir = right_score.dir.opposite();
            right.push((fragment, right_score));
            right_fragment_ref = &right.last().unwrap().0;
        } else {
            let fragment = find_and_remove(fragments, left_score.pos).unwrap();
            left_dir = left_score.dir.opposite();
            left.push((fragment, left_score));
            left_fragment_ref = &left.last().unwrap().0;
        }
    }

    (left, right)
}
