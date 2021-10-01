use std::io;

pub(crate) fn fetch_ppm(endpoint: &str) -> reqwest::Result<impl io::BufRead> {
    let mut endpoint = endpoint.to_owned();
    endpoint.push_str("/problem.ppm");
    reqwest::blocking::get(endpoint).map(io::BufReader::new)
}
