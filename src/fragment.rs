pub mod map_fragment;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::{
    basis::{Color, Dir, Image, Problem, Rot},
    grid::{Grid, Pos},
};

/// `Edge` は断片画像における辺のピクセル列を表す.
#[derive(Debug, Clone)]
pub struct Edge {
    pub dir: Dir,
    pub pixels: Vec<Color>,
}

/// `Edges` は断片画像の縁の四辺を表す. また, 断片画像を回転させたときでも同じように扱えるようにする.
#[derive(Debug, Clone)]
pub struct Edges([Edge; 4]);

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

    fn rotate(&mut self, rot: Rot) {
        for edge in &mut self.0 {
            edge.dir = edge.dir.rotate(rot);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Edge> {
        self.0.iter()
    }

    pub fn edge(&self, dir: Dir) -> &Edge {
        self.0
            .iter()
            .find(|edge| edge.dir == dir)
            .expect("dirs of edges were broken")
    }
}

/// `Fragment` は原画像から切り取った断片画像を表す.
#[derive(Debug, Clone)]
pub struct Fragment {
    /// 原画像におけるこの断片画像の座標.
    pub pos: Pos,
    /// この断片画像のを元の原画像から回転させている座標.
    pub rot: Rot,
    /// この断片画像の縁四辺.
    pub edges: Edges,
    pixels: LazyRotate,
}

impl Fragment {
    pub fn side_length(&self) -> usize {
        self.edges.0[0].pixels.len()
    }

    pub fn rotate(&mut self, rot: Rot) {
        self.rot += rot;
        self.edges.rotate(rot);
    }

    pub fn pixels(&mut self) -> &[Color] {
        self.pixels.get(self.rot)
    }

    pub fn apply_rotate(&mut self) {
        let _ = self.pixels.get(self.rot);

        self.pixels.rot = Rot::R0;
        self.rot = Rot::R0;
    }

    pub fn new_all(
        &Problem {
            rows,
            cols,
            image:
                Image {
                    width,
                    height,
                    ref pixels,
                },
            ..
        }: &Problem,
    ) -> Vec<Self> {
        let frag_edge = width / rows as u16;
        debug_assert_eq!(frag_edge, height / cols as u16, "Fragment must be a square");
        let grid = Grid::new(rows, cols);

        let mut frags = vec![];
        for col in 0..cols {
            for row in 0..rows {
                frags.push(Self::new(
                    pixels,
                    grid.clamping_pos(row, col),
                    width as usize,
                    frag_edge,
                ));
            }
        }
        frags
    }

    fn new(pixels: &[Color], pos: Pos, whole_width: usize, frag_edge: u16) -> Self {
        let as_index = |x: u16, y: u16| -> usize {
            let x = (x + pos.x() as u16 * frag_edge) as usize;
            let y = (y + pos.y() as u16 * frag_edge) as usize;
            x + y * whole_width
        };
        let mut north = vec![];
        let mut east = vec![];
        let mut south = vec![];
        let mut west = vec![];
        let mut all = vec![];
        for y in 0..frag_edge {
            for x in 0..frag_edge {
                let index = as_index(x, y);
                all.push(pixels[index]);
                if x == 0 {
                    west.push(pixels[index]);
                }
                if x == frag_edge - 1 {
                    east.push(pixels[index]);
                }
                if y == 0 {
                    north.push(pixels[index]);
                }
                if y == frag_edge - 1 {
                    south.push(pixels[index]);
                }
            }
        }
        // 辺のピクセルが時計回りになるようにする
        south.reverse();
        west.reverse();
        Self {
            pos,
            rot: Rot::R0,
            pixels: LazyRotate::new(all, north.len()),
            edges: Edges::new(north, east, south, west),
        }
    }
}

#[derive(Debug, Clone)]
struct LazyRotate {
    data: Vec<Color>,
    rot: Rot,
    row: usize,
}

impl LazyRotate {
    fn new(data: Vec<Color>, row: usize) -> Self {
        Self {
            data,
            rot: Rot::R0,
            row,
        }
    }

    fn get(&mut self, rot: Rot) -> &[Color] {
        if self.rot != rot {
            self.rotate(self.rot_count(rot));
            self.rot = rot;
        }

        &self.data
    }

    /// 自分を何回回転させれば `target_rot` になるかを求める
    fn rot_count(&self, target_rot: Rot) -> usize {
        let mut my_rot = self.rot;
        let mut count = 0;
        loop {
            my_rot += Rot::R90;
            count += 1;

            if my_rot == target_rot {
                break count;
            }
        }
    }

    fn rotate(&mut self, count: usize) {
        let init_vec = || vec![Color { r: 0, g: 0, b: 0 }; self.row * self.row];

        let mut temp = Cow::Borrowed(&self.data);
        let mut result = init_vec();

        for _ in 0..count {
            for i in 0..self.row {
                for j in 0..self.row {
                    result[j * self.row + self.row - 1 - i] = temp[i * self.row + j];
                }
            }

            temp = Cow::Owned(result);
            result = init_vec();
        }

        self.data = temp.into_owned();
    }
}
