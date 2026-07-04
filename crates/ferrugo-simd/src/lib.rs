//! Safe row-kernel boundary for raster compositing.

/// RGBA pixel width in bytes.
pub const RGBA8_BYTES_PER_PIXEL: usize = 4;

/// Blends one RGBA row span with an opaque source color and constant coverage.
///
/// The scalar implementation is the parity oracle for future architecture
/// kernels. It intentionally matches the renderer's truncating floating-point
/// source-over math.
#[inline]
pub fn source_over_opaque_normal_row(row: &mut [u8], source: [u8; 4], coverage: f64) {
    debug_assert_eq!(source[3], 255);
    debug_assert_eq!(row.len() % RGBA8_BYTES_PER_PIXEL, 0);
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return;
    }
    let inverse = 1.0 - coverage;
    for pixel in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        if pixel[3] == 255 {
            source_over_opaque_dest_pixel(pixel, source, coverage, inverse);
        } else {
            let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
            let blended = source_over(source, dest, coverage);
            pixel.copy_from_slice(&blended);
        }
    }
}

/// Blends one RGBA row span with a constant source color and constant coverage.
///
/// This is the scalar parity oracle for future architecture kernels that need
/// to handle non-opaque source colors.
#[inline]
pub fn source_over_normal_row(row: &mut [u8], source: [u8; 4], coverage: f64) {
    debug_assert_eq!(row.len() % RGBA8_BYTES_PER_PIXEL, 0);
    if source[3] == 255 {
        source_over_opaque_normal_row(row, source, coverage);
        return;
    }
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return;
    }
    for pixel in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let blended = source_over(source, dest, coverage);
        pixel.copy_from_slice(&blended);
    }
}

/// Blends one RGBA row span with a constant source color and per-pixel alpha coverage.
///
/// This scalar implementation is the parity oracle for future SIMD coverage
/// kernels. `coverage_alphas` stores 0-255 pixel coverage and is multiplied by
/// `alpha` before source-over compositing.
#[inline]
pub fn source_over_normal_row_with_coverage(
    row: &mut [u8],
    source: [u8; 4],
    alpha: f64,
    coverage_alphas: &[u8],
) {
    debug_assert_eq!(row.len(), coverage_alphas.len() * RGBA8_BYTES_PER_PIXEL);
    if alpha <= f64::EPSILON {
        return;
    }
    let alpha = alpha.clamp(0.0, 1.0);
    for (coverage_alpha, pixel) in coverage_alphas
        .iter()
        .copied()
        .zip(row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL))
    {
        if coverage_alpha == 0 {
            continue;
        }
        let coverage = (alpha * f64::from(coverage_alpha) / 255.0).clamp(0.0, 1.0);
        if coverage <= f64::EPSILON {
            continue;
        }
        if coverage >= 1.0 && source[3] == 255 {
            pixel.copy_from_slice(&source);
            continue;
        }
        if source[3] == 255 && pixel[3] == 255 {
            let inverse = 1.0 - coverage;
            source_over_opaque_dest_pixel(pixel, source, coverage, inverse);
        } else {
            let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
            let blended = source_over(source, dest, coverage);
            pixel.copy_from_slice(&blended);
        }
    }
}

#[inline]
fn source_over_opaque_dest_pixel(pixel: &mut [u8], source: [u8; 4], coverage: f64, inverse: f64) {
    pixel[0] = source_over_opaque_channel(source[0], pixel[0], coverage, inverse);
    pixel[1] = source_over_opaque_channel(source[1], pixel[1], coverage, inverse);
    pixel[2] = source_over_opaque_channel(source[2], pixel[2], coverage, inverse);
    pixel[3] = 255;
}

#[inline]
fn source_over_opaque_channel(source: u8, dest: u8, coverage: f64, inverse: f64) -> u8 {
    f64::from(source)
        .mul_add(coverage, f64::from(dest) * inverse)
        .floor()
        .clamp(0.0, 255.0) as u8
}

#[inline]
fn source_over(source: [u8; 4], dest: [u8; 4], coverage: f64) -> [u8; 4] {
    let source_alpha = (f64::from(source[3]) / 255.0 * coverage).clamp(0.0, 1.0);
    if source_alpha <= f64::EPSILON {
        return dest;
    }
    let dest_alpha = f64::from(dest[3]) / 255.0;
    let out_alpha = source_alpha.mul_add(1.0, dest_alpha * (1.0 - source_alpha));
    if out_alpha <= f64::EPSILON {
        return [0, 0, 0, 0];
    }
    [
        source_over_channel(source[0], dest[0], source_alpha, dest_alpha, out_alpha),
        source_over_channel(source[1], dest[1], source_alpha, dest_alpha, out_alpha),
        source_over_channel(source[2], dest[2], source_alpha, dest_alpha, out_alpha),
        normalized_to_u8(out_alpha),
    ]
}

#[inline]
fn source_over_channel(
    source: u8,
    dest: u8,
    source_alpha: f64,
    dest_alpha: f64,
    out_alpha: f64,
) -> u8 {
    ((f64::from(source) * source_alpha + f64::from(dest) * dest_alpha * (1.0 - source_alpha))
        / out_alpha)
        .floor()
        .clamp(0.0, 255.0) as u8
}

#[inline]
fn normalized_to_u8(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_over_opaque_normal_row_should_match_scalar_cases() {
        let mut row = vec![
            10, 20, 30, 255, 40, 50, 60, 128, 200, 190, 180, 0, 1, 2, 3, 255,
        ];
        source_over_opaque_normal_row(&mut row, [100, 120, 140, 255], 0.5);

        assert_eq!(
            row,
            vec![55, 70, 85, 255, 79, 96, 113, 192, 100, 120, 140, 128, 50, 61, 71, 255,]
        );
    }

    #[test]
    fn source_over_opaque_normal_row_should_leave_zero_coverage_unchanged() {
        let original = vec![10, 20, 30, 255, 40, 50, 60, 128];
        let mut row = original.clone();

        source_over_opaque_normal_row(&mut row, [100, 120, 140, 255], 0.0);

        assert_eq!(row, original);
    }

    #[test]
    fn source_over_opaque_normal_row_should_clamp_full_coverage_to_source() {
        let mut row = vec![10, 20, 30, 255, 40, 50, 60, 128];

        source_over_opaque_normal_row(&mut row, [100, 120, 140, 255], 2.0);

        assert_eq!(row, vec![100, 120, 140, 255, 100, 120, 140, 255]);
    }

    #[test]
    fn source_over_normal_row_should_match_non_opaque_scalar_cases() {
        let mut row = vec![10, 20, 30, 255, 40, 50, 60, 128, 200, 190, 180, 0];

        source_over_normal_row(&mut row, [100, 120, 140, 128], 0.5);

        assert_eq!(
            row,
            vec![32, 45, 57, 255, 64, 78, 92, 160, 100, 120, 140, 64]
        );
    }

    #[test]
    fn source_over_normal_row_with_coverage_should_match_constant_row_cases() {
        let mut actual = vec![
            10, 20, 30, 255, 40, 50, 60, 128, 200, 190, 180, 0, 1, 2, 3, 255,
        ];
        let mut expected = actual.clone();
        let source = [100, 120, 140, 128];
        let coverage = [0, 64, 128, 255];

        source_over_normal_row_with_coverage(&mut actual, source, 0.75, &coverage);
        for (coverage_alpha, pixel) in coverage
            .iter()
            .copied()
            .zip(expected.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL))
        {
            let effective_coverage = 0.75 * f64::from(coverage_alpha) / 255.0;
            let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
            pixel.copy_from_slice(&source_over(source, dest, effective_coverage));
        }

        assert_eq!(actual, expected);
    }
}
