#![allow(dead_code)]

use std::{
    fs::File,
    io::Write,
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

        let problem = image::read_problem(data.slice(..).reader()).unwrap();

        use bytes::Buf;

        let filename = format!("problem-{}.ppm", epoch);
        File::create(&filename).unwrap().write_all(&data).unwrap();
        println!("saved the problem to {}", filename);

        problem
    };
    println!("problem case: {:?}", problem);

    let grid = Grid::new(problem.rows, problem.cols);
    let fragments = fragment::Fragment::new_all(&problem);

    let recovered_image = pixel_match::resolve(fragments, grid);
    let rots = recovered_image.iter().map(|x| x.rot).collect::<Vec<_>>();
    println!("pixel_match::resolve() done");

    let movements = fragment::map_fragment::map_fragment(&recovered_image);

    for threshold in 2..=5 {
        let ops = move_resolve::resolve_approximately(
            grid,
            &movements,
            problem.select_limit,
            problem.swap_cost,
            problem.select_cost,
            threshold,
        );

        println!(
            "move_resolve::resolve_approx() done (threshold: {})",
            threshold
        );

        let answer = kaitou::ans(&ops, &rots);

        #[cfg(feature = "net")]
        submit(answer, &token, &endpoint);
        #[cfg(not(feature = "net"))]
        submit(
            answer,
            &format!("answer-{}-approx-{}.txt", epoch, threshold),
        );
    }

    let ops = move_resolve::resolve(
        grid,
        &movements,
        problem.select_limit,
        problem.swap_cost,
        problem.select_cost,
    );
    println!("move_resolve::resolve() done");

    let answer = kaitou::ans(&ops, &rots);

    #[cfg(feature = "net")]
    submit(answer, &token, &endpoint);
    #[cfg(not(feature = "net"))]
    submit(answer, &format!("answer-{}.txt", epoch));
}

#[cfg(feature = "net")]
fn submit(answer: String, token: &str, endpoint: &str) {
    println!("submitting");
    let submit_result = submit::submit(&endpoint, &token, answer);
    println!("submit result: {:#?}", submit_result);
}

#[cfg(not(feature = "net"))]
fn submit(answer: String, filename: &str) {
    File::create(filename)
        .unwrap()
        .write_all(answer.as_bytes())
        .unwrap();
    println!("saved answer to {}", filename);
}
