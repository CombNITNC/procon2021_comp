use crate::{
    basis::{Color, Dir},
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

use super::{average_distance, find_and_remove, find_with, DiffEntry};

fn get_edge_pixels(grid: &VecOnGrid<Option<Fragment>>, pos: Pos, dir: Dir) -> Option<&Vec<Color>> {
    Some(&grid[pos].as_ref()?.edges.edge(dir).pixels)
}

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

/// 2辺から最も合う断片を探して fragment_grid に入れる
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
    fragment.rotate(ref1_dir.calc_rot(min.dir));

    fragment_grid[pos] = Some(fragment);
}
