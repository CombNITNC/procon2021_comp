use super::{average_distance, find_and_remove, find_with, gui::EdgePos, DiffEntry, ResolveHints};
use crate::{
    basis::Dir,
    fragment::{Edge, Fragment},
};

fn find_by_single_side<'a, B>(
    fragments: &[Fragment],
    reference_edge: &Edge,
    blacklist: B,
) -> DiffEntry
where
    B: Iterator<Item = &'a EdgePos> + Clone + 'a,
{
    find_with(fragments, move |fragment| {
        let blacklist = blacklist.clone();
        fragment
            .edges
            .iter()
            .filter(move |e| {
                !blacklist
                    .clone()
                    .any(|b| b.pos == fragment.pos && b.dir == e.dir)
            })
            .map(move |edge| DiffEntry {
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
    hints: &mut ResolveHints,
) -> (Vec<Fragment>, Vec<Fragment>) {
    let num_fragment = num_fragment as usize;

    let right_dir = left_dir.opposite();
    let (mut left, mut right) = (vec![], vec![]);
    let (mut left_fragment_ref, mut right_fragment_ref) = (root_ref, root_ref);

    while right.len() + left.len() + (1/* for root */) != num_fragment {
        if let Some(pairs) = hints.confirmed_pairs_of(EdgePos::new(left_fragment_ref.pos, left_dir))
        {
            if right.len() + left.len() + pairs.len() + 1 > num_fragment {
                println!("shaker_fill: couldn't apply confirmed_pairs because of size overrun");
            } else {
                println!("applying {} confirmed pairs", pairs.len());
                for (pos, rot) in pairs {
                    let mut fragment = find_and_remove(fragments, pos).unwrap();
                    fragment.rotate(rot);
                    left.push(fragment);
                }
                left_fragment_ref = left.last().unwrap();
            }
        }

        if let Some(pairs) =
            hints.confirmed_pairs_of(EdgePos::new(right_fragment_ref.pos, right_dir))
        {
            if right.len() + left.len() + pairs.len() + 1 > num_fragment {
                println!("shaker_fill: couldn't apply confirmed_pairs because of size overrun");
            } else {
                println!("applying {} confirmed pairs", pairs.len());
                for (pos, rot) in pairs {
                    let mut fragment = find_and_remove(fragments, pos).unwrap();
                    fragment.rotate(rot);
                    right.push(fragment);
                }
                right_fragment_ref = right.last().unwrap();
            }
        }

        if right.len() + left.len() + 1 == num_fragment {
            break;
        }

        let right_score = find_by_single_side(
            fragments,
            right_fragment_ref.edges.edge(right_dir),
            hints.blacklist_of(right_fragment_ref),
        );

        let left_score = find_by_single_side(
            fragments,
            left_fragment_ref.edges.edge(left_dir),
            hints.blacklist_of(left_fragment_ref),
        );

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
