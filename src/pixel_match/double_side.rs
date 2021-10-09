use crate::{
    basis::{Color, Dir},
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

use super::{average_distance, find_and_remove, find_with, gui::EdgePos, DiffEntry, ResolveHints};

fn get_edge_pixels(grid: &VecOnGrid<Option<Fragment>>, pos: Pos, dir: Dir) -> Option<&Vec<Color>> {
    Some(&grid[pos].as_ref()?.edges.edge(dir).pixels)
}

fn find_by_double_side<'a, I, B>(
    fragments: &'a [Fragment],
    reference_iter: I,
    (blacklist, blacklist_ref_index): (B, usize),
) -> DiffEntry
where
    I: Iterator<Item = &'a Color> + Clone + 'a,
    B: Iterator<Item = &'a EdgePos> + Clone + 'a,
{
    find_with(fragments, move |fragment| {
        let reference_iter = reference_iter.clone();
        let blacklist = blacklist.clone();

        IntoIterator::into_iter([
            [Dir::North, Dir::East],
            [Dir::East, Dir::South],
            [Dir::South, Dir::West],
            [Dir::West, Dir::North],
        ])
        .filter(move |a| {
            !blacklist
                .clone()
                .any(|x| x.pos == fragment.pos && x.dir == a[blacklist_ref_index])
        })
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

fn fill_by_double_side_inner(
    fragments: &mut Vec<Fragment>,
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
    hints: &ResolveHints,
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

    let (blacklist_pos, index) = match (ref1_dir, ref2_dir) {
        (Dir::North | Dir::South, _) => (ref1_pos, 0),
        (_, Dir::North | Dir::South) => (ref2_pos, 1),
        _ => unreachable!("either ref1 or ref2 should refer Y-axis"),
    };

    let blacklist_pos = fragment_grid[blacklist_pos].as_ref().unwrap().pos;
    let blacklist = hints.blacklist_of(blacklist_pos);

    let min = find_by_double_side(fragments, reference_iter, (blacklist, index));

    let mut fragment = find_and_remove(fragments, min.pos).unwrap();
    fragment.rotate(ref1_dir.calc_rot(min.dir));

    fragment_grid[pos] = Some(fragment);
}

/// 2辺から最も合う断片を探して fragment_grid に入れる
pub(super) fn fill_by_double_side(
    root_pos: Pos,
    hints: &mut ResolveHints,
    fragments: &mut Vec<Fragment>,
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
) {
    let grid = fragment_grid.grid;

    // ------------
    //    2    x  1
    //         x
    // xxxxxxxxrxxx
    //    3    x  4
    // ------------

    // 1
    for x in root_pos.x() + 1..grid.width() {
        for y in (0..root_pos.y()).rev() {
            fill_by_double_side_inner(
                fragments,
                fragment_grid,
                hints,
                grid.pos(x, y),
                (grid.pos(x, y + 1), Dir::North),
                (grid.pos(x - 1, y), Dir::East),
            );
        }
    }

    // 2
    for x in (0..root_pos.x()).rev() {
        for y in (0..root_pos.y()).rev() {
            fill_by_double_side_inner(
                fragments,
                fragment_grid,
                hints,
                grid.pos(x, y),
                (grid.pos(x + 1, y), Dir::West),
                (grid.pos(x, y + 1), Dir::North),
            );
        }
    }

    // 3
    for x in (0..root_pos.x()).rev() {
        for y in root_pos.y() + 1..grid.height() {
            fill_by_double_side_inner(
                fragments,
                fragment_grid,
                hints,
                grid.pos(x, y),
                (grid.pos(x, y - 1), Dir::South),
                (grid.pos(x + 1, y), Dir::West),
            );
        }
    }

    // 4
    for x in root_pos.x() + 1..grid.width() {
        for y in root_pos.y() + 1..grid.height() {
            fill_by_double_side_inner(
                fragments,
                fragment_grid,
                hints,
                grid.pos(x, y),
                (grid.pos(x - 1, y), Dir::East),
                (grid.pos(x, y - 1), Dir::South),
            );
        }
    }
}
