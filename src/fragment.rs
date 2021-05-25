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
        let frag_edge = width / rows as u16;
        debug_assert_eq!(frag_edge, height / cols as u16, "Fragment must be a square");
        let grid = Grid::new(rows, cols);

        let mut frags = vec![];
        for col in 0..cols {
            for row in 0..rows {
                frags.push(Self::new(
                    &pixels,
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
        for y in 0..frag_edge {
            for x in 0..frag_edge {
                let index = as_index(x, y);
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

#[cfg(test)]
mod tests {
    use {
        crate::{
            basis::{Color, Dir, Rot},
            fragment::Fragment,
            grid::Grid,
        },
        std::io::{self, Read, Result},
    };

    fn pixels(to_skip: u64, width: usize, height: usize, path: &str) -> Result<Vec<Color>> {
        let mut img = std::fs::File::open(path)?;
        io::copy(&mut img.by_ref().take(to_skip), &mut io::sink())?;
        let mut pixel_components = vec![0; width * height * 3];
        img.read(&mut pixel_components)?;
        Ok(pixel_components
            .chunks(3)
            .map(|comps| Color {
                r: comps[0],
                g: comps[1],
                b: comps[2],
            })
            .collect())
    }

    #[test]
    fn case1() -> Result<()> {
        let width = 180;
        let pixels = pixels(32, width, 120, "test_cases/02_sampled.ppm")?;
        let grid = Grid::new(3, 2);
        let test_cases = &[
            grid.clamping_pos(0, 0),
            grid.clamping_pos(1, 1),
            grid.clamping_pos(2, 1),
        ];

        for pos in test_cases {
            let frag = Fragment::new(&pixels, pos.clone(), width, 60);

            let (x, y) = (pos.x() as usize, pos.y() as usize);
            let north = &frag.edges.0[0];
            assert!(matches!(north.dir, Dir::North));
            for i in 0..60 {
                assert_eq!(&north.pixels[i], &pixels[x * 60 + i + y * 60 * width]);
            }
            assert!(matches!(frag.rot, Rot::R0));
        }
        Ok(())
    }
}
