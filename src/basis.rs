#![allow(dead_code)]

/// `Color` は 24 ビットの RGB カラーを表す.
#[derive(Clone)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.r << 16 | self.g << 8 | self.b)
    }
}

/// `Pos` は画像における座標を表す.
///
/// フィールドの `u8` の上位 4 ビットに X 座標, 下位 4 ビットに Y 座標を格納する.
#[derive(Clone, Copy)]
pub(crate) struct Pos(u8);

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl Pos {
    pub(crate) fn new(x: u8, y: u8) -> Self {
        Self(x << 4 | y)
    }

    pub(crate) fn x(&self) -> u8 {
        self.0 >> 4 & 0xf
    }

    pub(crate) fn y(&self) -> u8 {
        self.0 & 0xf
    }

    pub(crate) fn is_valid(&self, width: u8, height: u8) -> bool {
        self.x() < width && self.y() < height
    }
}

/// `Movement` はある断片画像を動かして入れ替える向きを表す.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Movement {
    Up,
    Right,
    Down,
    Left,
}

/// `Operation` は座標 `select` の断片画像を選択してから `movements` の入れ替えを行う操作を表す.
#[derive(Debug)]
pub(crate) struct Operation {
    select: Pos,
    movements: Vec<Movement>,
}

/// `Rot` はある断片画像を原画像の状態から時計回りに回転させた角度を表す.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Rot {
    R0,
    R90,
    R180,
    R270,
}

/// `Dir` はある断片画像において辺が位置する向きを表す.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    fn r90(self) -> Dir {
        match self {
            Dir::North => Dir::East,
            Dir::East => Dir::South,
            Dir::South => Dir::West,
            Dir::West => Dir::North,
        }
    }

    /// `self` の値を `rot` だけ回転させた向きの値にする.
    pub(crate) fn rotate(self, rot: Rot) -> Self {
        match rot {
            Rot::R0 => self,
            Rot::R90 => self.r90(),
            Rot::R180 => self.r90().r90(),
            Rot::R270 => self.r90().r90().r90(),
        }
    }

    /// この辺 `self` に別の断片画像を回転させてその辺 `other` をつなげるとき, 別の断片画像を回転させる角度を計算する.
    pub(crate) fn calc_rot(mut self, mut other: Self) -> Rot {
        while !matches!(self, Dir::North) {
            self = self.r90();
            other = other.r90();
        }
        use Rot::*;
        match other {
            Dir::North => R180,
            Dir::East => R90,
            Dir::South => R0,
            Dir::West => R270,
        }
    }
}

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
