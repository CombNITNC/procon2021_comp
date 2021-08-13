use {
    crate::basis::Color,
    anyhow::{ensure, Context as _, Result},
    std::{
        error::Error,
        io::{BufRead, Read},
        str::FromStr,
    },
};

pub(crate) struct Problem {
    selectable_count: u8,
    selection_cost_convert_rate: u16,
    swap_cost_convert_rate: u8,

    horizontal_split_count: u8,
    vertical_split_count: u8,

    image: Image,
}

impl Problem {
    pub(crate) fn read(mut data: impl BufRead) -> Result<Self> {
        let nl = &mut || {
            let mut buf = String::new();
            data.read_line(&mut buf).context("failed to read line")?;
            Ok(buf)
        };

        let magic = Self::parse_line(nl, |x| Ok(x.to_string()), "magic number")?;
        ensure!(magic == "P6", "expected magic number, but found {}", magic);

        let (horizontal_split_count, vertical_split_count) =
            Self::parse_line(nl, Self::parse_split_count, "split count")?;

        let selectable_count =
            Self::parse_line(nl, Self::parse_selectable_count, "selectable count")?;

        let (selection_cost_convert_rate, swap_cost_convert_rate) =
            Self::parse_line(nl, Self::parse_cost, "cost convert rate")?;

        let (width, height) = Self::parse_line(nl, Self::parse_dim, "image dimensions")?;
        let _max_color_value = Self::parse_line(nl, Self::parse_max_color_value, "max color value");

        Ok(Self {
            selectable_count,
            selection_cost_convert_rate,
            swap_cost_convert_rate,
            horizontal_split_count,
            vertical_split_count,
            image: Image::read(data, width, height)?,
        })
    }

    fn parse_line<O, P, R>(next_line: &mut R, parser: P, expect: &str) -> Result<O>
    where
        O: 'static,
        R: FnMut() -> Result<String>,
        P: Fn(&str) -> Result<O>,
    {
        let line = next_line().with_context(|| format!("failed to read {}", expect))?;
        parser(&line)
            .with_context(|| format!("failed to parse {} line. raw line: {}", expect, line))
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
        let h = Self::parse_token::<_, u8>(tokens, "horizontal split count")?;
        let v = Self::parse_token::<_, u8>(tokens, "vertical split count")?;

        Ok((h, v))
    }

    fn parse_selectable_count(line: &str) -> Result<u8> {
        let tokens = &mut line.split_ascii_whitespace();

        ensure!(tokens.next() == Some("#"), "expected comment line");
        Self::parse_token(tokens, "selectable count")
    }

    // returns: (select, swap)
    fn parse_cost(line: &str) -> Result<(u16, u8)> {
        let mut tokens = line.split_ascii_whitespace();

        ensure!(tokens.next() == Some("#"), "expected comment line");
        let select = Self::parse_token(&mut tokens, "selection cost convert rate")?;
        let swap = Self::parse_token(&mut tokens, "swap cost convert rate")?;

        Ok((select, swap))
    }

    fn parse_dim(line: &str) -> Result<(u16, u16)> {
        let tokens = &mut line.split_ascii_whitespace();
        let w = Self::parse_token(tokens, "image width")?;
        let h = Self::parse_token(tokens, "image height")?;

        Ok((w, h))
    }

    fn parse_max_color_value(line: &str) -> Result<u16> {
        let value = Self::parse_token(&mut line.split_ascii_whitespace(), "max color value")?;

        // 問題文の指定から、24ビット画像のはず
        ensure!(
            value <= (u8::MAX as _),
            "color value is unexpectedly big. (doesn't fit to u8)"
        );

        Ok(value)
    }
}

pub(crate) struct Image {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) data: Vec<Vec<Color>>,
}

impl Image {
    // http://netpbm.sourceforge.net/doc/ppm.html
    fn read(data: impl Read, width: u16, height: u16) -> Result<Self> {
        let mut bytes_iter = data.bytes().skip_while(|x| {
            if let Ok(x) = x {
                (*x as char).is_ascii_whitespace()
            } else {
                false
            }
        });

        let mut image_data = Vec::with_capacity(height as _);
        image_data.push(Vec::with_capacity(width as _));

        let mut r = None;
        let mut g = None;
        let mut b = None;

        while let Some(byte_result) = bytes_iter.next() {
            let byte = byte_result.context("failed to read image body byte")?;

            match (r, g, b) {
                (None, None, None) => r = Some(byte),
                (Some(_), None, None) => g = Some(byte),
                (Some(_), Some(_), None) => b = Some(byte),
                (Some(ar), Some(ag), Some(ab)) => {
                    let last = image_data.last_mut().unwrap();
                    last.push(Color {
                        r: ar,
                        g: ag,
                        b: ab,
                    });

                    r = None;
                    g = None;
                    b = None;

                    if last.len() == width as _ {
                        image_data.push(Vec::with_capacity(width as _));
                    }
                }
                _ => unreachable!(),
            }
        }

        assert!(r.is_none() && g.is_none() && b.is_none());
        assert!(image_data.last().unwrap().len() == width as _);

        Ok(Image {
            width,
            height,
            data: image_data,
        })
    }
}
