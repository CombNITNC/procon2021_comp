use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut img = File::create("01_q.ppm")?;
    img.write_all(
        br"P6
# 2 2
# 1
# 3 1
32 32
255
",
    )?;
    const RADIUS: i32 = 16;
    for y in -RADIUS..RADIUS {
        for x in -RADIUS..RADIUS {
            let shift = if x < 0 {
                0
            } else if y < 0 {
                -16
            } else {
                16
            };
            let color = if x * x + (y - shift) * (y - shift) < RADIUS * RADIUS {
                255
            } else {
                0
            };
            img.write(&[color, color, color])?;
        }
    }
    let mut img = File::create("01_a.txt")?;
    img.write_all(
        br"0000
1
10
1
D
",
    )?;
    Ok(())
}
