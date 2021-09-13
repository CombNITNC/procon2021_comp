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
        result += &format!("{}",r);
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
