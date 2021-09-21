#![allow(dead_code)]

use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use png::{BitDepth, ColorType, Compression, Encoder};

mod basis;
mod fragment;
mod grid;
mod image;
mod kaitou;
mod move_resolve;
mod pixel_match;
mod submit;

use crate::{
    basis::Color,
    fragment::Fragment,
    grid::{Grid, VecOnGrid},
};

fn main() {
    let file = File::open("problem.ppm").expect("failed to open problem file");
    let reader = BufReader::new(file);
    let problem = image::read_problem(reader).unwrap();
    let grid = Grid::new(problem.rows, problem.cols);

    let recovered_image = pixel_match::resolve(&problem, grid);
    debug_image_output("recovered_image.png", grid, recovered_image);
}

fn debug_image_output(name: &str, grid: Grid, fragment_grid: VecOnGrid<Option<Fragment>>) {
    let mut colors_grid: VecOnGrid<Option<Vec<Color>>> = VecOnGrid::with_default(grid);

    let side_length = fragment_grid
        .iter()
        .next()
        .unwrap()
        .as_ref()
        .unwrap()
        .side_length();

    for (pos, data) in fragment_grid.into_iter_with_pos() {
        colors_grid[pos] = Some(data.unwrap().pixels());
    }

    let f = File::create(name).unwrap();
    let f = BufWriter::new(f);

    let mut encoder = Encoder::new(
        f,
        (side_length * grid.width() as usize) as u32,
        (side_length * grid.height() as usize) as u32,
    );

    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);

    let mut writer = encoder.write_header().unwrap();

    let zeros = vec![Color { r: 0, g: 0, b: 0 }; side_length];
    let mut data = vec![];

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                if let Some(t) = &colors_grid[(grid.pos(x, y))] {
                    data.extend_from_slice(
                        &t[(py * side_length) as usize..((py + 1) * side_length) as usize],
                    );
                } else {
                    data.extend_from_slice(&zeros);
                }
            }
        }
    }

    let data = data
        .into_iter()
        .flat_map(|x| [x.r, x.g, x.b])
        .collect::<Vec<_>>();

    writer.write_image_data(&data).unwrap();
}
