use std::{
    fs::File,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use procon2021_comp::{
    fragment, grid::Grid, image, kaitou, move_resolve, move_resolve::ResolveParam, pixel_match,
};

fn main() {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let problem = {
        let path = std::env::args()
            .next()
            .expect("the problem file must be given");
        let file = File::open(path).expect("failed to open problem file");
        let reader = std::io::BufReader::new(file);
        image::read_problem(reader).unwrap()
    };

    println!("problem case: {:?}", problem);

    let grid = Grid::new(problem.rows, problem.cols);
    let fragments = fragment::Fragment::new_all(&problem);

    let recovered_image = pixel_match::resolve(fragments, grid);
    let rots = recovered_image.iter().map(|x| x.rot).collect::<Vec<_>>();
    println!("pixel_match::resolve() done");

    let movements = fragment::map_fragment::map_fragment(&recovered_image);

    let operations_candidate = move_resolve::resolve(
        grid,
        &movements,
        ResolveParam {
            select_limit: problem.select_limit,
            swap_cost: problem.swap_cost,
            select_cost: problem.select_cost,
        },
    );
    println!("move_resolve::resolve() done");

    operations_candidate.for_each(|ops| {
        let answer = kaitou::ans(&ops, &rots);

        submit(answer, &format!("answer-{}.txt", epoch));
    });
}

fn submit(answer: String, filename: &str) {
    File::create(filename)
        .unwrap()
        .write_all(answer.as_bytes())
        .unwrap();
    println!("saved answer to {}", filename);
}
