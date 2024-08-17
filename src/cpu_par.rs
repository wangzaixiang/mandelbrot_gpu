use num::Complex;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use crate::cpu::{pixel_to_point};

pub fn render(pixes: &mut [u8],
                       bounds: (usize, usize),
                       upper_left: Complex<f32>,
                       lower_right: Complex<f32>)  {
    let bands: Vec<(usize, &mut [u8])> = pixes.chunks_mut(bounds.0)
        .enumerate().collect();

    bands.into_par_iter()
        .for_each( |(i, band)| {
            let top = i;
            let band_bounds = (bounds.0, 1);
            let band_upper_left = pixel_to_point(bounds, (0, top),
                                                 upper_left, lower_right);
            let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1),
                                                  upper_left, lower_right);
            crate::cpu::render(band, band_bounds, band_upper_left, band_lower_right);
        } );
}
