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
        debug_assert_eq!(north.len(), east.len());
        debug_assert_eq!(north.len(), south.len());
        debug_assert_eq!(north.len(), west.len());
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
        let grid = Grid::new(rows, cols);
        let frag_width = width / rows as u16;
        let frag_height = height / cols as u16;
        let mut frags = vec![];

        for col in 0..cols {
            for row in 0..rows {
                frags.push(Self::new(
                    &pixels,
                    grid.clamping_pos(row, col),
                    width as usize,
                    frag_width,
                    frag_height,
                ));
            }
        }
        frags
    }

    fn new(
        pixels: &[Color],
        pos: Pos,
        whole_width: usize,
        frag_width: u16,
        frag_height: u16,
    ) -> Self {
        let as_index = |x: u16, y: u16| -> usize {
            let x = (x + pos.x() as u16 * frag_width) as usize;
            let y = (y + pos.y() as u16 * frag_height) as usize;
            x + y * whole_width
        };
        let mut north = vec![];
        let mut east = vec![];
        let mut south = vec![];
        let mut west = vec![];
        for y in 0..frag_height {
            for x in 0..frag_width {
                let index = as_index(x, y);
                if x == 0 {
                    west.push(pixels[index]);
                }
                if x == frag_width - 1 {
                    east.push(pixels[index]);
                }
                if y == 0 {
                    north.push(pixels[index]);
                }
                if y == frag_height - 1 {
                    south.push(pixels[index]);
                }
            }
        }
        // make edges to be rotating clockwise
        south.reverse();
        west.reverse();
        Self {
            pos,
            rot: Rot::R0,
            edges: Edges::new(north, east, south, west),
        }
    }
}
