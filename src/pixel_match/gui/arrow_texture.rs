use sdl2::{
    pixels::PixelFormatEnum,
    render::{Texture, TextureCreator},
    surface::Surface,
    video::WindowContext,
};

pub(super) fn arrow_texture(creator: &TextureCreator<WindowContext>) -> Texture {
    let bitmap = include_str!("./arrow.ascii");

    let mut surface = Surface::new(13, 13, PixelFormatEnum::RGB888).unwrap();
    surface.with_lock_mut(|surface_data| {
        for (i, c) in bitmap.chars().filter(|&x| x == '.' || x == '#').enumerate() {
            let mut write = |r, g, b| {
                let i = i * 4;
                surface_data[i] = r;
                surface_data[i + 1] = g;
                surface_data[i + 2] = b;
            };

            match c {
                '.' => write(0, 0, 0),
                '#' => write(0, 255, 0),
                _ => unreachable!(),
            }
        }
    });

    creator.create_texture_from_surface(surface).unwrap()
}
