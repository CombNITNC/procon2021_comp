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

use crate::{
    grid::Grid,
    move_resolve::{approx::gen::FromOutside, ResolveParam},
};

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
    let mut min_cost = 20000;

    for threshold_x in 2..=4 {
        for threshold_y in 2..=4 {
            let result = move_resolve::resolve_approximately(
                grid,
                &movements,
                ResolveParam {
                    select_limit: problem.select_limit,
                    swap_cost: problem.swap_cost,
                    select_cost: problem.select_cost,
                },
                (threshold_x, threshold_y),
                min_cost,
                FromOutside,
            );
            if result.is_none() {
                println!(
                    "move_resolve::resolve_approx() none (threshold: {}-{})",
                    threshold_x, threshold_y
                );
                println!();

                continue;
            }
            let (ops, cost) = result.unwrap();

            println!(
                "move_resolve::resolve_approx() done (threshold: {}-{})",
                threshold_x, threshold_y
            );

            if cost < min_cost {
                min_cost = cost;
                println!("best cost. submitting");
                let answer = kaitou::ans(&ops, &rots);

                #[cfg(feature = "net")]
                submit(answer, &token, &endpoint);
                #[cfg(not(feature = "net"))]
                submit(
                    answer,
                    &format!(
                        "answer-{}-approx-{}-{}.txt",
                        epoch, threshold_x, threshold_y
                    ),
                );

                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            println!();
        }
    }

    println!("finding best score");
    let ops = move_resolve::resolve(
        grid,
        &movements,
        ResolveParam {
            select_limit: problem.select_limit,
            swap_cost: problem.swap_cost,
            select_cost: problem.select_cost,
        },
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
    let submit_result = submit::submit(endpoint, token, answer);
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
