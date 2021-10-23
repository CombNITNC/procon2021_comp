use std::{
    fs::File,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use procon2021_comp::{
    fragment, grid::Grid, image, kaitou, move_resolve, move_resolve::ResolveParam, pixel_match,
};

#[cfg(not(feature = "net"))]
compile_error!("The `net` feature is required for main");

fn main() {
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

        submit(answer, &token, &endpoint);
    });
}

fn submit(answer: String, token: &str, endpoint: &str) {
    println!("submitting");
    let submit_result = submit::submit(endpoint, token, answer);
    println!("submit result: {:#?}", submit_result);
}

mod fetch {
    pub fn fetch_ppm(endpoint: &str) -> reqwest::Result<bytes::Bytes> {
        let mut endpoint = endpoint.to_owned();
        endpoint.push_str("/problem.ppm");
        reqwest::blocking::get(endpoint).and_then(|x| x.bytes())
    }
}
mod submit {
    use anyhow::{bail, ensure, Context as _, Result};

    #[derive(Debug)]
    pub struct SubmitResult {
        pub pos_mismatch_count: usize,
        pub rot_mismatch_count: usize,
        pub request_id: Option<String>,
    }

    pub fn submit(endpoint: &str, token: &str, answer: String) -> Result<SubmitResult> {
        let res = reqwest::blocking::Client::builder()
            .build()
            .context("failed to build reqwest client")?
            .post(endpoint)
            .header("procon-token", token)
            .body(answer)
            .send()
            .context("failed to send answer to procon server")?;

        if !res.status().is_success() {
            bail!("PORT request failed. {:#?}", res);
        }

        let request_id = res
            .headers()
            .get("procon-request-id")
            .map(|x| x.to_str().unwrap().to_string());

        let body = res.text().context("failed to decode body")?;

        let (pos_mismatch_count, rot_mismatch_count) = parse_post_response(&body)
            .with_context(|| format!("failed to parse body. raw: '{}'", body))?;

        Ok(SubmitResult {
            pos_mismatch_count,
            rot_mismatch_count,
            request_id,
        })
    }

    fn parse_post_response(body: &str) -> Result<(usize, usize)> {
        let mut body_tokens = body.split_ascii_whitespace();

        ensure!(
            body_tokens.next() == Some("ACCEPTED"),
            "excepted 'ACCEPTED'"
        );

        let pos = body_tokens
            .next()
            .context("excepted pos_mismatch_count")?
            .parse()
            .context("failed to parse pos_mismatch_count")?;

        let rot = body_tokens
            .next()
            .context("excepted rot_mismatch_count")?
            .parse()
            .context("failed to parse rot_mismatch_count")?;

        Ok((pos, rot))
    }

    #[test]
    fn test_parse_post_response() {
        assert_eq!(parse_post_response("ACCEPTED 2 3").unwrap(), (2, 3));
        assert_eq!(parse_post_response("ACCEPTED 04 23").unwrap(), (4, 23));
        assert_eq!(parse_post_response("HOGE FUGA").ok(), None);
    }
}
