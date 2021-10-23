use {
    crate::basis::{Color, Image, Problem},
    anyhow::{ensure, Context as _, Result},
    std::{
        error::Error,
        io::{BufRead, Read},
        str::FromStr,
    },
};

pub fn read_problem(mut data: impl BufRead) -> Result<Problem> {
    let nl = &mut || {
        let mut buf = String::new();
        data.read_line(&mut buf).context("failed to read line")?;
        Ok(buf.trim().to_string())
    };

    let magic = parse_line(nl, |x| Ok(x.to_string()), "magic number")?;
    ensure!(magic == "P6", "expected magic number, but found {}", magic);

    let (horizontal_split_count, vertical_split_count) =
        parse_line(nl, parse_split_count, "split count")?;

    let selectable_count = parse_line(nl, parse_selectable_count, "selectable count")?;

    let (selection_cost_convert_rate, swap_cost_convert_rate) =
        parse_line(nl, parse_cost, "cost convert rate")?;

    let (width, height) = parse_line(nl, parse_dim, "image dimensions")?;
    let _max_color_value = parse_line(nl, parse_max_color_value, "max color value");

    let image = read_image(data, width, height).context("failed to read image")?;

    Ok(Problem {
        select_limit: selectable_count,
        select_cost: selection_cost_convert_rate,
        swap_cost: swap_cost_convert_rate,
        rows: horizontal_split_count,
        cols: vertical_split_count,
        image,
    })
}

fn parse_line<O, P, R>(next_line: &mut R, parser: P, expect: &str) -> Result<O>
where
    O: 'static,
    R: FnMut() -> Result<String>,
    P: Fn(&str) -> Result<O>,
{
    let line = next_line().with_context(|| format!("failed to read {}", expect))?;
    parser(&line).with_context(|| format!("failed to parse {} line. raw line: {}", expect, line))
}

fn parse_token<'i, I, T>(iter: &mut I, expect: &str) -> Result<T>
where
    I: Iterator<Item = &'i str>,
    T: FromStr,
    T::Err: Send + Sync + Error + 'static, // because of anyhow::Context requirements
{
    iter.next()
        .with_context(|| format!("expected {}, but no next token found", expect))?
        .parse()
        .with_context(|| format!("failed to parse {}", expect))
}

// returns: (horizontal, vertical)
fn parse_split_count(line: &str) -> Result<(u8, u8)> {
    let tokens = &mut line.split_ascii_whitespace();

    ensure!(tokens.next() == Some("#"), "expected comment line");
    let h = parse_token::<_, u8>(tokens, "horizontal split count")?;
    let v = parse_token::<_, u8>(tokens, "vertical split count")?;

    Ok((h, v))
}

fn parse_selectable_count(line: &str) -> Result<u8> {
    let tokens = &mut line.split_ascii_whitespace();

    ensure!(tokens.next() == Some("#"), "expected comment line");
    parse_token(tokens, "selectable count")
}

// returns: (select, swap)
fn parse_cost(line: &str) -> Result<(u16, u16)> {
    let mut tokens = line.split_ascii_whitespace();

    ensure!(tokens.next() == Some("#"), "expected comment line");
    let select = parse_token(&mut tokens, "selection cost convert rate")?;
    let swap = parse_token(&mut tokens, "swap cost convert rate")?;

    Ok((select, swap))
}

fn parse_dim(line: &str) -> Result<(u16, u16)> {
    let tokens = &mut line.split_ascii_whitespace();
    let w = parse_token(tokens, "image width")?;
    let h = parse_token(tokens, "image height")?;

    Ok((w, h))
}

fn parse_max_color_value(line: &str) -> Result<u16> {
    let value = parse_token(&mut line.split_ascii_whitespace(), "max color value")?;

    // 問題文の指定から、24ビット画像のはず
    ensure!(
        value <= (u8::MAX as _),
        "max color value is unexpectedly big. (doesn't fit to u8)"
    );

    Ok(value)
}

// http://netpbm.sourceforge.net/doc/ppm.html
fn read_image(data: impl Read, width: u16, height: u16) -> Result<Image> {
    let bytes_iter = data.bytes();

    let mut image_data = Vec::with_capacity(width as usize * height as usize);

    let mut r = None;
    let mut g = None;

    for byte_result in bytes_iter {
        let byte = byte_result.context("failed to read image body byte")?;

        match (r, g) {
            (None, None) => r = Some(byte),
            (Some(_), None) => g = Some(byte),
            (Some(ar), Some(ag)) => {
                image_data.push(Color {
                    r: ar,
                    g: ag,
                    b: byte,
                });

                r = None;
                g = None;
            }
            _ => unreachable!(),
        }
    }

    ensure!(
        r.is_none() && g.is_none(),
        "there were trailing bytes (rg buffers are not none)"
    );

    ensure!(
        image_data.len() == width as usize * height as usize,
        "image pixel count mismatch"
    );

    Ok(Image {
        width,
        height,
        pixels: image_data,
    })
}

#[test]
fn problem_read_test() {
    let problem = include_bytes!("../test_cases/01_q.ppm");
    let reader = std::io::BufReader::new(problem.as_ref());
    let result = read_problem(reader).unwrap();
    assert_eq!(result.select_limit, 1);
    assert_eq!(result.select_cost, 3);
    assert_eq!(result.swap_cost, 1);
    assert_eq!(result.rows, 2);
    assert_eq!(result.cols, 2);
}
