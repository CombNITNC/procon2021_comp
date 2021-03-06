use std::collections::HashMap;
use std::sync::mpsc;

use crate::basis::{Color, Dir, Rot};
use crate::fragment::Fragment;
use crate::grid::{Grid, Pos, VecOnGrid};
use crate::pixel_match::gui::{EdgePos, GuiRequest, GuiResponse};

mod double_side;
mod gui;
mod shaker;

use self::gui::RecalculateArtifact;

pub fn resolve(fragments: Vec<Fragment>, grid: Grid) -> VecOnGrid<Fragment> {
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
                        println!(
                            "recalculating. blocklists: {} entries",
                            hint.blocklist.len()
                        );
                        println!("whitelists: {} entries", hint.locked_pairs.len());

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

    // ????????????????????????????????????????????????
    let root = fragments
        .find_and_remove(|x| x.pos == grid.pos(0, 0))
        .unwrap();

    // ????????????????????????????????????????????????
    let (up, down) =
        shaker::shaker_fill(grid.height(), &mut fragments, Dir::North, &root, &mut hints);
    let (left, right) =
        shaker::shaker_fill(grid.width(), &mut fragments, Dir::West, &root, &mut hints);

    // root ??????????????????????????????????????????????????????????????????root?????????????????????????????????
    let root_pos = grid.pos(left.len() as _, up.len() as _);

    place_shaker_result_on_grid(&mut fragment_grid, root, [up, down, left, right]);

    // r = root, x = ??????????????????????????? ????????????????????? fragment_grid ????????????????????????????????????????????????
    // ------------
    //    2    x  1
    //         x
    // xxxxxxxxrxxx
    //    3    x  4
    // ------------
    // ?????? 1,2,3,4 ??????????????????????????????????????? root ???????????????????????????????????????
    // 2?????????????????????????????????????????????????????????????????????????????????

    double_side::fill_by_double_side(root_pos, &mut hints, &mut fragments, &mut fragment_grid);

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

/// f ?????????????????? DiffEntry ??????????????????????????????????????????
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

/// reference ??? challenge ????????????????????????????????????
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

#[derive(Debug, Default, Clone)]
struct ResolveHints {
    blocklist: HashMap<Pos, Vec<EdgePos>>,
    locked_pairs: HashMap<EdgePos, LockedPairs>,
}

#[derive(Debug, Clone)]
struct LockedPairs {
    tail: Vec<(Pos, Rot)>,
    continue_after_apply: bool,
}

impl LockedPairs {
    fn new(tail: Vec<(Pos, Rot)>) -> Self {
        Self {
            tail,
            continue_after_apply: true,
        }
    }

    fn stop_after_apply(&mut self) {
        self.continue_after_apply = false;
    }
}

impl ResolveHints {
    fn push_blocklist(&mut self, pos: Pos, against: EdgePos) {
        self.blocklist.entry(pos).or_default().push(against);
    }

    fn push_locked_pair(&mut self, pos: EdgePos, pairs: LockedPairs) {
        self.locked_pairs.insert(pos, pairs);
    }

    fn remove_blocklist(&mut self, pos: Pos, against: EdgePos) {
        self.blocklist
            .entry(pos)
            .or_default()
            .find_and_remove(|&x| x == against);
    }

    fn remove_locked_pair(&mut self, pos: EdgePos) {
        self.locked_pairs.remove(&pos);
    }

    fn lock_pair_as_end(&mut self, pos: EdgePos) {
        if let Some(r) = self.locked_pairs.get_mut(&pos) {
            r.continue_after_apply = false;
        }
    }

    fn take_blacklist(&self, pos: Pos) -> impl Iterator<Item = &EdgePos> + Clone {
        self.blocklist.get(&pos).into_iter().flatten()
    }

    fn take_locked_pairs(&mut self, pos: EdgePos) -> Option<LockedPairs> {
        self.locked_pairs.remove(&pos)
    }
}

trait FindAndRemove<T> {
    fn find_and_remove(&mut self, pred: impl FnMut(&T) -> bool) -> Option<T>;
}

impl<T> FindAndRemove<T> for Vec<T> {
    fn find_and_remove(&mut self, pred: impl FnMut(&T) -> bool) -> Option<T> {
        Some(self.remove(self.iter().position(pred)?))
    }
}
