use super::{average_distance, find_and_remove, find_with, gui::EdgePos, DiffEntry, ResolveHints};
use crate::{
    basis::Dir,
    fragment::{Edge, Fragment},
};
use std::cell::RefCell;

fn find_by_single_side<'a, B>(
    fragments: &[Fragment],
    reference_edge: &Edge,
    blocklist: B,
) -> DiffEntry
where
    B: Iterator<Item = &'a EdgePos> + Clone + 'a,
{
    find_with(fragments, move |fragment| {
        let blocklist = blocklist.clone();
        fragment
            .edges
            .iter()
            .filter(move |e| {
                !blocklist
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

struct Context<'a> {
    hints: RefCell<&'a mut ResolveHints>,
    num_fragment: u8,
    fragments: RefCell<&'a mut Vec<Fragment>>,
    root_ref: &'a Fragment,
}

struct Finder<'a> {
    dir: Dir,
    list: &'a RefCell<Vec<Fragment>>,
    oppisite_list: &'a RefCell<Vec<Fragment>>,
    ctx: &'a Context<'a>,
    stop: bool,
}

impl<'a> Finder<'a> {
    fn apply_confirmed_pairs(&mut self) {
        let fragment_pos = self.list.borrow().last().unwrap_or(self.ctx.root_ref).pos;
        let edgepos = EdgePos::new(fragment_pos, self.dir);

        if let Some(pairs) = self.ctx.hints.borrow_mut().confirmed_pairs_of(edgepos) {
            let (pairs, should_continue) = pairs;
            if self.list.borrow().len() + self.oppisite_list.borrow().len() + pairs.len() + 1
                > self.ctx.num_fragment as usize
            {
                println!("shaker_fill: couldn't apply confirmed_pairs because of size overrun");
            } else {
                let mut done = false;
                let pairs_len = pairs.len();
                for (i, (pos, rot)) in pairs.into_iter().enumerate() {
                    let mut fragment = match find_and_remove(*self.ctx.fragments.borrow_mut(), pos)
                    {
                        Some(v) => v,
                        None => {
                            println!("shaker_fill: partially applied confirmed_pair because fragment in pair is already taken. edgepos: {:?}", edgepos);
                            break;
                        }
                    };

                    fragment.rotate(rot);
                    self.list.borrow_mut().push(fragment);

                    if i == pairs_len - 1 {
                        done = true;
                    }
                }
                if done && !should_continue {
                    self.stop = true;
                }
            }
        }
    }

    fn find_match(&self) -> DiffEntry {
        let list_ref = self.list.borrow();
        let fragment_ref = list_ref.last().unwrap_or(self.ctx.root_ref);
        let mut result = find_by_single_side(
            *self.ctx.fragments.borrow(),
            fragment_ref.edges.edge(self.dir),
            self.ctx.hints.borrow().blocklist_of(fragment_ref.pos),
        );

        if self.stop {
            result.score = f64::MAX;
        }

        result
    }

    fn adopt(&mut self, d: DiffEntry) {
        let mut fragment = find_and_remove(*self.ctx.fragments.borrow_mut(), d.pos).unwrap();
        fragment.rotate(self.dir.calc_rot(d.dir));
        self.list.borrow_mut().push(fragment);
    }
}

/// root_ref から left_dir と left_dir.opposite() 方向に探索して、スコアが良い順に採用する。
pub(super) fn shaker_fill(
    num_fragment: u8,
    fragments: &mut Vec<Fragment>,
    left_dir: Dir,
    root_ref: &Fragment,
    hints: &mut ResolveHints,
) -> (Vec<Fragment>, Vec<Fragment>) {
    let (left, right) = (RefCell::new(vec![]), RefCell::new(vec![]));

    let ctx = Context {
        hints: RefCell::new(hints),
        num_fragment,
        fragments: RefCell::new(fragments),
        root_ref,
    };

    let mut left_finder = Finder {
        dir: left_dir,
        list: &left,
        oppisite_list: &right,
        ctx: &ctx,
        stop: false,
    };

    let mut right_finder = Finder {
        dir: left_dir.opposite(),
        list: &right,
        oppisite_list: &left,
        ctx: &ctx,
        stop: false,
    };

    while right.borrow().len() + left.borrow().len() + (1/* for root */) != num_fragment as usize {
        right_finder.apply_confirmed_pairs();
        left_finder.apply_confirmed_pairs();

        if right.borrow().len() + left.borrow().len() + 1 == num_fragment as usize {
            break;
        }

        let right_score = right_finder.find_match();
        let left_score = left_finder.find_match();

        if right_score.score < left_score.score {
            right_finder.adopt(right_score);
        } else {
            left_finder.adopt(left_score);
        }
    }

    (left.into_inner(), right.into_inner())
}
