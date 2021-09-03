use crate::{
    basis::{Color, Dir},
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

use super::{
    average_distance, find_and_remove, find_with, get_edge_pixels, rotate_count, DiffEntry,
};

fn find_by_double_side<'a, I>(fragments: &'a [Fragment], reference_iter: I) -> DiffEntry
where
    I: Iterator<Item = &'a Color> + Clone + 'a,
{
    find_with(fragments, move |fragment| {
        let reference_iter = reference_iter.clone();

        std::array::IntoIter::new([
            [Dir::North, Dir::East],
            [Dir::East, Dir::South],
            [Dir::South, Dir::West],
            [Dir::West, Dir::North],
        ])
        .map(move |[dir_a, dir_b]| (fragment.edges.edge(dir_a), fragment.edges.edge(dir_b)))
        .map(move |(edge_a, edge_b)| DiffEntry {
            pos: fragment.pos,
            dir: edge_a.dir,
            score: average_distance(
                reference_iter.clone(),
                edge_a.pixels.iter().chain(edge_b.pixels.iter()),
            ),
        })
    })
}

pub(super) fn fill_by_double_side(
    fragments: &mut Vec<Fragment>,
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
    pos: Pos,
    (ref1_pos, ref1_dir): (Pos, Dir),
    (ref2_pos, ref2_dir): (Pos, Dir),
) {
    let reference_iter = get_edge_pixels(fragment_grid, ref1_pos, ref1_dir)
        .unwrap()
        .iter()
        .rev()
        .chain(
            get_edge_pixels(fragment_grid, ref2_pos, ref2_dir)
                .unwrap()
                .iter()
                .rev(),
        );

    let min = find_by_double_side(fragments, reference_iter);

    let mut fragment = find_and_remove(fragments, min.pos).unwrap();
    fragment.rotate(rotate_count(ref1_dir, min.dir));

    *fragment_grid.get_mut(pos).unwrap() = Some(fragment);
}
