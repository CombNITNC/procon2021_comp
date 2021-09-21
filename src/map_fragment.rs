use crate::{
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

/// 原画像とマッチング後の画像を使って, 完成形からどのように断片画像の位置が移動したかを求める.
///
/// [`crate::move_resolve::resolve`] の `movements` の形式に等しい.
pub(crate) fn map_fragment(
    original: &VecOnGrid<Fragment>,
    matched: &VecOnGrid<Fragment>,
) -> Vec<(Pos, Pos)> {
    let mut map = vec![];
    for (before, after) in original.iter().zip(matched.iter()) {
        if before.pos != after.pos {
            map.push((after.pos, before.pos));
        }
    }
    map
}
