use crate::basis::{Color, Dir};
use crate::fragment::Fragment;
use crate::grid::{Grid, Pos, VecOnGrid};

mod double_side;
mod shaker;

use double_side::fill_by_double_side;
use shaker::shaker_fill;

pub(crate) fn resolve(mut fragments: Vec<Fragment>, grid: Grid) -> VecOnGrid<Fragment> {
    let mut fragment_grid = VecOnGrid::<Option<Fragment>>::with_default(grid);

    // 必ず向きの正しい左上の断片を取得
    let root = find_and_remove(&mut fragments, grid.pos(0, 0)).unwrap();

    // そこから上下左右に伸ばす形で探索
    let (up, down) = shaker_fill(grid.height(), &mut fragments, Dir::North, &root);
    let (left, right) = shaker_fill(grid.width(), &mut fragments, Dir::West, &root);

    // root から上下左右に何個断片が有るかわかったので、rootのあるべき座標が分かる
    let root_pos = grid.pos(left.len() as _, up.len() as _);

    place_shaker_result_on_grid(&mut fragment_grid, root, [up, down, left, right]);

    // r = root, x = すでにわかった断片 としたとき、今 fragment_grid は以下のような状態になっている。
    // ------------
    //    2    x  1
    //         x
    // xxxxxxxxrxxx
    //    3    x  4
    // ------------
    // この 1,2,3,4 で示したスペースをそれぞれ root に近い断片から埋めていく。
    // 2辺わかった状態で探索できるため、精度向上が期待できる。

    // 1
    for x in root_pos.x() + 1..grid.width() {
        for y in (0..root_pos.y()).rev() {
            fill_by_double_side(
                &mut fragments,
                &mut fragment_grid,
                grid.pos(x, y),
                (grid.pos(x, y + 1), Dir::North),
                (grid.pos(x - 1, y), Dir::East),
            );
        }
    }

    // 2
    for x in (0..root_pos.x()).rev() {
        for y in (0..root_pos.y()).rev() {
            fill_by_double_side(
                &mut fragments,
                &mut fragment_grid,
                grid.pos(x, y),
                (grid.pos(x + 1, y), Dir::West),
                (grid.pos(x, y + 1), Dir::North),
            );
        }
    }

    // 3
    for x in (0..root_pos.x()).rev() {
        for y in root_pos.y() + 1..grid.height() {
            fill_by_double_side(
                &mut fragments,
                &mut fragment_grid,
                grid.pos(x, y),
                (grid.pos(x, y - 1), Dir::South),
                (grid.pos(x + 1, y), Dir::West),
            );
        }
    }

    // 4
    for x in root_pos.x() + 1..grid.width() {
        for y in root_pos.y() + 1..grid.height() {
            fill_by_double_side(
                &mut fragments,
                &mut fragment_grid,
                grid.pos(x, y),
                (grid.pos(x - 1, y), Dir::East),
                (grid.pos(x, y - 1), Dir::South),
            );
        }
    }

    VecOnGrid::from_vec(
        grid,
        fragment_grid
            .into_iter()
            .map(|x| x.expect("there were not filled fragment on grid"))
            .collect(),
    )
    .unwrap()
}

#[inline]
fn place_shaker_result_on_grid(
    fragment_grid: &mut VecOnGrid<Option<Fragment>>,
    root: Fragment,
    [up, down, left, right]: [Vec<Fragment>; 4],
) {
    let root_pos = fragment_grid.grid.pos(left.len() as _, up.len() as _);

    let mut place = |x, y, cell| {
        let pos = fragment_grid.grid.pos(x as _, y as _);
        fragment_grid[pos] = Some(cell)
    };

    place(left.len() as u8, up.len() as u8, root);

    // North
    for (i, v) in up.into_iter().enumerate() {
        place(root_pos.x(), root_pos.y() - 1 - i as u8, v);
    }

    // South
    for (i, v) in down.into_iter().enumerate() {
        place(root_pos.x(), root_pos.y() + 1 + i as u8, v);
    }

    // West
    for (i, v) in left.into_iter().enumerate() {
        place(root_pos.x() - 1 - i as u8, root_pos.y(), v);
    }

    // East
    for (i, v) in right.into_iter().enumerate() {
        place(root_pos.x() + 1 + i as u8, root_pos.y(), v);
    }
}

#[derive(Debug)]
struct DiffEntry {
    pos: Pos,
    dir: Dir,
    score: f64,
}

/// f から返される DiffEntry たちから最も最適なものを返す
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

/// reference と challenge 間の色距離の平均を求める
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

/// vec から pos を持つ Fragment の所有権を取得する
#[inline]
fn find_and_remove(vec: &mut Vec<Fragment>, pos: Pos) -> Option<Fragment> {
    Some(vec.remove(vec.iter().position(|x| x.pos == pos)?))
}
