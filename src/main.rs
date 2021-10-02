#![allow(dead_code)]

use std::{
    fs::File,
    time::{SystemTime, UNIX_EPOCH},
};

mod basis;
mod fragment;
mod grid;
mod image;
mod kaitou;
mod move_resolve;
mod pixel_match;

#[cfg(feature = "net")]
mod fetch;
#[cfg(feature = "net")]
mod submit;

use crate::grid::Grid;

fn main() {
    #[cfg(feature = "net")]
    let (token, endpoint) = {
        dotenv::dotenv().ok();
        (
            std::env::var("TOKEN").expect("set TOKEN environment variable for auto submit"),
            std::env::var("SERVER_ENDPOINT")
                .expect("set SERVER_ENDPOINT environment variable for auto submit"),
        )
    };

    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    #[cfg(not(feature = "net"))]
    let problem = {
        let file = File::open("problem.ppm").expect("failed to open problem file");
        let reader = std::io::BufReader::new(file);
        image::read_problem(reader).unwrap()
    };

    #[cfg(feature = "net")]
    let problem = {
        let data = fetch::fetch_ppm(&endpoint).unwrap();
        println!("fetch::fetch_ppm() done");

        use bytes::Buf;
        use std::io::Write;

        let filename = format!("problem-{}.ppm", epoch);
        File::create(&filename).unwrap().write_all(&data).unwrap();
        println!("saved the problem to {}", filename);

        image::read_problem(data.reader()).unwrap()
    };

    let grid = Grid::new(problem.rows, problem.cols);
    let fragments = fragment::Fragment::new_all(&problem);

    let recovered_image = pixel_match::resolve(fragments, grid);
    println!("pixel_match::resolve() done");

    let movements = fragment::map_fragment::map_fragment(&recovered_image);

    let ops = move_resolve::resolve(
        grid,
        &movements,
        problem.select_limit,
        problem.swap_cost,
        problem.select_cost,
    );
    println!("move_resolve::resolve() done");

    let rots = recovered_image.iter().map(|x| x.rot).collect::<Vec<_>>();
    let answer = kaitou::ans(&ops, &rots);

    #[cfg(feature = "net")]
    {
        println!("submitting");
        let submit_result = submit::submit(&endpoint, &token, answer);
        println!("submit result: {:#?}", submit_result);
    }
}
