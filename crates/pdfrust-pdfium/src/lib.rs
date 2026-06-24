//! Local PDFium backend for the thumbnail facade.

#![deny(unsafe_op_in_unsafe_fn)]

use std::env;
use std::fmt;
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
        _source: PdfSource<'_>,
        _options: &ThumbnailOptions,
    ) -> Result<Thumbnail, ThumbnailError> {
        Err(ThumbnailError::Unsupported)
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
    use std::os::raw::{c_char, c_int, c_ulong, c_void};
    use std::path::{Path, PathBuf};
    use std::ptr::NonNull;

    use super::PdfiumBackendError;

    const RTLD_NOW: c_int = 2;

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

    pub(super) struct PdfiumLibrary {
        path: PathBuf,
        handle: NonNull<c_void>,
        init_library: FpdfInitLibrary,
        destroy_library: FpdfDestroyLibrary,
        get_last_error: FpdfGetLastError,
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
}
