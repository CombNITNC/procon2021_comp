pub(crate) fn fetch_ppm(endpoint: &str) -> reqwest::Result<bytes::Bytes> {
    let mut endpoint = endpoint.to_owned();
    endpoint.push_str("/problem.ppm");
    reqwest::blocking::get(endpoint).and_then(|x| x.bytes())
}
