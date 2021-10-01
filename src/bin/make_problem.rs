use image::{io::Reader, ImageFormat};
use std::{
    env,
    fs::File,
    io::{BufReader, Write},
    path::Path,
};

fn main() {
    let mut args = env::args().skip(1);
    let rows: u32 = args
        .next()
        .expect("the splitting rows must be provided")
        .parse()
        .expect("expected an integer");
    let cols: u32 = args
        .next()
        .expect("the splitting columns must be provided")
        .parse()
        .expect("expected an integer");

    let src_path = args.next().expect("the source image path must be provided");
    let src_path = Path::new(&src_path);
    let src = File::open(src_path).expect("the source image path must be valid");
    let reader = Reader::new(BufReader::new(src)).with_guessed_format().unwrap();
    let img = reader
        .decode()
        .expect("the source image format is not supported");
    let rgb = img.to_rgb8();
    let width = rgb.width();
    let height = rgb.height();
    let rgb_pixels = rgb.into_raw();

    assert_eq!(width % rows, 0, "width must be divisible by split rows");
    assert_eq!(height % cols, 0, "height must be divisible by split cols");
    assert_eq!(width / rows, height / cols, "fragments must be square");

    println!("P6");
    println!("# {} {}", rows, cols);
    println!("# 3");
    println!("# 2 1");
    println!("{} {}", width, height);
    println!("255");

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&rgb_pixels)
        .expect("failed to output binary");
}
