use crate::{
    fragment::Fragment,
    grid::{Pos, VecOnGrid},
};

/// 原画像とマッチング後の画像を使って, 完成形からどのように断片画像の位置が移動したかを求める.
///
/// [`crate::move_resolve::resolve`] の `movements` の形式に等しい.
pub(crate) fn map_fragment(matched: &VecOnGrid<Fragment>) -> Vec<(Pos, Pos)> {
    let mut map = vec![];
    for (pos, frag) in matched.iter_with_pos() {
        // frag.pos の位置から pos の位置へ移動してきた
        if frag.pos != pos {
            map.push((pos, frag.pos));
        }
    }
    map
}
