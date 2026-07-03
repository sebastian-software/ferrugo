# ferrugo-simd

Safe row-kernel boundary for Ferrugo raster compositing.

The crate starts with scalar reference kernels so `ferrugo-render` can depend on
a stable safe API before architecture-specific SIMD is added. Future NEON,
AVX2, AVX-512, or WASM SIMD implementations should live behind this crate's
safe functions and keep renderer crates free of unsafe code.
