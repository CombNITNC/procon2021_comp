use crate::basis::{Color, Dir, Rot};
use crate::fragment::Fragment;
use crate::grid::{Pos, VecOnGrid};

mod double_side;
mod shaker;

#[derive(Debug)]
struct DiffEntry {
    pos: Pos,
    dir: Dir,
    score: f64,
}

#[inline]
fn find_with<'a, F, I>(fragments: &'a [Fragment], f: F) -> DiffEntry
where
    F: FnMut(&'a Fragment) -> I,
    I: Iterator<Item = DiffEntry> + 'a,
{
    fragments
        .iter()
        .flat_map(f)
        .min_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
        .expect("there were no fragments")
}

#[inline]
fn average_distance<'a>(
    reference: impl Iterator<Item = &'a Color>,
    challenge: impl Iterator<Item = &'a Color>,
) -> f64 {
    let mut count = 0;
    let mut sum_of_distance: f64 = 0.;

    for (r, c) in reference.zip(challenge) {
        let distance = r.euclidean_distance(*c);
        sum_of_distance += distance;
        count += 1;
    }

    sum_of_distance / count as f64
}

fn find_and_remove(vec: &mut Vec<Fragment>, pos: Pos) -> Option<Fragment> {
    Some(vec.remove(vec.iter().position(|x| x.pos == pos)?))
}

fn get_edge_pixels<'a>(
    vec: &'a VecOnGrid<Option<Fragment>>,
    pos: Pos,
    dir: Dir,
) -> Option<&'a Vec<Color>> {
    Some(&vec.get(pos)?.as_ref()?.edges.edge(dir).pixels)
}

fn place_initial_result_on_grid(
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
    root: Fragment,
    [up, down, left, right]: [Vec<(Fragment, DiffEntry)>; 4],
) {
    let root_pos = Pos::new(left.len() as _, up.len() as _);

    *fragment_grid.get_mut(root_pos).unwrap() = Some(root);

    let mut place = |dir, data: Vec<(Fragment, DiffEntry)>| {
        place_for_direction(
            fragment_grid,
            root_pos,
            dir,
            data.into_iter().map(|(mut fragment, diffentry)| {
                fragment.rotate(rotate_count(dir, diffentry.dir));
                fragment
            }),
        )
    };

    place(Dir::North, up);
    place(Dir::South, down);
    place(Dir::West, left);
    place(Dir::East, right);
}

fn place_for_direction<T>(
    grid: &mut VecOnGrid<Option<T>>,
    from: Pos,
    dir: Dir,
    data: impl Iterator<Item = T>,
) {
    let mut place = |x, y, cell| *grid.get_mut(Pos::new(x as _, y as _)).unwrap() = Some(cell);

    match dir {
        Dir::North => {
            for (i, v) in data.enumerate() {
                place(from.x(), from.y() - 1 - i as u8, v);
            }
        }

        Dir::South => {
            for (i, v) in data.enumerate() {
                place(from.x(), from.y() + 1 + i as u8, v);
            }
        }

        Dir::West => {
            for (i, v) in data.enumerate() {
                place(from.x() - 1 - i as u8, from.y(), v);
            }
        }

        Dir::East => {
            for (i, v) in data.enumerate() {
                place(from.x() + 1 + i as u8, from.y(), v);
            }
        }
    }
}

#[inline]
fn rotate_count(from: Dir, to: Dir) -> Rot {
    use Rot::*;
    match from {
        Dir::North => match to {
            Dir::South => R0,
            Dir::East => R90,
            Dir::North => R180,
            Dir::West => R270,
        },
        Dir::East => match to {
            Dir::West => R0,
            Dir::South => R90,
            Dir::East => R180,
            Dir::North => R270,
        },
        Dir::South => match to {
            Dir::North => R0,
            Dir::West => R90,
            Dir::South => R180,
            Dir::East => R270,
        },
        Dir::West => match to {
            Dir::East => R0,
            Dir::North => R90,
            Dir::West => R180,
            Dir::South => R270,
        },
    }
}
