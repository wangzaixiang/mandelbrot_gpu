mod cpu;
mod cpu_par;
mod gpu;

fn timeit<T, F>(label: &str, f: F) -> T where F: FnOnce()->T  {
    let start = std::time::Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    println!("{} Elapsed: {}.{:03} seconds", label, elapsed.as_secs(), elapsed.subsec_millis());
    result
}

use std::fs::File;
use std::str::FromStr;
use image::ColorType;
use image::png::PngEncoder;
use num::Complex;
use crate::cpu::parse_pair;

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);
    encoder.encode(&pixels,
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::L8).expect("PNG encoder error");

    Ok(())
}



fn parse_complex(s: &str) -> Option<Complex<f32>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

pub fn main(){
    let args : Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        eprintln!( "Usage: {} cpu|cpu_par|gpu FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);
        eprintln!("Example: {} cpu output.png 300x200 -1.20,0.35 -1,0.20", args[0]);
        eprintln!("Example: {} cpu output.png 4096x2560 -1.20,0.35 -1,0.20", args[0]);
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[3], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[4])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[5])
        .expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];
    timeit(&format!("render by {}", args[1].as_str()), || {
        match args[1].as_str() {
            "cpu" => cpu::render(&mut pixels, bounds, upper_left, lower_right),
            "cpu_par" => cpu_par::render(&mut pixels, bounds, upper_left, lower_right),
            "gpu" => gpu::render(&mut pixels, bounds, upper_left, lower_right),
            _ => cpu::render(&mut pixels, bounds, upper_left, lower_right)
        }
    });

    write_image(&args[2], &pixels, bounds).expect("error writing PNG file");

    println!("open the file {} to view the mandelbrot image", &args[1]);
}