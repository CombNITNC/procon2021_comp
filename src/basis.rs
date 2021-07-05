#![allow(dead_code)]

use crate::grid::Pos;

/// `Color` は 24 ビットの RGB カラーを表す.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Movement {
    Up,
    Right,
    Down,
    Left,
}

impl Movement {
    /// `from` から `to` へ移動させるときの向きを求める.
    /// 要件: `from.manhattan_distance(to) == 1 && ((from.x() == to.x()) ^ (from.y() == to.y()))`
    pub(crate) fn between_pos(from: Pos, to: Pos) -> Self {
        use std::cmp::Ordering::*;
        match (from.x().cmp(&to.x()), from.y().cmp(&to.y())) {
            (Less, _) => Self::Right,
            (Greater, _) => Self::Left,
            (_, Less) => Self::Down,
            (_, Greater) => Self::Up,
            _ => unreachable!(),
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
#[derive(Debug, Clone, Copy)]
pub(crate) enum Rot {
    R0,
    R90,
    R180,
    R270,
}

/// `Dir` はある断片画像において辺が位置する向きを表す.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// `Problem` は原画像から抽出される問題設定の情報を表す.
pub(crate) struct Problem {
    pub(crate) select_limit: u8,
    pub(crate) select_cost: u16,
    pub(crate) swap_cost: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) rows: u8,
    pub(crate) cols: u8,
    pub(crate) pixels: Vec<Color>,
}
