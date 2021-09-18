#![allow(dead_code)]

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    time::Duration,
};

use png::{BitDepth, ColorType, Compression, Encoder};
use sdl2::{
    keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, render::TextureQuery, surface::Surface,
};

mod basis;
mod fragment;
mod grid;
mod image;
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
    let fragments = fragment::Fragment::new_all(&problem);
    let side_length = fragments[0].side_length();
    let mut recovered_image = pixel_match::resolve(fragments, grid);
    debug_image_output("recovered_image.png", grid, &mut recovered_image);

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let ttf = sdl2::ttf::init().unwrap();

    let font = ttf.load_font("./mplus-1m-medium.ttf", 128).unwrap();

    let mut canvas = video
        .window("procon2021_comp", 800, 800)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap()
        .into_canvas()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    use sdl2::pixels::Color as SdlColor;

    let recovered_image_texture = {
        let width = (side_length * grid.width() as usize) as u32;
        let height = (side_length * grid.height() as usize) as u32;
        let mut surface = Surface::new(width, height, PixelFormatEnum::RGB24).unwrap();

        let zeros = vec![0; side_length * 3];
        let mut data = vec![];

        for y in 0..grid.height() {
            for py in 0..side_length {
                for x in 0..grid.width() {
                    if let Some(ref mut x) = &mut recovered_image[(grid.pos(x, y))] {
                        let pixels = x.pixels()
                            [(py * side_length) as usize..((py + 1) * side_length) as usize]
                            .iter()
                            .flat_map(|x| [x.r, x.g, x.b]);
                        data.extend(pixels);
                    } else {
                        data.extend_from_slice(&zeros);
                    }
                }
            }
        }

        surface.with_lock_mut(|x| x.copy_from_slice(&data));
        texture_creator
            .create_texture_from_surface(surface)
            .unwrap()
    };

    let text_texture = {
        let surface = font
            .render("Hello Rust!")
            .blended(SdlColor::RGBA(0, 0, 0, 0))
            .unwrap();

        texture_creator
            .create_texture_from_surface(&surface)
            .unwrap()
    };

    let TextureQuery {
        width: text_width,
        height: text_height,
        ..
    } = text_texture.query();

    'mainloop: loop {
        for event in sdl.event_pump().unwrap().poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'mainloop;
                }

                _ => {}
            }
        }

        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 0));
        canvas.clear();

        canvas.copy(&recovered_image_texture, None, None).unwrap();
        canvas
            .copy(
                &text_texture,
                None,
                Rect::new(0, 0, text_width, text_height),
            )
            .unwrap();
        canvas.present();

        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }
}

fn debug_image_output(name: &str, grid: Grid, fragment_grid: &mut VecOnGrid<Option<Fragment>>) {
    let side_length = fragment_grid
        .iter()
        .next()
        .unwrap()
        .as_ref()
        .unwrap()
        .side_length();

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

    let zeros = vec![0; side_length * 3];
    let mut data = vec![];

    for y in 0..grid.height() {
        for py in 0..side_length {
            for x in 0..grid.width() {
                if let Some(ref mut x) = &mut fragment_grid[(grid.pos(x, y))] {
                    let pixels = x.pixels()
                        [(py * side_length) as usize..((py + 1) * side_length) as usize]
                        .iter()
                        .flat_map(|x| [x.r, x.g, x.b]);
                    data.extend(pixels);
                } else {
                    data.extend_from_slice(&zeros);
                }
            }
        }
    }

    writer.write_image_data(&data).unwrap();
}
