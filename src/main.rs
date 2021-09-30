#![allow(dead_code)]

use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor},
};

use cfg_if::cfg_if;
use png::{BitDepth, ColorType, Compression, Encoder};
use rand::prelude::*;

mod basis;
mod fragment;
mod grid;
mod image;
mod kaitou;
mod move_resolve;
mod pixel_match;
#[cfg(feature = "net")]
mod submit;

use crate::{
    basis::{Color, Image, Problem, Rot},
    fragment::Fragment,
    grid::{Grid, VecOnGrid},
};

fn main() {
    cfg_if! {
        if #[cfg(feature = "net")] {
            dotenv::dotenv().ok();
            let submit_token =
                std::env::var("TOKEN").expect("set TOKEN environment variable for auto submit");
        }
    }

    let file = File::open("problem.ppm").expect("failed to open problem file");
    let reader = BufReader::new(file);
    let problem = image::read_problem(reader).unwrap();
    let grid = Grid::new(problem.rows, problem.cols);
    let fragments = fragment::Fragment::new_all(&problem);

    let mut recovered_image = pixel_match::resolve(fragments, grid);
    println!("pixel_match::resolve() done");

    let movements = fragment::map_fragment::map_fragment(&recovered_image);
    println!("{:#?}", movements);

    let ops = move_resolve::resolve(
        grid,
        &movements,
        problem.select_limit,
        problem.swap_cost,
        problem.select_cost,
    );
    println!("{:#?}", ops);

    println!("move_resolve::resolve() done");
    println!("submitting");

    let rots = recovered_image.iter().map(|x| x.rot).collect::<Vec<_>>();
    let result = kaitou::ans(&ops, &rots);

    println!("{}", result);

    cfg_if! {
        if #[cfg(feature = "net")] {
            let submit_result = submit::submit(&submit_token, &result);
            println!("submit result: {:#?}", submit_result);
        }
    }
}

fn debug_image_output(name: &str, grid: Grid, fragment_grid: &mut VecOnGrid<Fragment>) {
    let side_length = fragment_grid.iter().next().unwrap().side_length();

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

    let mut data = vec![];

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                data.extend(
                    fragment_grid[(grid.pos(x, y))].pixels()
                        [(py * side_length) as usize..((py + 1) * side_length) as usize]
                        .iter()
                        .flat_map(|x| [x.r, x.g, x.b]),
                );
            }
        }
    }

    writer.write_image_data(&data).unwrap();
}

fn biggest_case() -> Problem {
    const ROWS: u8 = 16;
    const COLS: u8 = ROWS;

    let decoder = png::Decoder::new(Cursor::new(include_bytes!("../problem.png")));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    buf.truncate(info.buffer_size());

    let buf = buf
        .chunks(3)
        .map(|a| Color {
            r: a[0],
            g: a[1],
            b: a[2],
        })
        .collect::<Vec<_>>();

    let source = Problem {
        select_limit: 0,
        select_cost: 0,
        swap_cost: 0,
        rows: ROWS,
        cols: COLS,
        image: Image {
            width: info.width as _,
            height: info.height as _,
            pixels: buf,
        },
    };

    let mut fragments = fragment::Fragment::new_all(&source);

    // fixed rng for stabilize test results
    let mut rng = StdRng::seed_from_u64(0);

    let grid = Grid::new(ROWS, COLS);
    let mut fragment_grid = VecOnGrid::with_default(grid);

    for (pos, cell) in fragment_grid.iter_mut_with_pos() {
        let index = rng.gen_range(0..fragments.len());

        let rot = if pos == grid.pos(0, 0) {
            Rot::R0
        } else {
            Rot::from_num(rng.gen_range(0..4))
        };

        let mut fragment = fragments.remove(index);
        fragment.rotate(rot);
        fragment.apply_rotate();
        fragment.pos = pos;

        *cell = Some(fragment);
    }

    println!("shuffle complete");

    let side_length = (info.width / COLS as u32) as usize;
    let zeros = vec![Color { r: 0, g: 0, b: 0 }; side_length];
    let mut data = vec![];

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                if let Some(ref mut x) = &mut fragment_grid[(grid.pos(x, y))] {
                    let pixels =
                        &x.pixels()[(py * side_length) as usize..((py + 1) * side_length) as usize];
                    data.extend_from_slice(pixels);
                } else {
                    data.extend_from_slice(&zeros);
                }
            }
        }
    }

    Problem {
        select_limit: 0,
        select_cost: 0,
        swap_cost: 0,
        rows: ROWS,
        cols: COLS,
        image: Image {
            width: info.width as _,
            height: info.height as _,
            pixels: data,
        },
    }
}
