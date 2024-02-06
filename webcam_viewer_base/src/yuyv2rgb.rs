//! Source:
//! https://gist.github.com/arifd/ea820ec97265a023e67a88b66955855d
#![allow(dead_code)]

use ndarray::{array, Array2};
use rayon::prelude::*;

/// Copies an input buffer of format YUYV422 to the output buffer
/// in the format of RGB24
#[inline]
pub fn yuv422_to_rgb24(in_buf: &[u8], out_buf: &mut [u8]) {
    debug_assert!(out_buf.len() as f32 == in_buf.len() as f32 * 1.5);

    in_buf
        .par_chunks_exact(4) // FIXME: use par_array_chunks() when stabalized (https://github.com/rayon-rs/rayon/pull/789)
        .zip(out_buf.par_chunks_exact_mut(6))
        .for_each(|(ch, out)| {
            let y1 = ch[0];
            let y2 = ch[2];
            let cb = ch[1];
            let cr = ch[3];

            let (r, g, b) = ycbcr_to_rgb(y1, cb, cr);

            out[0] = r;
            out[1] = g;
            out[2] = b;

            let (r, g, b) = ycbcr_to_rgb(y2, cb, cr);

            out[3] = r;
            out[4] = g;
            out[5] = b;
        });
}

#[inline]
pub fn yuv422_to_rgb32(in_buf: &[u8], out_buf: &mut [u8]) {
    debug_assert!(out_buf.len() == in_buf.len() * 2);

    in_buf
        .par_chunks_exact(4) // FIXME: use par_array_chunks() when stabalized (https://github.com/rayon-rs/rayon/pull/789)
        .zip(out_buf.par_chunks_exact_mut(8))
        .for_each(|(ch, out)| {
            let y1 = ch[0];
            let y2 = ch[2];
            let cb = ch[1];
            let cr = ch[3];

            let (r, g, b) = ycbcr_to_rgb(y1, cb, cr);

            out[0] = b;
            out[1] = g;
            out[2] = r;
            // out[3] = 0;

            let (r, g, b) = ycbcr_to_rgb(y2, cb, cr);

            out[4] = b;
            out[5] = g;
            out[6] = r;
            // out[7] = 0;
        });
}

#[inline]
fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let ycbcr = array![y as f32, cb as f32 - 128.0f32, cr as f32 - 128.0f32, 0.0];
    let conversion_matrix: Array2<f32> = array![
        [1.0, 0.00000, 1.5748, 0.0],
        [1.0, -0.187324, -0.468124, 0.0],
        [1.0, 1.8556, 0.00000, 0.0]
    ];
    let rgb = conversion_matrix.dot(&ycbcr);

    (clamp(rgb[0]), clamp(rgb[1]), clamp(rgb[2]))
}

// fn rgb_to_ycbcr((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
//     let rgb = F32x4(r as f32, g as f32, b as f32, 1.0);
//     let y = sum(mul(&rgb, F32x4(0.299000, 0.587000, 0.114000, 0.0)));
//     let cb = sum(mul(&rgb, F32x4(-0.168736, -0.331264, 0.500000, 128.0)));
//     let cr = sum(mul(&rgb, F32x4(0.500000, -0.418688, -0.081312, 128.0)));

//     (clamp(y), clamp(cb), clamp(cr))
// }

#[inline]
fn clamp(val: f32) -> u8 {
    if val < 0.0 {
        0
    } else if val > 255.0 {
        255
    } else {
        val.round() as u8
    }
}
