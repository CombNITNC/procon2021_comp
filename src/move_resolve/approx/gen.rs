use crate::{
    basis::Movement,
    grid::{board::BoardFinder, Pos},
};

use super::NextTargetsGenerator;

#[derive(Debug, Clone)]
pub struct FromOutside;

impl NextTargetsGenerator for FromOutside {
    fn next_targets(&mut self, finder: &BoardFinder) -> Vec<Pos> {
        let mut start = finder.offset();
        let end = finder.move_pos_to(finder.offset(), Movement::Left);
        let mut result = vec![];
        while start != end {
            result.push(start);
            start = finder.move_pos_to(start, Movement::Right);
        }
        if !result.is_empty() {
            result.push(end);
        }
        result
    }
}

#[test]
fn test_from_outside() {
    use crate::grid::Grid;

    let grid = Grid::new(4, 3);
    let finder = BoardFinder::new(grid);
    let mut gen = FromOutside;

    let expected = vec![
        grid.pos(0, 0),
        grid.pos(1, 0),
        grid.pos(2, 0),
        grid.pos(3, 0),
    ];
    let actual = gen.next_targets(&finder);

    assert_eq!(expected, actual);
}
