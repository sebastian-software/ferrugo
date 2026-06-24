//! Local PDFium backend for the thumbnail facade.

#![deny(unsafe_op_in_unsafe_fn)]

use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use pdfrust_thumbnail::{PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions};

static PDFIUM_LOCK: Mutex<()> = Mutex::new(());

/// Environment variable pointing at a local PDFium dynamic library.
pub const PDFIUM_LIBRARY_ENV: &str = "PDFRUST_PDFIUM_LIBRARY";

/// Backend that loads a local PDFium library at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfiumBackend {
    library_path: PathBuf,
}

impl PdfiumBackend {
    /// Creates a backend from an explicit dynamic-library path.
    #[must_use]
    pub fn new(library_path: impl Into<PathBuf>) -> Self {
        Self {
            library_path: library_path.into(),
        }
    }

    /// Creates a backend from [`PDFIUM_LIBRARY_ENV`].
    ///
    /// # Errors
    ///
    /// Returns [`PdfiumBackendError::MissingLibraryPath`] when the environment
    /// variable is absent or empty.
    pub fn from_env() -> Result<Self, PdfiumBackendError> {
        let value = env::var_os(PDFIUM_LIBRARY_ENV)
            .filter(|value| !value.is_empty())
            .ok_or(PdfiumBackendError::MissingLibraryPath {
                variable: PDFIUM_LIBRARY_ENV,
            })?;
        Ok(Self::new(PathBuf::from(value)))
    }

    /// Returns the configured PDFium dynamic-library path.
    #[must_use]
    pub fn library_path(&self) -> &Path {
        &self.library_path
    }

    /// Loads PDFium, initializes it, reads the initial error code, and shuts it down.
    ///
    /// # Errors
    ///
    /// Returns [`PdfiumBackendError`] when the library cannot be loaded or a
    /// required symbol is missing.
    pub fn smoke_test(&self) -> Result<PdfiumProbe, PdfiumBackendError> {
        let _guard = PDFIUM_LOCK
            .lock()
            .map_err(|_| PdfiumBackendError::LockPoisoned)?;
        let library = unsafe { sys::PdfiumLibrary::open(&self.library_path) }?;
        unsafe { library.init() };
        let last_error = unsafe { library.get_last_error() };
        unsafe { library.destroy() };
        Ok(PdfiumProbe {
            library_path: self.library_path.clone(),
            initialized: true,
            last_error,
        })
    }
}

impl ThumbnailBackend for PdfiumBackend {
    fn backend_name(&self) -> &'static str {
        "pdfium"
    }

    fn render(
        &self,
        source: PdfSource<'_>,
        options: &ThumbnailOptions,
    ) -> Result<Thumbnail, ThumbnailError> {
        let bytes = load_source(source)?;
        let _guard = PDFIUM_LOCK.lock().map_err(|_| {
            ThumbnailError::internal("PDFium backend serialization lock is poisoned")
        })?;
        let library = unsafe { sys::PdfiumLibrary::open(&self.library_path) }
            .map_err(|err| ThumbnailError::internal(err.to_string()))?;
        unsafe { library.init() };
        let result = unsafe { library.render_first_page_rgba(&bytes, options) };
        unsafe { library.destroy() };
        result
    }
}

fn load_source(source: PdfSource<'_>) -> Result<std::borrow::Cow<'_, [u8]>, ThumbnailError> {
    match source {
        PdfSource::Bytes(bytes) => Ok(std::borrow::Cow::Borrowed(bytes)),
        PdfSource::File(path) => fs::read(path)
            .map(std::borrow::Cow::Owned)
            .map_err(|_| ThumbnailError::Malformed),
    }
}

/// Result of a local PDFium initialization probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfiumProbe {
    /// Library path used for the probe.
    pub library_path: PathBuf,
    /// Whether `FPDF_InitLibrary` returned normally.
    pub initialized: bool,
    /// `FPDF_GetLastError` immediately after initialization.
    pub last_error: u32,
}

/// PDFium backend setup and loading failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfiumBackendError {
    /// The runtime library path environment variable is missing.
    MissingLibraryPath { variable: &'static str },
    /// The runtime library could not be opened.
    OpenLibrary { path: PathBuf, message: String },
    /// A required PDFium symbol could not be loaded.
    LoadSymbol {
        symbol: &'static str,
        message: String,
    },
    /// The global backend serialization lock was poisoned.
    LockPoisoned,
    /// The current platform is not supported by the runtime loader.
    UnsupportedPlatform,
}

impl fmt::Display for PdfiumBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLibraryPath { variable } => {
                write!(f, "{variable} is not set")
            }
            Self::OpenLibrary { path, message } => {
                write!(
                    f,
                    "failed to open PDFium library `{}`: {message}",
                    path.display()
                )
            }
            Self::LoadSymbol { symbol, message } => {
                write!(f, "failed to load PDFium symbol `{symbol}`: {message}")
            }
            Self::LockPoisoned => f.write_str("PDFium backend lock is poisoned"),
            Self::UnsupportedPlatform => f.write_str("runtime PDFium loading is unsupported"),
        }
    }
}

impl std::error::Error for PdfiumBackendError {}

#[cfg(unix)]
mod sys {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_float, c_int, c_ulong, c_void};
    use std::path::{Path, PathBuf};
    use std::ptr::NonNull;

    use super::PdfiumBackendError;
    use pdfrust_thumbnail::{PixelFormat, Rgba, Thumbnail, ThumbnailError, ThumbnailOptions};

    const RTLD_NOW: c_int = 2;
    const NO_PASSWORD: *const c_char = std::ptr::null();
    const NO_RENDER_FLAGS: c_int = 0;

    #[cfg_attr(target_os = "linux", link(name = "dl"))]
    extern "C" {
        fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlerror() -> *const c_char;
    }

    type FpdfInitLibrary = unsafe extern "C" fn();
    type FpdfDestroyLibrary = unsafe extern "C" fn();
    type FpdfGetLastError = unsafe extern "C" fn() -> c_ulong;
    type FpdfLoadMemDocument64 =
        unsafe extern "C" fn(*const c_void, usize, *const c_char) -> *mut c_void;
    type FpdfCloseDocument = unsafe extern "C" fn(*mut c_void);
    type FpdfLoadPage = unsafe extern "C" fn(*mut c_void, c_int) -> *mut c_void;
    type FpdfClosePage = unsafe extern "C" fn(*mut c_void);
    type FpdfGetPageWidthF = unsafe extern "C" fn(*mut c_void) -> c_float;
    type FpdfGetPageHeightF = unsafe extern "C" fn(*mut c_void) -> c_float;
    type FpdfBitmapCreate = unsafe extern "C" fn(c_int, c_int, c_int) -> *mut c_void;
    type FpdfBitmapDestroy = unsafe extern "C" fn(*mut c_void);
    type FpdfBitmapFillRect = unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int, u32);
    type FpdfRenderPageBitmap =
        unsafe extern "C" fn(*mut c_void, *mut c_void, c_int, c_int, c_int, c_int, c_int, c_int);
    type FpdfBitmapGetBuffer = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
    type FpdfBitmapGetStride = unsafe extern "C" fn(*mut c_void) -> c_int;

    pub(super) struct PdfiumLibrary {
        path: PathBuf,
        handle: NonNull<c_void>,
        init_library: FpdfInitLibrary,
        destroy_library: FpdfDestroyLibrary,
        get_last_error: FpdfGetLastError,
        load_mem_document64: FpdfLoadMemDocument64,
        close_document: FpdfCloseDocument,
        load_page: FpdfLoadPage,
        close_page: FpdfClosePage,
        get_page_width_f: FpdfGetPageWidthF,
        get_page_height_f: FpdfGetPageHeightF,
        bitmap_create: FpdfBitmapCreate,
        bitmap_destroy: FpdfBitmapDestroy,
        bitmap_fill_rect: FpdfBitmapFillRect,
        render_page_bitmap: FpdfRenderPageBitmap,
        bitmap_get_buffer: FpdfBitmapGetBuffer,
        bitmap_get_stride: FpdfBitmapGetStride,
    }

    impl PdfiumLibrary {
        pub(super) unsafe fn open(path: &Path) -> Result<Self, PdfiumBackendError> {
            let path_string = path.to_string_lossy();
            let c_path = CString::new(path_string.as_bytes()).map_err(|err| {
                PdfiumBackendError::OpenLibrary {
                    path: path.to_path_buf(),
                    message: err.to_string(),
                }
            })?;
            let raw = unsafe { dlopen(c_path.as_ptr(), RTLD_NOW) };
            let handle = NonNull::new(raw).ok_or_else(|| PdfiumBackendError::OpenLibrary {
                path: path.to_path_buf(),
                message: last_error_message(),
            })?;
            let library = Self {
                path: path.to_path_buf(),
                handle,
                init_library: unsafe { load_symbol(handle, "FPDF_InitLibrary") }?,
                destroy_library: unsafe { load_symbol(handle, "FPDF_DestroyLibrary") }?,
                get_last_error: unsafe { load_symbol(handle, "FPDF_GetLastError") }?,
                load_mem_document64: unsafe { load_symbol(handle, "FPDF_LoadMemDocument64") }?,
                close_document: unsafe { load_symbol(handle, "FPDF_CloseDocument") }?,
                load_page: unsafe { load_symbol(handle, "FPDF_LoadPage") }?,
                close_page: unsafe { load_symbol(handle, "FPDF_ClosePage") }?,
                get_page_width_f: unsafe { load_symbol(handle, "FPDF_GetPageWidthF") }?,
                get_page_height_f: unsafe { load_symbol(handle, "FPDF_GetPageHeightF") }?,
                bitmap_create: unsafe { load_symbol(handle, "FPDFBitmap_Create") }?,
                bitmap_destroy: unsafe { load_symbol(handle, "FPDFBitmap_Destroy") }?,
                bitmap_fill_rect: unsafe { load_symbol(handle, "FPDFBitmap_FillRect") }?,
                render_page_bitmap: unsafe { load_symbol(handle, "FPDF_RenderPageBitmap") }?,
                bitmap_get_buffer: unsafe { load_symbol(handle, "FPDFBitmap_GetBuffer") }?,
                bitmap_get_stride: unsafe { load_symbol(handle, "FPDFBitmap_GetStride") }?,
            };
            Ok(library)
        }

        pub(super) unsafe fn init(&self) {
            unsafe { (self.init_library)() };
        }

        pub(super) unsafe fn destroy(&self) {
            unsafe { (self.destroy_library)() };
        }

        pub(super) unsafe fn get_last_error(&self) -> u32 {
            unsafe { (self.get_last_error)() as u32 }
        }

        pub(super) unsafe fn render_first_page_rgba(
            &self,
            bytes: &[u8],
            options: &ThumbnailOptions,
        ) -> Result<Thumbnail, ThumbnailError> {
            if options.max_edge == 0 {
                return Err(ThumbnailError::internal(
                    "max_edge must be greater than zero",
                ));
            }
            let document = unsafe {
                (self.load_mem_document64)(
                    bytes.as_ptr().cast::<c_void>(),
                    bytes.len(),
                    NO_PASSWORD,
                )
            };
            let document = NonNull::new(document).ok_or(ThumbnailError::Malformed)?;
            let document = PdfDocument {
                library: self,
                document,
            };
            let page = unsafe {
                (self.load_page)(document.document.as_ptr(), options.page_index as c_int)
            };
            let page = NonNull::new(page).ok_or(ThumbnailError::Malformed)?;
            let page = PdfPage {
                library: self,
                page,
            };
            let page_width = unsafe { (self.get_page_width_f)(page.page.as_ptr()) };
            let page_height = unsafe { (self.get_page_height_f)(page.page.as_ptr()) };
            let (width, height) = scaled_dimensions(page_width, page_height, options.max_edge)?;
            let bitmap = unsafe { (self.bitmap_create)(width as c_int, height as c_int, 1) };
            let bitmap = NonNull::new(bitmap)
                .ok_or_else(|| ThumbnailError::internal("PDFium bitmap allocation failed"))?;
            let bitmap = PdfBitmap {
                library: self,
                bitmap,
            };
            unsafe {
                (self.bitmap_fill_rect)(
                    bitmap.bitmap.as_ptr(),
                    0,
                    0,
                    width as c_int,
                    height as c_int,
                    pdfium_color(options.background),
                );
                (self.render_page_bitmap)(
                    bitmap.bitmap.as_ptr(),
                    page.page.as_ptr(),
                    0,
                    0,
                    width as c_int,
                    height as c_int,
                    0,
                    NO_RENDER_FLAGS,
                );
            }
            let stride = unsafe { (self.bitmap_get_stride)(bitmap.bitmap.as_ptr()) };
            if stride <= 0 {
                return Err(ThumbnailError::internal(
                    "PDFium returned invalid bitmap stride",
                ));
            }
            let buffer = unsafe { (self.bitmap_get_buffer)(bitmap.bitmap.as_ptr()) };
            let buffer = NonNull::new(buffer)
                .ok_or_else(|| ThumbnailError::internal("PDFium returned a null bitmap buffer"))?;
            let source_len = (stride as usize)
                .checked_mul(height as usize)
                .ok_or_else(|| ThumbnailError::internal("PDFium bitmap size overflow"))?;
            let source =
                unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), source_len) };
            let rgba = bgra_to_rgba(source, stride as usize, width, height)?;
            Thumbnail::rgba(width, height, rgba)
        }
    }

    struct PdfDocument<'a> {
        library: &'a PdfiumLibrary,
        document: NonNull<c_void>,
    }

    impl Drop for PdfDocument<'_> {
        fn drop(&mut self) {
            unsafe { (self.library.close_document)(self.document.as_ptr()) };
        }
    }

    struct PdfPage<'a> {
        library: &'a PdfiumLibrary,
        page: NonNull<c_void>,
    }

    impl Drop for PdfPage<'_> {
        fn drop(&mut self) {
            unsafe { (self.library.close_page)(self.page.as_ptr()) };
        }
    }

    struct PdfBitmap<'a> {
        library: &'a PdfiumLibrary,
        bitmap: NonNull<c_void>,
    }

    impl Drop for PdfBitmap<'_> {
        fn drop(&mut self) {
            unsafe { (self.library.bitmap_destroy)(self.bitmap.as_ptr()) };
        }
    }

    impl Drop for PdfiumLibrary {
        fn drop(&mut self) {
            let _ = unsafe { dlclose(self.handle.as_ptr()) };
        }
    }

    unsafe fn load_symbol<T>(
        handle: NonNull<c_void>,
        symbol: &'static str,
    ) -> Result<T, PdfiumBackendError>
    where
        T: Copy,
    {
        let c_symbol = CString::new(symbol).expect("PDFium symbol names do not contain NUL bytes");
        let raw = unsafe { dlsym(handle.as_ptr(), c_symbol.as_ptr()) };
        if raw.is_null() {
            return Err(PdfiumBackendError::LoadSymbol {
                symbol,
                message: last_error_message(),
            });
        }
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<*mut c_void>());
        let typed = unsafe { std::mem::transmute_copy::<*mut c_void, T>(&raw) };
        Ok(typed)
    }

    fn scaled_dimensions(
        page_width: c_float,
        page_height: c_float,
        max_edge: u32,
    ) -> Result<(u32, u32), ThumbnailError> {
        if !page_width.is_finite()
            || !page_height.is_finite()
            || page_width <= 0.0
            || page_height <= 0.0
        {
            return Err(ThumbnailError::Malformed);
        }
        let page_max = page_width.max(page_height);
        let scale = if page_max > max_edge as c_float {
            max_edge as c_float / page_max
        } else {
            1.0
        };
        let width = ((page_width * scale).round() as u32).clamp(1, max_edge);
        let height = ((page_height * scale).round() as u32).clamp(1, max_edge);
        Ok((width, height))
    }

    const fn pdfium_color(color: Rgba) -> u32 {
        ((color.a as u32) << 24)
            | ((color.r as u32) << 16)
            | ((color.g as u32) << 8)
            | color.b as u32
    }

    fn bgra_to_rgba(
        source: &[u8],
        source_stride: usize,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, ThumbnailError> {
        let target_stride = (width as usize)
            .checked_mul(PixelFormat::Rgba8.bytes_per_pixel())
            .ok_or_else(|| ThumbnailError::internal("target stride overflow"))?;
        if source_stride < target_stride {
            return Err(ThumbnailError::internal(
                "PDFium bitmap stride is too small",
            ));
        }
        let target_len = target_stride
            .checked_mul(height as usize)
            .ok_or_else(|| ThumbnailError::internal("target bitmap size overflow"))?;
        let mut target = vec![0; target_len];
        for row in 0..height as usize {
            let source_row = &source[row * source_stride..row * source_stride + target_stride];
            let target_row = &mut target[row * target_stride..(row + 1) * target_stride];
            for (source_pixel, target_pixel) in source_row
                .chunks_exact(4)
                .zip(target_row.chunks_exact_mut(4))
            {
                target_pixel[0] = source_pixel[2];
                target_pixel[1] = source_pixel[1];
                target_pixel[2] = source_pixel[0];
                target_pixel[3] = source_pixel[3];
            }
        }
        Ok(target)
    }

    fn last_error_message() -> String {
        let error = unsafe { dlerror() };
        if error.is_null() {
            return "unknown dynamic loader error".to_string();
        }
        unsafe { CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned()
    }

    impl std::fmt::Debug for PdfiumLibrary {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PdfiumLibrary")
                .field("path", &self.path)
                .finish_non_exhaustive()
        }
    }
}

#[cfg(not(unix))]
mod sys {
    use std::path::Path;

    use super::PdfiumBackendError;

    pub(super) struct PdfiumLibrary;

    impl PdfiumLibrary {
        pub(super) unsafe fn open(_path: &Path) -> Result<Self, PdfiumBackendError> {
            Err(PdfiumBackendError::UnsupportedPlatform)
        }

        pub(super) unsafe fn init(&self) {}

        pub(super) unsafe fn destroy(&self) {}

        pub(super) unsafe fn get_last_error(&self) -> u32 {
            0
        }

        pub(super) unsafe fn render_first_page_rgba(
            &self,
            _bytes: &[u8],
            _options: &ThumbnailOptions,
        ) -> Result<Thumbnail, ThumbnailError> {
            Err(ThumbnailError::Unsupported)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn backend_name_should_be_stable() {
        let backend = PdfiumBackend::new("/tmp/libpdfium.dylib");

        assert_eq!(backend.backend_name(), "pdfium");
    }

    #[test]
    fn from_env_should_report_missing_library_path() {
        env::remove_var(PDFIUM_LIBRARY_ENV);

        let error = PdfiumBackend::from_env().expect_err("missing env should fail");

        assert_eq!(error.to_string(), "PDFRUST_PDFIUM_LIBRARY is not set");
    }

    #[test]
    fn new_should_keep_library_path() {
        let path = PathBuf::from("/tmp/libpdfium.dylib");
        let backend = PdfiumBackend::new(&path);

        assert_eq!(backend.library_path(), path.as_path());
    }

    #[test]
    fn render_should_report_malformed_when_file_cannot_be_read() {
        let backend = PdfiumBackend::new("/tmp/libpdfium.dylib");
        let options = ThumbnailOptions::default();
        let source = PdfSource::from_path(Path::new("/tmp/does-not-exist.pdf"));

        let error = backend
            .render(source, &options)
            .expect_err("missing file should fail");

        assert_eq!(error, ThumbnailError::Malformed);
    }
}
