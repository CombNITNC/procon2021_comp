/// `Pos` は `Grid` に存在する座標を表す.
///
/// フィールドの `u8` の上位 4 ビットに X 座標, 下位 4 ビットに Y 座標を格納する. それぞれは必ず `Grid` の `width` と `height` 未満になる.
#[derive(Clone, Copy)]
pub(crate) struct Pos(u8);

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl Pos {
    fn new(x: u8, y: u8) -> Self {
        debug_assert!(x <= 0xf, "");
        debug_assert!(y <= 0xf, "");
        Self((x as u8) << 4 | y as u8)
    }

    pub(crate) fn x(&self) -> u8 {
        self.0 >> 4 & 0xf
    }

    pub(crate) fn y(&self) -> u8 {
        self.0 & 0xf
    }
}

/// `Grid` は原画像を断片画像に分ける時の分割グリッドを表す. `Pos` はこれを介してのみ作成できる.
struct Grid {
    width: u8,
    height: u8,
}

impl Grid {
    pub(crate) fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    pub(crate) fn wrapping_pos(&self, x: u8, y: u8) -> Pos {
        Pos::new(x.clamp(0, self.width - 1), y.clamp(0, self.height - 1))
    }

    pub(crate) fn up_of(&self, pos: Pos) -> Option<Pos> {
        (pos.y() != 0).then(|| Pos::new(pos.x(), pos.y() - 1))
    }
    pub(crate) fn right_of(&self, pos: Pos) -> Option<Pos> {
        (pos.x() + 1 != self.width).then(|| Pos::new(pos.x() + 1, pos.y()))
    }
    pub(crate) fn down_of(&self, pos: Pos) -> Option<Pos> {
        (pos.y() + 1 != self.height).then(|| Pos::new(pos.x(), pos.y() + 1))
    }
    pub(crate) fn left_of(&self, pos: Pos) -> Option<Pos> {
        (pos.x() != 0).then(|| Pos::new(pos.x() - 1, pos.y()))
    }

    pub(crate) fn around_of(&self, pos: Pos) -> Vec<Pos> {
        [
            self.up_of(pos),
            self.right_of(pos),
            self.down_of(pos),
            self.left_of(pos),
        ]
        .iter()
        .flatten()
        .cloned()
        .collect()
    }
}
