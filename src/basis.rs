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
        self.x() <= width && self.y() <= height
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

/// `Edge` は画像上の連続する一直線のピクセル列を表す.
#[derive(Debug, Clone)]
pub(crate) struct Edge(pub(crate) Vec<Color>);

/// `Fragment` は原画像から切り取った断片画像を表す. その座標 `pos` と縁四辺 `north_edge`, `east_edge`, `south_edge`, `west_edge` を表す.
#[derive(Debug)]
pub(crate) struct Fragment {
    pos: Pos,
    north_edges: Edge,
    east_edges: Edge,
    south_edges: Edge,
    west_edges: Edge,
}
