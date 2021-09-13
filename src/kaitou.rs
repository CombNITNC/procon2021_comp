use crate::basis::{Movement, Operation, Rot};

pub(crate) fn ans(ope: &[Operation], rot: &[Rot]) -> String {
    let mut result = String::new();

    //回転情報
    for i in rot {
        let r = match i {
            Rot::R0 => 0,
            Rot::R90 => 1,
            Rot::R180 => 2,
            Rot::R270 => 3,
        };
        result += &format!("{}", r);
    }
    result += "/r/n";

    //選択回数
    result += &format!("{}/r/n", ope.len());

    for i in ope {
        //選択画像位置
        result += &format!("{}{}/r/n", i.select.x(), i.select.y());

        //交換回数
        result += &format!("{}/r/n", i.movements.len());

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
        result += "/r/n";
    }

    result
}

#[test]
fn case1() {
    use crate::grid::Grid;

    let grid = Grid::new(12, 2);

    let expected = "01230320111103230210\r\n1\r\nA0\r\n3\r\nUDLR".to_owned();
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
