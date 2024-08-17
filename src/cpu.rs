use std::str::FromStr;
use num::Complex;

fn escape_time(c: Complex<f32>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        let norm = z.norm_sqr();
        if norm > 4.0 {
            return Some(i);
        }
    }
    None
}

pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None,
            }
        }
    }
}
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}


pub fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f32>,
                  lower_right: Complex<f32>)
                  -> Complex<f32> {
    let (width, height) = (lower_right.re - upper_left.re,
                           upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f32 * width / bounds.0 as f32,
        im: upper_left.im - pixel.1 as f32 * height / bounds.1 as f32,
    }
}

pub fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: Complex<f32>,
          lower_right: Complex<f32>) {
    assert_eq!(pixels.len(), bounds.0 * bounds.1);
    for r in 0..bounds.1 {
        for c in 0..bounds.0 {
            let point = pixel_to_point(bounds, (c, r), upper_left, lower_right);
            pixels[r * bounds.0 + c] =
                match escape_time(point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}



