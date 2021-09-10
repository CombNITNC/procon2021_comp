use {
    crate::{
        basis::{Color, Dir, Rot},
        fragment::Fragment,
        grid::Grid,
    },
    std::io::{self, Read, Result},
};

fn pixels(to_skip: u64, width: usize, height: usize, path: &str) -> Result<Vec<Color>> {
    let mut img = std::fs::File::open(path)?;
    io::copy(&mut img.by_ref().take(to_skip), &mut io::sink())?;
    let mut pixel_components = vec![0; width * height * 3];
    img.read(&mut pixel_components)?;
    Ok(pixel_components
        .chunks(3)
        .map(|comps| Color {
            r: comps[0],
            g: comps[1],
            b: comps[2],
        })
        .collect())
}

#[test]
fn case1() -> Result<()> {
    let width = 180;
    let frag_edge = 60;
    let rows = 3usize;
    let cols = 2usize;
    let pixels = pixels(32, width, 120, "test_cases/02_sampled.ppm")?;
    let grid = Grid::new(rows as u8, cols as u8);

    for y in 0..cols {
        for x in 0..rows {
            let pos = grid.pos(x as u8, y as u8);
            let frag = Fragment::new(&pixels, pos.clone(), width, frag_edge as u16);

            let up_left = x * frag_edge + y * frag_edge * width;

            let edge = frag.edges.edge(Dir::North);
            assert!(matches!(edge.dir, Dir::North));
            for i in 0..60 {
                assert_eq!(&edge.pixels[i], &pixels[up_left + i]);
            }

            let edge = frag.edges.edge(Dir::East);
            assert!(matches!(edge.dir, Dir::East));
            for i in 0..60 {
                assert_eq!(
                    &edge.pixels[i],
                    &pixels[up_left + (frag_edge - 1) + i * width]
                );
            }

            let edge = frag.edges.edge(Dir::South);
            assert!(matches!(edge.dir, Dir::South));
            for i in 0..60 {
                assert_eq!(
                    &edge.pixels[i],
                    &pixels[up_left + (frag_edge - 1) * width + (frag_edge - 1) - i]
                );
            }

            let edge = frag.edges.edge(Dir::West);
            assert!(matches!(edge.dir, Dir::West));
            for i in 0..60 {
                assert_eq!(
                    &edge.pixels[i],
                    &pixels[up_left + (frag_edge - 1) * width - i * width]
                );
            }

            assert!(matches!(frag.rot, Rot::R0));
        }
    }
    Ok(())
}
