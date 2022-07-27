use crate::basis::{Movement, Operation, Rot};

const NEW_LINE: &str = "\r\n";

pub fn ans(ope: &[Operation], rot: &[Rot]) -> String {
    let mut result = String::new();

    //回転情報
    for i in rot {
        let r = match i {
            Rot::R0 => 0,
            Rot::R90 => 1,
            Rot::R180 => 2,
            Rot::R270 => 3,
        };
        result += &r.to_string();
    }
    result += NEW_LINE;

    //選択回数
    result += &ope.len().to_string();
    result += NEW_LINE;

    for i in ope {
        //選択画像位置
        use std::fmt::Write as _;
        let _ = write!(result, "{:X}{:X}", i.select.x(), i.select.y());
        result += NEW_LINE;

        //交換回数
        result += &i.movements.len().to_string();
        result += NEW_LINE;

        //交換操作
        for j in &i.movements {
            let m = match j {
                Movement::Up => 'U',
                Movement::Right => 'R',
                Movement::Down => 'D',
                Movement::Left => 'L',
            };
            result.push(m);
        }
        result += NEW_LINE;
    }

    result
}

#[test]
fn case1() {
    use crate::grid::Grid;

    let grid = Grid::new(12, 2);

    let expected = "01230320111103230210\r\n1\r\nA1\r\n4\r\nUDLR\r\n".to_owned();
    let actual = ans(
        &[Operation {
            select: grid.pos(10, 1),
            movements: vec![
                Movement::Up,
                Movement::Down,
                Movement::Left,
                Movement::Right,
            ],
        }],
        &[
            Rot::R0,
            Rot::R90,
            Rot::R180,
            Rot::R270,
            Rot::R0,
            Rot::R270,
            Rot::R180,
            Rot::R0,
            Rot::R90,
            Rot::R90,
            Rot::R90,
            Rot::R90,
            Rot::R0,
            Rot::R270,
            Rot::R180,
            Rot::R270,
            Rot::R0,
            Rot::R180,
            Rot::R90,
            Rot::R0,
        ],
    );

    assert_eq!(expected, actual);
}
