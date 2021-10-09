#![allow(dead_code)]

use std::ops::{Add, AddAssign};

use crate::grid::Pos;

/// `Color` は 24 ビットの RGB カラーを表す.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Color {
    /// RGB色空間での色同士の距離を求める
    #[inline]
    pub(crate) fn euclidean_distance(&self, c: Color) -> f64 {
        let r = (self.r as i16 - c.r as i16) as f64;
        let g = (self.g as i16 - c.g as i16) as f64;
        let b = (self.b as i16 - c.b as i16) as f64;
        f64::sqrt(r * r + g * g + b * b)
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Color")
            .field(&format_args!(
                "#{:06x}",
                (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
            ))
            .finish()
    }
}

/// `Movement` はある断片画像を動かして入れ替える向きを表す.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Movement {
    Up,
    Right,
    Down,
    Left,
}

impl Movement {
    /// `from` から `to` へ移動させるときの向きを求める.
    /// 要件: (from.x() == to.x()) ^ (from.y() == to.y())`
    pub(crate) fn between_pos(from: Pos, to: Pos) -> Self {
        use Movement::*;
        let from_x = from.x() as i32;
        let from_y = from.y() as i32;
        let to_x = to.x() as i32;
        let to_y = to.y() as i32;
        if from_x == to_x {
            match (1 < (from_y - to_y).abs(), from_y < to_y) {
                (true, true) => Up,
                (true, false) => Down,
                (false, true) => Down,
                (false, false) => Up,
            }
        } else if from_y == to_y {
            match (1 < (from_x - to_x).abs(), from_x < to_x) {
                (true, true) => Left,
                (true, false) => Right,
                (false, true) => Right,
                (false, false) => Left,
            }
        } else {
            unreachable!()
        }
    }

    pub(crate) fn turn_right(self) -> Self {
        match self {
            Movement::Up => Movement::Right,
            Movement::Right => Movement::Down,
            Movement::Down => Movement::Left,
            Movement::Left => Movement::Up,
        }
    }

    pub(crate) fn turn_left(self) -> Self {
        match self {
            Movement::Up => Movement::Left,
            Movement::Right => Movement::Up,
            Movement::Down => Movement::Right,
            Movement::Left => Movement::Down,
        }
    }

    pub(crate) fn opposite(self) -> Self {
        match self {
            Movement::Up => Movement::Down,
            Movement::Right => Movement::Left,
            Movement::Down => Movement::Up,
            Movement::Left => Movement::Right,
        }
    }
}

/// `Operation` は座標 `select` の断片画像を選択してから `movements` の入れ替えを行う操作を表す.
#[derive(Debug, PartialEq)]
pub(crate) struct Operation {
    pub(crate) select: Pos,
    pub(crate) movements: Vec<Movement>,
}

/// `Rot` はある断片画像を原画像の状態から時計回りに回転させた角度を表す.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rot {
    R0,
    R90,
    R180,
    R270,
}

impl Rot {
    #[inline]
    pub(crate) fn as_num(self) -> u8 {
        match self {
            Rot::R0 => 0,
            Rot::R90 => 1,
            Rot::R180 => 2,
            Rot::R270 => 3,
        }
    }

    #[inline]
    pub(crate) fn from_num(rot: u8) -> Self {
        assert!(rot <= 3, "rot must be lower than 4");
        match rot {
            0 => Rot::R0,
            1 => Rot::R90,
            2 => Rot::R180,
            3 => Rot::R270,
            _ => unreachable!(),
        }
    }

    pub(crate) fn as_degrees(self) -> f64 {
        match self {
            Rot::R0 => 0.0,
            Rot::R90 => 90.0,
            Rot::R180 => 180.0,
            Rot::R270 => 270.0,
        }
    }
}

impl Add for Rot {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_num((self.as_num() + rhs.as_num()) % 4)
    }
}

impl AddAssign for Rot {
    fn add_assign(&mut self, rhs: Self) {
        let rot = (self.as_num() + rhs.as_num()) % 4;
        *self = Self::from_num(rot);
    }
}

/// `Dir` はある断片画像において辺が位置する向きを表す.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// 自分を四角形の辺の方向としたとき、対辺の方向を返す。
    #[inline]
    pub(crate) fn opposite(self) -> Self {
        match self {
            Dir::North => Dir::South,
            Dir::East => Dir::West,
            Dir::South => Dir::North,
            Dir::West => Dir::East,
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

/// `Problem` は原画像から抽出される問題設定の情報を表す.
#[derive(Debug)]
pub(crate) struct Problem {
    pub(crate) select_limit: u8,
    pub(crate) select_cost: u16,
    pub(crate) swap_cost: u16,
    pub(crate) rows: u8,
    pub(crate) cols: u8,
    pub(crate) image: Image,
}

pub(crate) struct Image {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) pixels: Vec<Color>,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}
