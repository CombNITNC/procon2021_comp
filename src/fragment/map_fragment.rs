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

#[test]
fn test_map() {
    use crate::{basis::Rot, fragment::Edges, grid::Grid};

    let grid = Grid::new(2, 2);
    let case = {
        let mut vec = VecOnGrid::with_init(
            grid,
            Fragment {
                pos: grid.pos(0, 0),
                rot: Rot::R0,
                edges: Edges::new(vec![], vec![], vec![], vec![]),
                pixels: LazyRotate::new(vec![], 0),
            },
        );
        vec[grid.pos(0, 0)].pos = grid.pos(1, 1);
        vec[grid.pos(1, 0)].pos = grid.pos(0, 0);
        vec[grid.pos(0, 1)].pos = grid.pos(1, 0);
        vec[grid.pos(1, 1)].pos = grid.pos(0, 1);
        vec
    };

    let expected: Vec<(Pos, Pos)> = vec![
        (grid.pos(0, 0), grid.pos(1, 1)),
        (grid.pos(1, 0), grid.pos(0, 0)),
        (grid.pos(0, 1), grid.pos(1, 0)),
        (grid.pos(1, 1), grid.pos(0, 1)),
    ];
    let actual = map_fragment(&case);

    assert_eq!(expected, actual);
}
