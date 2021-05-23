use {
    crate::basis::Color,
    anyhow::{bail, Result},
    std::io::Read,
};

pub(crate) struct Image {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) data: Vec<Vec<Color>>,
}

impl Image {
    // http://netpbm.sourceforge.net/doc/ppm.html
    pub(crate) fn read_ppm(data: impl Read) -> Result<Self> {
        let mut bytes_iter = data.bytes().enumerate();

        let mut next_token = || {
            let mut buffer = String::new();

            let mut last_index = 0;

            while let Some((index, Ok(byte))) = bytes_iter.next() {
                let byte = byte as char;
                assert!(byte.is_ascii());

                // skip trailling whitespaces
                if last_index == 0 && byte.is_whitespace() {
                    continue;
                }

                last_index = index;

                if byte.is_whitespace() {
                    break;
                }

                buffer.push(byte);
            }

            if buffer.is_empty() {
                None
            } else {
                assert!(last_index != 0);
                Some((last_index, buffer))
            }
        };

        // check magic number
        match next_token() {
            Some((_, m)) if m == "P6" => {}
            t => bail!("expected magic number \"P6\", but found {:?}", t),
        };

        let width: u16 = match next_token().map(|(_, s)| s.parse()) {
            Some(Ok(w)) => w,
            Some(Err(e)) => bail!("failed to parse width: {:?}", e),
            None => bail!("expected width, but found none"),
        };

        let height: u16 = match next_token().map(|(_, s)| s.parse()) {
            Some(Ok(h)) => h,
            Some(Err(e)) => bail!("failed to parse height: {:?}", e),
            None => bail!("expected maximum value of color, but found none"),
        };

        let max_color_value: u16 = match next_token().map(|(_, s)| s.parse()) {
            Some(Ok(c)) => c,
            Some(Err(e)) => bail!("failed to parse max_color_value {:?}", e),
            None => bail!("expected maximum value of color, but found none"),
        };

        assert!(max_color_value <= 255);

        let mut image_data = Vec::with_capacity(height as _);
        let mut r = None;
        let mut g = None;
        let mut b = None;

        while let Some((_, Ok(byte))) = bytes_iter.next() {
            match (r, g, b) {
                (None, None, None) => r = Some(byte),
                (Some(_), None, None) => g = Some(byte),
                (Some(_), Some(_), None) => b = Some(byte),
                (Some(ar), Some(ag), Some(ab)) => {
                    if image_data.is_empty() {
                        image_data.push(Vec::with_capacity(width as _));
                    }

                    image_data.last_mut().unwrap().push(Color {
                        r: ar,
                        g: ag,
                        b: ab,
                    });

                    r = None;
                    g = None;
                    b = None;

                    if image_data.last().unwrap().len() == width as _ {
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
