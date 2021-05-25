use crate::{
    basis::{Color, Dir, Problem, Rot},
    grid::{Grid, Pos},
};

/// `Edge` は断片画像における辺のピクセル列を表す.
#[derive(Debug, Clone)]
pub(crate) struct Edge {
    pub(crate) dir: Dir,
    pub(crate) pixels: Vec<Color>,
}

/// `Edges` は断片画像の縁の四辺を表す. また, 断片画像を回転させたときでも同じように扱えるようにする.
#[derive(Debug)]
pub(crate) struct Edges([Edge; 4]);

impl Edges {
    fn new(north: Vec<Color>, east: Vec<Color>, south: Vec<Color>, west: Vec<Color>) -> Self {
        Self([
            Edge {
                dir: Dir::North,
                pixels: north,
            },
            Edge {
                dir: Dir::East,
                pixels: east,
            },
            Edge {
                dir: Dir::South,
                pixels: south,
            },
            Edge {
                dir: Dir::West,
                pixels: west,
            },
        ])
    }
}

/// `Fragment` は原画像から切り取った断片画像を表す. その座標 `pos` と回転させた向き `rot` と縁四辺 `edges` を表す.
#[derive(Debug)]
pub(crate) struct Fragment {
    pos: Pos,
    rot: Rot,
    edges: Edges,
}

impl Fragment {
    pub(crate) fn new_all(
        Problem {
            width,
            height,
            rows,
            cols,
            pixels,
            ..
        }: Problem,
    ) -> Vec<Self> {
        let frag_width = (width / rows as u16) as u8;
        let frag_height = (height / cols as u16) as u8;
        let grid = Grid::new(frag_width, frag_height);
        let mut frags = vec![];

        for col in 0..cols {
            for row in 0..rows {
                frags.push(Self::new(
                    &pixels,
                    grid.clamping_pos(row, col),
                    frag_width,
                    frag_height,
                ));
            }
        }
        frags
    }

    fn new(pixels: &[Color], pos: Pos, frag_width: u8, frag_height: u8) -> Self {
        todo!()
    }
}
