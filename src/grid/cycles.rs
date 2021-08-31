use super::{Grid, Pos, VecOnGrid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CyclesNode {
    Child { parent: Pos },
    Root { len: u8 },
}

use CyclesNode::*;

/// 入れ替える頂点の互換を Union-Find で管理する.
#[derive(Debug, Clone)]
pub(crate) struct Cycles<'grid> {
    map: VecOnGrid<'grid, (CyclesNode, Pos)>,
}

impl<'grid> Cycles<'grid> {
    pub(crate) fn new(grid: &'grid Grid, cycles: &[(Pos, Pos)]) -> Self {
        let mut c = Self {
            map: VecOnGrid::with_init(grid, (Root { len: 1 }, grid.clamping_pos(0, 0))),
        };
        for &(a, b) in cycles {
            c.map[b].1 = a;
            c.union(a, b);
        }
        c
    }

    fn union(&mut self, a: Pos, b: Pos) {
        assert!(self.map.grid.is_pos_valid(a));
        assert!(self.map.grid.is_pos_valid(b));
        let a = self.repr(a);
        let b = self.repr(b);
        if a == b {
            return;
        }
        if let ((Root { len: a_len }, _), (Root { len: b_len }, _)) = self.map.pick_two(a, b) {
            if a_len < b_len {
                self.map.swap(a, b);
            }
            if let ((Root { len: a_len }, _), (Root { len: b_len }, _)) =
                self.map.pick_two_mut(a, b)
            {
                *a_len += *b_len;
            }
            self.map[b].0 = Child { parent: a };
        }
    }

    fn repr(&mut self, x: Pos) -> Pos {
        assert!(self.map.grid.is_pos_valid(x));
        match self.map[x].0 {
            Root { .. } => x,
            Child { parent } => {
                let parent = self.repr(parent);
                self.map[x].0 = Child { parent };
                parent
            }
        }
    }

    pub(crate) fn grid(&self) -> &Grid {
        self.map.grid
    }

    pub(crate) fn tree_count(&self) -> usize {
        self.map
            .iter()
            .filter(|n| matches!(n.0, Root { .. }))
            .count()
    }

    pub(crate) fn on_swap(&mut self, a: Pos, b: Pos) {
        let (a_node, b_node) = self.map.pick_two_mut(a, b);
        std::mem::swap(&mut a_node.1, &mut b_node.1);
        if self.map[a].1 == b && self.map[b].1 == a {
            self.map[a].0 = Root { len: 1 };
            self.map[b].0 = Root { len: 1 };
            return;
        }
        if self.repr(a) == self.repr(b) {
            if self.map[a].1 == a {
                self.map[a].0 = Root { len: 1 };
            }
            if self.map[b].1 == b {
                self.map[b].0 = Root { len: 1 };
            }
        } else if self.map[a].1 == b || self.map[b].1 == a {
            self.union(a, b);
        }
    }

    pub(crate) fn scatter_amount(&self) -> u64 {
        self.map
            .iter_with_pos()
            .map(|(pos, &(n, goal))| match n {
                Child { parent } => self.grid().looping_manhattan_dist(pos, parent) as u64,
                Root { .. } => self.grid().looping_manhattan_dist(pos, goal) as u64,
            })
            .sum()
    }

    pub(crate) fn cycle_size(&mut self, belonged: Pos) -> u64 {
        assert!(self.map.grid.is_pos_valid(belonged));
        let repr = self.repr(belonged);
        match self.map[repr].0 {
            Child { .. } => unreachable!(),
            Root { len } => len as u64,
        }
    }

    pub(crate) fn different_cells(&'grid self) -> impl Iterator<Item = Pos> + 'grid {
        self.map
            .iter_with_pos()
            .filter(|&(_, &i)| !matches!(i.0, Root { len: 1 }))
            .map(|(p, _)| p)
    }
}

impl PartialEq for Cycles<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.map.iter().zip(other.map.iter()).all(|(a, b)| a == b)
    }
}

#[test]
fn test_cycles() {
    // 00 11
    // 10 01
    let grid = Grid::new(2, 2);
    let mut cycles = Cycles::new(
        &grid,
        &[
            (grid.pos(1, 0), grid.pos(0, 1)),
            (grid.pos(0, 1), grid.pos(1, 1)),
            (grid.pos(1, 1), grid.pos(1, 0)),
        ],
    );

    cycles.on_swap(grid.pos(0, 1), grid.pos(1, 1));

    assert_eq!(
        cycles.map[grid.pos(0, 1)],
        (CyclesNode::Root { len: 1 }, grid.pos(0, 1))
    );
    assert_eq!(
        cycles.map[grid.pos(1, 1)],
        (
            CyclesNode::Child {
                parent: grid.pos(1, 0)
            },
            grid.pos(1, 0)
        )
    );

    cycles.on_swap(grid.pos(1, 0), grid.pos(1, 1));

    grid.all_pos().for_each(|p| {
        assert_eq!(cycles.map[p], (CyclesNode::Root { len: 1 }, p));
    });
}
