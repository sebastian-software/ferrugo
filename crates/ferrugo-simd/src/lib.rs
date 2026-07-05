//! Safe row-kernel boundary for raster compositing.

#![cfg_attr(
    not(test),
    deny(clippy::expect_used, clippy::panic, clippy::unwrap_used)
)]

/// RGBA pixel width in bytes.
pub const RGBA8_BYTES_PER_PIXEL: usize = 4;
const SIMD_COVERAGE_MIN_PIXELS: usize = 16;

/// Runtime backend used by a row kernel call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKernelBackend {
    /// Floating-point scalar parity path.
    ScalarFloat,
    /// Integer scalar parity path for opaque source-over coverage spans.
    ScalarInteger,
    /// AArch64 NEON integer row kernel.
    Aarch64Neon,
    /// x86/x86_64 SSE2 integer row kernel.
    X86Sse2,
    /// x86/x86_64 AVX2 integer row kernel.
    X86Avx2,
}

impl RowKernelBackend {
    /// Stable trace label for this backend.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ScalarFloat => "scalar-float",
            Self::ScalarInteger => "scalar-integer",
            Self::Aarch64Neon => "aarch64-neon",
            Self::X86Sse2 => "x86-sse2",
            Self::X86Avx2 => "x86-avx2",
        }
    }

    /// Returns true when the backend used an architecture vector kernel.
    pub const fn is_simd(self) -> bool {
        matches!(self, Self::Aarch64Neon | Self::X86Sse2 | Self::X86Avx2)
    }
}

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
) -> RowKernelBackend {
    debug_assert_eq!(row.len(), coverage_alphas.len() * RGBA8_BYTES_PER_PIXEL);
    if can_use_opaque_integer_coverage(source, alpha, coverage_alphas)
        && row.len() == coverage_alphas.len() * RGBA8_BYTES_PER_PIXEL
    {
        return dispatch_opaque_integer_coverage(row, source, coverage_alphas);
    }
    source_over_normal_row_with_coverage_scalar(row, source, alpha, coverage_alphas);
    RowKernelBackend::ScalarFloat
}

fn source_over_normal_row_with_coverage_scalar(
    row: &mut [u8],
    source: [u8; 4],
    alpha: f64,
    coverage_alphas: &[u8],
) {
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

fn can_use_opaque_integer_coverage(source: [u8; 4], alpha: f64, coverage_alphas: &[u8]) -> bool {
    source[3] == 255 && (alpha - 1.0).abs() <= f64::EPSILON && !coverage_alphas.is_empty()
}

fn dispatch_opaque_integer_coverage(
    row: &mut [u8],
    source: [u8; 4],
    coverage_alphas: &[u8],
) -> RowKernelBackend {
    if coverage_alphas.len() < SIMD_COVERAGE_MIN_PIXELS {
        source_over_normal_row_with_coverage_scalar(row, source, 1.0, coverage_alphas);
        return RowKernelBackend::ScalarFloat;
    }
    if !has_opaque_dest_row(row) {
        source_over_opaque_coverage_integer_scalar(row, source, coverage_alphas);
        return RowKernelBackend::ScalarInteger;
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Runtime feature detection confirms NEON support, and the
            // kernel only reads/writes within row and coverage slices.
            unsafe {
                source_over_opaque_coverage_neon(row, source, coverage_alphas);
            }
            return RowKernelBackend::Aarch64Neon;
        }
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: Runtime feature detection confirms AVX2 support, and the
            // kernel only reads/writes within row and coverage slices.
            unsafe {
                source_over_opaque_coverage_avx2(row, source, coverage_alphas);
            }
            return RowKernelBackend::X86Avx2;
        }
        if std::arch::is_x86_feature_detected!("sse2") {
            // SAFETY: Runtime feature detection confirms SSE2 support, and the
            // kernel only reads/writes within row and coverage slices.
            unsafe {
                source_over_opaque_coverage_sse2(row, source, coverage_alphas);
            }
            return RowKernelBackend::X86Sse2;
        }
    }
    source_over_opaque_coverage_integer_scalar(row, source, coverage_alphas);
    RowKernelBackend::ScalarInteger
}

fn source_over_opaque_coverage_integer_scalar(
    row: &mut [u8],
    source: [u8; 4],
    coverage_alphas: &[u8],
) {
    for (coverage_alpha, pixel) in coverage_alphas
        .iter()
        .copied()
        .zip(row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL))
    {
        if coverage_alpha == 0 {
            continue;
        }
        if coverage_alpha == 255 {
            pixel.copy_from_slice(&source);
            continue;
        }
        if pixel[3] != 255 {
            let coverage = f64::from(coverage_alpha) / 255.0;
            let dest = [pixel[0], pixel[1], pixel[2], pixel[3]];
            let blended = source_over(source, dest, coverage);
            pixel.copy_from_slice(&blended);
            continue;
        }
        let inverse = 255 - u16::from(coverage_alpha);
        let coverage = u16::from(coverage_alpha);
        for channel in 0..3 {
            let value = u16::from(source[channel]) * coverage + u16::from(pixel[channel]) * inverse;
            pixel[channel] = div_255_u16(value) as u8;
        }
        pixel[3] = 255;
    }
}

fn has_opaque_dest_row(row: &[u8]) -> bool {
    row.chunks_exact(RGBA8_BYTES_PER_PIXEL)
        .all(|pixel| pixel[3] == 255)
}

#[inline]
fn div_255_u16(value: u16) -> u16 {
    (value + 1 + (value >> 8)) >> 8
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn source_over_opaque_coverage_neon(
    row: &mut [u8],
    source: [u8; 4],
    coverage_alphas: &[u8],
) {
    // SAFETY: The caller performed runtime NEON detection. Pointer arithmetic
    // stays within `row`, and each vector load/store covers exactly four RGBA
    // pixels guarded by the loop bound.
    unsafe {
        use std::arch::aarch64::*;

        let source_bytes = repeated_source_16(source);
        let source_vector = vld1q_u8(source_bytes.as_ptr());
        let mut offset = 0;
        while offset + 4 <= coverage_alphas.len() {
            let coverage_bytes = repeated_coverage_16(&coverage_alphas[offset..offset + 4]);
            let coverage = vld1q_u8(coverage_bytes.as_ptr());
            let inverse = vsubq_u8(vdupq_n_u8(255), coverage);
            let ptr = row.as_mut_ptr().add(offset * RGBA8_BYTES_PER_PIXEL);
            let dest = vld1q_u8(ptr);
            let blended = neon_blend_opaque_coverage(source_vector, dest, coverage, inverse);
            vst1q_u8(ptr, blended);
            set_opaque_alpha_bytes(&mut row[offset * RGBA8_BYTES_PER_PIXEL..][..16]);
            offset += 4;
        }
        source_over_opaque_coverage_integer_scalar(
            &mut row[offset * RGBA8_BYTES_PER_PIXEL..],
            source,
            &coverage_alphas[offset..],
        );
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn neon_blend_opaque_coverage(
    source: std::arch::aarch64::uint8x16_t,
    dest: std::arch::aarch64::uint8x16_t,
    coverage: std::arch::aarch64::uint8x16_t,
    inverse: std::arch::aarch64::uint8x16_t,
) -> std::arch::aarch64::uint8x16_t {
    // SAFETY: The parent NEON kernel is only called after runtime feature
    // detection, and all operations stay within vector registers.
    unsafe {
        use std::arch::aarch64::*;

        let low = neon_div_255_u16(vaddq_u16(
            vmulq_u16(
                vmovl_u8(vget_low_u8(source)),
                vmovl_u8(vget_low_u8(coverage)),
            ),
            vmulq_u16(vmovl_u8(vget_low_u8(dest)), vmovl_u8(vget_low_u8(inverse))),
        ));
        let high = neon_div_255_u16(vaddq_u16(
            vmulq_u16(
                vmovl_u8(vget_high_u8(source)),
                vmovl_u8(vget_high_u8(coverage)),
            ),
            vmulq_u16(
                vmovl_u8(vget_high_u8(dest)),
                vmovl_u8(vget_high_u8(inverse)),
            ),
        ));
        vcombine_u8(vmovn_u16(low), vmovn_u16(high))
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn neon_div_255_u16(
    value: std::arch::aarch64::uint16x8_t,
) -> std::arch::aarch64::uint16x8_t {
    // SAFETY: The parent NEON kernel is only called after runtime feature
    // detection, and all operations stay within vector registers.
    unsafe {
        use std::arch::aarch64::*;

        vshrq_n_u16(
            vaddq_u16(vaddq_u16(value, vshrq_n_u16(value, 8)), vdupq_n_u16(1)),
            8,
        )
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn source_over_opaque_coverage_sse2(
    row: &mut [u8],
    source: [u8; 4],
    coverage_alphas: &[u8],
) {
    // SAFETY: The caller performed runtime SSE2 detection. Pointer arithmetic
    // stays within `row`, and each vector load/store covers exactly four RGBA
    // pixels guarded by the loop bound.
    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let source_bytes = repeated_source_16(source);
        let source_vector = _mm_loadu_si128(source_bytes.as_ptr().cast());
        let mut offset = 0;
        while offset + 4 <= coverage_alphas.len() {
            let coverage_bytes = repeated_coverage_16(&coverage_alphas[offset..offset + 4]);
            let coverage = _mm_loadu_si128(coverage_bytes.as_ptr().cast());
            let inverse = _mm_sub_epi8(_mm_set1_epi8(-1), coverage);
            let ptr = row.as_mut_ptr().add(offset * RGBA8_BYTES_PER_PIXEL).cast();
            let dest = _mm_loadu_si128(ptr);
            let blended = sse2_blend_opaque_coverage(source_vector, dest, coverage, inverse);
            _mm_storeu_si128(ptr, blended);
            set_opaque_alpha_bytes(&mut row[offset * RGBA8_BYTES_PER_PIXEL..][..16]);
            offset += 4;
        }
        source_over_opaque_coverage_integer_scalar(
            &mut row[offset * RGBA8_BYTES_PER_PIXEL..],
            source,
            &coverage_alphas[offset..],
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn source_over_opaque_coverage_avx2(
    row: &mut [u8],
    source: [u8; 4],
    coverage_alphas: &[u8],
) {
    // SAFETY: The caller performed runtime AVX2 detection. Pointer arithmetic
    // stays within `row`, and each vector load/store covers exactly eight RGBA
    // pixels guarded by the loop bound.
    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let source_bytes = repeated_source_32(source);
        let source_vector = _mm256_loadu_si256(source_bytes.as_ptr().cast());
        let mut offset = 0;
        while offset + 8 <= coverage_alphas.len() {
            let coverage_bytes = repeated_coverage_32(&coverage_alphas[offset..offset + 8]);
            let coverage = _mm256_loadu_si256(coverage_bytes.as_ptr().cast());
            let inverse = _mm256_sub_epi8(_mm256_set1_epi8(-1), coverage);
            let ptr = row.as_mut_ptr().add(offset * RGBA8_BYTES_PER_PIXEL).cast();
            let dest = _mm256_loadu_si256(ptr);
            let blended = avx2_blend_opaque_coverage(source_vector, dest, coverage, inverse);
            _mm256_storeu_si256(ptr, blended);
            set_opaque_alpha_bytes(&mut row[offset * RGBA8_BYTES_PER_PIXEL..][..32]);
            offset += 8;
        }
        source_over_opaque_coverage_sse2(
            &mut row[offset * RGBA8_BYTES_PER_PIXEL..],
            source,
            &coverage_alphas[offset..],
        );
    }
}

#[cfg(target_arch = "x86")]
type M128i = std::arch::x86::__m128i;
#[cfg(target_arch = "x86_64")]
type M128i = std::arch::x86_64::__m128i;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn sse2_blend_opaque_coverage(
    source: M128i,
    dest: M128i,
    coverage: M128i,
    inverse: M128i,
) -> M128i {
    // SAFETY: The parent SSE2/AVX2 kernel is only called after runtime feature
    // detection, and all operations stay within vector registers.
    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let zero = _mm_setzero_si128();
        let low = sse2_div_255_u16(_mm_add_epi16(
            _mm_mullo_epi16(
                _mm_unpacklo_epi8(source, zero),
                _mm_unpacklo_epi8(coverage, zero),
            ),
            _mm_mullo_epi16(
                _mm_unpacklo_epi8(dest, zero),
                _mm_unpacklo_epi8(inverse, zero),
            ),
        ));
        let high = sse2_div_255_u16(_mm_add_epi16(
            _mm_mullo_epi16(
                _mm_unpackhi_epi8(source, zero),
                _mm_unpackhi_epi8(coverage, zero),
            ),
            _mm_mullo_epi16(
                _mm_unpackhi_epi8(dest, zero),
                _mm_unpackhi_epi8(inverse, zero),
            ),
        ));
        _mm_packus_epi16(low, high)
    }
}

#[cfg(target_arch = "x86")]
type M256i = std::arch::x86::__m256i;
#[cfg(target_arch = "x86_64")]
type M256i = std::arch::x86_64::__m256i;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn avx2_blend_opaque_coverage(
    source: M256i,
    dest: M256i,
    coverage: M256i,
    inverse: M256i,
) -> M256i {
    // SAFETY: The parent AVX2 kernel is only called after runtime feature
    // detection, and all operations stay within vector registers.
    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let zero = _mm256_setzero_si256();
        let low = avx2_div_255_u16(_mm256_add_epi16(
            _mm256_mullo_epi16(
                _mm256_unpacklo_epi8(source, zero),
                _mm256_unpacklo_epi8(coverage, zero),
            ),
            _mm256_mullo_epi16(
                _mm256_unpacklo_epi8(dest, zero),
                _mm256_unpacklo_epi8(inverse, zero),
            ),
        ));
        let high = avx2_div_255_u16(_mm256_add_epi16(
            _mm256_mullo_epi16(
                _mm256_unpackhi_epi8(source, zero),
                _mm256_unpackhi_epi8(coverage, zero),
            ),
            _mm256_mullo_epi16(
                _mm256_unpackhi_epi8(dest, zero),
                _mm256_unpackhi_epi8(inverse, zero),
            ),
        ));
        _mm256_packus_epi16(low, high)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn sse2_div_255_u16(value: M128i) -> M128i {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    _mm_srli_epi16(
        _mm_add_epi16(
            _mm_add_epi16(value, _mm_srli_epi16(value, 8)),
            _mm_set1_epi16(1),
        ),
        8,
    )
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn avx2_div_255_u16(value: M256i) -> M256i {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    _mm256_srli_epi16(
        _mm256_add_epi16(
            _mm256_add_epi16(value, _mm256_srli_epi16(value, 8)),
            _mm256_set1_epi16(1),
        ),
        8,
    )
}

fn repeated_source_16(source: [u8; 4]) -> [u8; 16] {
    [
        source[0], source[1], source[2], source[3], source[0], source[1], source[2], source[3],
        source[0], source[1], source[2], source[3], source[0], source[1], source[2], source[3],
    ]
}

fn set_opaque_alpha_bytes(row: &mut [u8]) {
    debug_assert_eq!(row.len() % RGBA8_BYTES_PER_PIXEL, 0);
    for pixel in row.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        pixel[3] = 255;
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn repeated_source_32(source: [u8; 4]) -> [u8; 32] {
    let mut bytes = [0; 32];
    for chunk in bytes.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        chunk.copy_from_slice(&source);
    }
    bytes
}

fn repeated_coverage_16(coverage: &[u8]) -> [u8; 16] {
    debug_assert_eq!(coverage.len(), 4);
    let mut bytes = [0; 16];
    for (pixel, coverage) in bytes
        .chunks_exact_mut(RGBA8_BYTES_PER_PIXEL)
        .zip(coverage.iter().copied())
    {
        pixel[0] = coverage;
        pixel[1] = coverage;
        pixel[2] = coverage;
        pixel[3] = 255;
    }
    bytes
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn repeated_coverage_32(coverage: &[u8]) -> [u8; 32] {
    debug_assert_eq!(coverage.len(), 8);
    let mut bytes = [0; 32];
    for (pixel, coverage) in bytes
        .chunks_exact_mut(RGBA8_BYTES_PER_PIXEL)
        .zip(coverage.iter().copied())
    {
        pixel[0] = coverage;
        pixel[1] = coverage;
        pixel[2] = coverage;
        pixel[3] = 255;
    }
    bytes
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

    #[test]
    fn source_over_normal_row_with_coverage_should_match_scalar_oracle_for_opaque_rows() {
        let source = [90, 130, 220, 255];
        let coverage = (0..32)
            .map(|index| ((index * 37) % 256) as u8)
            .collect::<Vec<_>>();
        let mut actual = coverage
            .iter()
            .enumerate()
            .flat_map(|(index, _)| {
                [
                    (index * 13 % 256) as u8,
                    (index * 29 % 256) as u8,
                    (index * 47 % 256) as u8,
                    255,
                ]
            })
            .collect::<Vec<_>>();
        let mut expected = actual.clone();

        let backend = source_over_normal_row_with_coverage(&mut actual, source, 1.0, &coverage);
        source_over_normal_row_with_coverage_scalar(&mut expected, source, 1.0, &coverage);

        assert_ne!(backend, RowKernelBackend::ScalarFloat);
        assert_eq!(actual, expected);
    }

    #[test]
    fn source_over_normal_row_with_coverage_should_report_runtime_simd_backend_when_available() {
        let source = [90, 130, 220, 255];
        let coverage = (0..32)
            .map(|index| ((index * 17) % 256) as u8)
            .collect::<Vec<_>>();
        let mut row = coverage
            .iter()
            .enumerate()
            .flat_map(|(index, _)| {
                [
                    (index * 7 % 256) as u8,
                    (index * 11 % 256) as u8,
                    (index * 19 % 256) as u8,
                    255,
                ]
            })
            .collect::<Vec<_>>();

        let backend = source_over_normal_row_with_coverage(&mut row, source, 1.0, &coverage);

        #[cfg(target_arch = "aarch64")]
        assert_eq!(backend, RowKernelBackend::Aarch64Neon);
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        assert!(matches!(
            backend,
            RowKernelBackend::X86Avx2 | RowKernelBackend::X86Sse2
        ));
    }

    #[test]
    fn source_over_normal_row_with_coverage_should_match_scalar_oracle_for_translucent_rows() {
        let mut row = vec![10, 20, 30, 128, 40, 50, 60, 255];
        let mut expected = row.clone();
        let coverage = [64, 128];

        let backend =
            source_over_normal_row_with_coverage(&mut row, [100, 120, 140, 255], 1.0, &coverage);
        source_over_normal_row_with_coverage_scalar(
            &mut expected,
            [100, 120, 140, 255],
            1.0,
            &coverage,
        );

        assert_eq!(backend, RowKernelBackend::ScalarFloat);
        assert_eq!(row, expected);
    }
}
