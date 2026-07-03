//! Safe row-kernel boundary for raster compositing.

/// RGBA pixel width in bytes.
pub const RGBA8_BYTES_PER_PIXEL: usize = 4;

/// Blends one RGBA row span with an opaque source color and constant coverage.
///
/// The scalar implementation is the parity oracle for future architecture
/// kernels. It intentionally matches the renderer's truncating floating-point
/// source-over math.
pub fn source_over_opaque_normal_row(row: &mut [u8], source: [u8; 4], coverage: f64) {
    debug_assert_eq!(source[3], 255);
    debug_assert_eq!(row.len() % RGBA8_BYTES_PER_PIXEL, 0);
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return;
    }
    for pixel in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
        let blended = if dest[3] == 255 {
            source_over_opaque_dest(source, dest, coverage)
        } else {
            source_over(source, dest, coverage)
        };
        pixel.copy_from_slice(&blended);
    }
}

fn source_over_opaque_dest(source: [u8; 4], dest: [u8; 4], coverage: f64) -> [u8; 4] {
    let inverse = 1.0 - coverage;
    [
        source_over_opaque_channel(source[0], dest[0], coverage, inverse),
        source_over_opaque_channel(source[1], dest[1], coverage, inverse),
        source_over_opaque_channel(source[2], dest[2], coverage, inverse),
        255,
    ]
}

fn source_over_opaque_channel(source: u8, dest: u8, coverage: f64, inverse: f64) -> u8 {
    f64::from(source)
        .mul_add(coverage, f64::from(dest) * inverse)
        .floor()
        .clamp(0.0, 255.0) as u8
}

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
}
