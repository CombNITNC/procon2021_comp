use std::{env, io};

pub(crate) fn fetch_ppm() -> reqwest::Result<impl io::BufRead> {
    let mut endpoint =
        env::var("ENDPOINT").expect("ENDPOINT var needed to fetch the problem ppm image");
    endpoint.push_str("/problem.ppm");
    reqwest::blocking::get(endpoint).map(io::BufReader::new)
}
