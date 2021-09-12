use crate::basis::{Operation, Rot, Movement};

pub(crate) fn ans(ope:&[Operation], rot:&[Rot]) -> String{
    let mut result= String::new();

    //回転情報
    for i in rot{
        result += match i {
            Rot::R0 => &format!("{}",0),
            Rot::R90 => &format!("{}",1),
            Rot::R180 => &format!("{}",2),
            Rot::R270 => &format!("{}",3),
        };
    }
    result += &format!("/r/n");

    //選択回数
    result += &format!("{}/r/n",ope.len());

    for i in ope{
        //選択画像位置 x(),y()の使い方違うかも
        result += &format!("{}{}/r/n",i.select.x(),i.select.y());

        //交換回数
        result += &format!("{}/r/n",i.movements.len());
        
        //交換操作
        for j in 0..(i.movements.len() - 1){
            result += match i.movements[j]{
                Movement::Up => &format!("{}",'U'),
                Movement::Right => &format!("{}",'R'),
                Movement::Down => &format!("{}",'D'),
                Movement::Left => &format!("{}",'L'),
            };
        }
        result += &format!("/r/n");
    }

    result
}