use anyhow::{bail, ensure, Context as _, Result};

#[derive(Debug)]
pub(crate) struct SubmitResult {
    pub(crate) pos_mismatch_count: usize,
    pub(crate) rot_mismatch_count: usize,
    pub(crate) request_id: Option<String>,
}

const ENDPOINT: &str = "https://proco32-practice.kosen.work";

pub(crate) fn submit(token: &str, answer: &str) -> Result<SubmitResult> {
    let res = reqwest::blocking::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .post(ENDPOINT)
        .header("procon-token", token)
        .body(answer.to_string())
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
