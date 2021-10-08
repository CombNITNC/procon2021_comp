use std::sync::mpsc;

use crate::basis::{Color, Dir, Rot};
use crate::fragment::Fragment;
use crate::grid::{Grid, Pos, VecOnGrid};
use crate::pixel_match::gui::{EdgePos, GuiRequest, GuiResponse};

mod double_side;
mod gui;
mod shaker;

use double_side::fill_by_double_side;
use shaker::shaker_fill;

use self::gui::RecalculateArtifact;

pub(crate) fn resolve(fragments: Vec<Fragment>, grid: Grid) -> VecOnGrid<Fragment> {
    let (gtx, rx) = mpsc::channel();
    let (tx, grx) = mpsc::channel();

    let solver_thread = std::thread::Builder::new()
        .name("pixel matcher".into())
        .spawn(move || {
            let (recovered_image, root_pos) =
                solve(fragments.clone(), grid, ResolveHints::default());

            let mut result = recovered_image.clone();

            tx.send(GuiResponse::Recalculated(RecalculateArtifact {
                recovered_image,
                root_pos,
            }))
            .unwrap();

            loop {
                match rx.recv() {
                    Ok(GuiRequest::Recalculate(hint)) => {
                        println!("recalculating. blacklists: {{");
                        for h in &hint.blacklist {
                            println!("    {:?}", h);
                        }
                        println!("}}");
                        println!("whitelists: {{");
                        for h in &hint.confirmed_pairs {
                            println!("    {:?}", h);
                        }
                        println!("}}");

                        let (recovered_image, root_pos) = solve(fragments.clone(), grid, hint);

                        result = recovered_image.clone();

                        tx.send(GuiResponse::Recalculated(RecalculateArtifact {
                            recovered_image,
                            root_pos,
                        }))
                        .unwrap();
                    }

                    Ok(GuiRequest::Quit) => break,

                    Err(_) => {
                        eprintln!("main thread channel unexpectedly closed. maybe it has panicked");
                        break;
                    }
                }
            }
            result
        })
        .expect("failed to launch pixel matcher thread");

    gui::begin(gui::GuiContext { tx: gtx, rx: grx });

    let result = solver_thread
        .join()
        .unwrap_or_else(|e| std::panic::resume_unwind(e));

    VecOnGrid::from_vec(
        grid,
        result
            .into_iter()
            .map(|x| x.expect("there were not filled fragment on grid"))
            .collect(),
    )
    .unwrap()
}

// returns: (recovered_image, root_pos)
fn solve(
    mut fragments: Vec<Fragment>,
    grid: Grid,
    mut hints: ResolveHints,
) -> (VecOnGrid<Option<Fragment>>, Pos) {
    let mut fragment_grid = VecOnGrid::<Option<Fragment>>::with_default(grid);

    // 必ず向きの正しい左上の断片を取得
    let root = find_and_remove(&mut fragments, grid.pos(0, 0)).unwrap();

    // そこから上下左右に伸ばす形で探索
    let (up, down) = shaker_fill(grid.height(), &mut fragments, Dir::North, &root, &mut hints);
    let (left, right) = shaker_fill(grid.width(), &mut fragments, Dir::West, &root, &mut hints);

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

    (fragment_grid, root_pos)
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

#[derive(Debug, Default, Clone)]
struct ResolveHints {
    blacklist: Vec<(Pos, EdgePos)>,
    confirmed_pairs: Vec<(EdgePos, Vec<(Pos, Rot)>)>,
}

impl ResolveHints {
    fn blacklist_of<'a>(
        &'a self,
        fragment: &'a Fragment,
    ) -> impl Iterator<Item = &'a EdgePos> + Clone + 'a {
        self.blacklist
            .iter()
            .filter(move |&(x, _)| *x == fragment.pos)
            .map(|(_, x)| x)
    }

    fn confirmed_pairs_of(&mut self, pos: EdgePos) -> Option<Vec<(Pos, Rot)>> {
        let (index, _) = self
            .confirmed_pairs
            .iter()
            .enumerate()
            .filter(|(_, (p, _))| *p == pos)
            .max_by_key(|(_, (_, v))| v.len())?;

        Some(self.confirmed_pairs.remove(index).1)
    }
}
