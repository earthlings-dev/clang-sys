// SPDX-License-Identifier: Apache-2.0

//================================================
// Macros
//================================================

#[cfg(feature = "runtime")]
macro_rules! link {
    (
        @LOAD:
        $(#[doc=$doc:expr])*
        #[cfg($cfg:meta)]
        fn $name:ident($($pname:ident: $pty:ty), *) $(-> $ret:ty)*
    ) => (
        $(#[doc=$doc])*
        #[cfg($cfg)]
        pub fn $name(library: &mut super::SharedLibrary) {
            let symbol = unsafe { library.library.get(stringify!($name).as_bytes()) }.ok();
            library.functions.$name = match symbol {
                Some(s) => *s,
                None => None,
            };
        }

        #[cfg(not($cfg))]
        pub fn $name(_: &mut super::SharedLibrary) {}
    );

    (
        @LOAD:
        fn $name:ident($($pname:ident: $pty:ty), *) $(-> $ret:ty)*
    ) => (
        link!(@LOAD: #[cfg(feature = "runtime")] fn $name($($pname: $pty), *) $(-> $ret)*);
    );

    (
        $(
            $(#[doc=$doc:expr] #[cfg($cfg:meta)])*
            pub fn $name:ident($($pname:ident: $pty:ty), *) $(-> $ret:ty)*;
        )+
    ) => (
        use std::cell::{RefCell};
        use std::fmt;
        use std::sync::{Arc};
        use std::path::{Path, PathBuf};

        /// The (minimum) version of a `libclang` shared library.
        #[allow(missing_docs)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Version {
            V3_5 = 35,
            V3_6 = 36,
            V3_7 = 37,
            V3_8 = 38,
            V3_9 = 39,
            V4_0 = 40,
            V5_0 = 50,
            V6_0 = 60,
            V7_0 = 70,
            V8_0 = 80,
            V9_0 = 90,
            V11_0 = 110,
            V12_0 = 120,
            V16_0 = 160,
            V17_0 = 170,
            V18_0 = 180,
            V19_0 = 190,
            V20_0 = 200,
            V21_0 = 210,
            V22_0 = 220,
            V23_0 = 230,
        }

        impl fmt::Display for Version {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use Version::*;
                match self {
                    V3_5 => write!(f, "3.5.x"),
                    V3_6 => write!(f, "3.6.x"),
                    V3_7 => write!(f, "3.7.x"),
                    V3_8 => write!(f, "3.8.x"),
                    V3_9 => write!(f, "3.9.x"),
                    V4_0 => write!(f, "4.0.x"),
                    V5_0 => write!(f, "5.0.x"),
                    V6_0 => write!(f, "6.0.x"),
                    V7_0 => write!(f, "7.0.x"),
                    V8_0 => write!(f, "8.0.x"),
                    V9_0 => write!(f, "9.0.x - 10.0.x"),
                    V11_0 => write!(f, "11.0.x"),
                    V12_0 => write!(f, "12.0.x - 15.0.x"),
                    V16_0 => write!(f, "16.0.x"),
                    V17_0 => write!(f, "17.0.x"),
                    V18_0 => write!(f, "18.0.x"),
                    V19_0 => write!(f, "19.0.x"),
                    V20_0 => write!(f, "20.0.x"),
                    V21_0 => write!(f, "21.0.x"),
                    V22_0 => write!(f, "22.0.x"),
                    V23_0 => write!(f, "23.0.x or later"),
                }
            }
        }

        /// The set of functions loaded dynamically.
        #[derive(Debug, Default)]
        pub struct Functions {
            $(
                $(#[doc=$doc] #[cfg($cfg)])*
                pub $name: Option<unsafe extern "C" fn($($pname: $pty), *) $(-> $ret)*>,
            )+
        }

        /// A dynamically loaded instance of the `libclang` library.
        #[derive(Debug)]
        pub struct SharedLibrary {
            pub(crate) library: libloading::Library,
            pub(crate) path: PathBuf,
            pub functions: Functions,
        }

        impl SharedLibrary {
            fn new(library: libloading::Library, path: PathBuf) -> Self {
                Self { library, path, functions: Functions::default() }
            }

            /// Returns the path to this `libclang` shared library.
            pub fn path(&self) -> &Path {
                &self.path
            }

            /// Returns the (minimum) version of this `libclang` shared library.
            ///
            /// This method uses a hybrid detection strategy:
            ///
            /// 1. **Marker function detection**: Checks for unique functions introduced
            ///    in specific versions (fast, works for v19, v20, v21)
            /// 2. **Version string parsing**: Falls back to parsing `clang_getClangVersion()`
            ///    for accurate detection of all versions (v17-v23+)
            ///
            /// # Returns
            ///
            /// - `Some(Version::VXX_0)` - The detected Clang version
            /// - `None` - Version too old to be supported (v3.4 or earlier)
            ///
            /// # Version Support
            ///
            /// - **Clang 23.x**: Fully detected via version string parsing
            /// - **Clang 22.x**: Fully detected via version string parsing
            /// - **Clang 21.x**: Detected via `clang_getFullyQualifiedName` marker + string
            /// - **Clang 20.x**: Detected via `clang_getOffsetOfBase` marker
            /// - **Clang 19.x**: Detected via `clang_Cursor_getBinaryOpcode` marker
            /// - **Clang 18.x**: Fully detected via version string parsing
            /// - **Clang 17.x**: Detected via `clang_CXXMethod_isExplicit` marker + string
            /// - **Clang 16.x and older**: Detected via their respective markers
            ///
            /// # Examples
            ///
            /// ```no_run
            /// # #[cfg(feature = "runtime")]
            /// # fn example() {
            /// # use clang_sys::{load, get_library, Version};
            /// load().expect("Failed to load libclang");
            /// let library = get_library().expect("Library not loaded");
            /// match library.version() {
            ///     Some(Version::V23_0) => println!("Clang 23.x detected"),
            ///     Some(Version::V22_0) => println!("Clang 22.x detected"),
            ///     Some(v) => println!("Clang version: {}", v),
            ///     None => println!("Unsupported old version"),
            /// }
            /// # }
            /// ```
            pub fn version(&self) -> Option<Version> {
                /// Helper macro to check if a marker function exists in the library.
                ///
                /// If the function exists, immediately returns the specified version.
                /// This provides fast detection for versions with unique marker functions.
                macro_rules! check {
                    ($fn:expr, $version:ident) => {
                        // SAFETY: Symbol lookup is safe. Library is valid and loaded.
                        if self.library.get::<unsafe extern "C" fn()>($fn).is_ok() {
                            return Some(Version::$version);
                        }
                    };
                }

                // SAFETY: All symbol lookups and function calls are on the valid,
                // loaded libclang library stored in self.library.
                unsafe {
                    // Version detection strategy: ordered newest to oldest.
                    // Uses marker functions for fast detection, with version string
                    // parsing as fallback for accurate detection of all versions.

                    // Clang 21.0+: Added `clang_getFullyQualifiedName` and GCC assembly API.
                    // For v21+, we parse the version string to distinguish v21/v22/v23.
                    // SAFETY: Symbol lookup is safe.
                    if self.library.get::<unsafe extern "C" fn()>(b"clang_getFullyQualifiedName").is_ok() {
                        // SAFETY: Library is valid and loaded. version_from_string
                        // performs its own safety checks on all FFI calls.
                        return self.version_from_string().or(Some(Version::V21_0));
                    }

                    // Clang 20.0: Added base class introspection via `clang_getOffsetOfBase`.
                    check!(b"clang_getOffsetOfBase", V20_0);

                    // Clang 19.0: Added binary operator introspection.
                    check!(b"clang_Cursor_getBinaryOpcode", V19_0);

                    // Clang 17.0+: Added C++ method classification via `clang_CXXMethod_isExplicit`.
                    // For v17/v18, we parse the version string to distinguish them accurately.
                    // Clang 18 added no unique public C API functions (only enum values).
                    // SAFETY: Symbol lookup is safe.
                    if self.library.get::<unsafe extern "C" fn()>(b"clang_CXXMethod_isExplicit").is_ok() {
                        // SAFETY: Library is valid and loaded. version_from_string
                        // performs its own safety checks on all FFI calls.
                        return self.version_from_string().or(Some(Version::V17_0));
                    }

                    // Clang 16.0: Added copy assignment operator checking.
                    check!(b"clang_CXXMethod_isCopyAssignmentOperator", V16_0);

                    // Clang 12.0: Added variable declaration initializer access.
                    check!(b"clang_Cursor_getVarDeclInitializer", V12_0);

                    // Clang 11.0: Added value type access.
                    check!(b"clang_Type_getValueType", V11_0);

                    // Clang 9.0: Added anonymous record declaration checking.
                    check!(b"clang_Cursor_isAnonymousRecordDecl", V9_0);

                    // Clang 8.0: Added Objective-C property getter name access.
                    check!(b"clang_Cursor_getObjCPropertyGetterName", V8_0);

                    // Clang 7.0: Added real path name access for files.
                    check!(b"clang_File_tryGetRealPathName", V7_0);

                    // Clang 6.0: Added invocation emission path option.
                    check!(b"clang_CXIndex_setInvocationEmissionPathOption", V6_0);

                    // Clang 5.0: Added external symbol checking.
                    check!(b"clang_Cursor_isExternalSymbol", V5_0);

                    // Clang 4.0: Added evaluation result as long long.
                    check!(b"clang_EvalResult_getAsLongLong", V4_0);

                    // Clang 3.9: Added C++ constructor conversion checking.
                    check!(b"clang_CXXConstructor_isConvertingConstructor", V3_9);

                    // Clang 3.8: Added C++ field mutability checking.
                    check!(b"clang_CXXField_isMutable", V3_8);

                    // Clang 3.7: Added field offset access.
                    check!(b"clang_Cursor_getOffsetOfField", V3_7);

                    // Clang 3.6: Added storage class access.
                    check!(b"clang_Cursor_getStorageClass", V3_6);

                    // Clang 3.5: Added template argument counting.
                    check!(b"clang_Type_getNumTemplateArguments", V3_5);
                }

                // No marker function matched and version string parsing failed or not available.
                // This indicates a version older than 3.5 or an unsupported configuration.
                None
            }

            /// Parse version from `clang_getClangVersion()` string.
            ///
            /// This method provides accurate version detection for all Clang versions,
            /// including those that don't introduce unique marker functions in the
            /// C API (such as v18, v22, and v23).
            ///
            /// The version string format is typically: `"clang version MAJOR.MINOR.PATCH"`
            /// (e.g., `"clang version 23.1.0"`).
            ///
            /// # Returns
            ///
            /// - `Some(Version::VXX_0)` if the version can be successfully parsed
            /// - `None` if version parsing fails or the version is unsupported
            ///
            /// # Safety
            ///
            /// This function calls unsafe libclang C FFI functions and must only be
            /// called with a valid, loaded libclang library. The caller must ensure:
            ///
            /// - `self.library` contains a valid libloading::Library instance
            /// - The library exports the required functions: `clang_getClangVersion`,
            ///   `clang_getCString`, and `clang_disposeString`
            /// - The library remains loaded for the duration of this call
            unsafe fn version_from_string(&self) -> Option<Version> {
                use std::ffi::CStr;
                use std::os::raw::c_char;

                // Local copy of CXString to avoid module path issues in the macro.
                // This must match the ABI layout of the actual CXString in libclang.
                #[repr(C)]
                #[derive(Copy, Clone)]
                struct CXString {
                    /// Opaque data pointer managed by libclang
                    data: *const std::os::raw::c_void,
                    /// Internal flags used by libclang for memory management
                    private_flags: std::os::raw::c_uint,
                }

                // SAFETY: All operations are FFI calls to functions exported by the
                // loaded libclang library. We verify each function exists before calling.
                // CXString memory is properly disposed via clang_disposeString.
                unsafe {
                    // Get the version function from the loaded library.
                    // SAFETY: Library is valid and loaded. Symbol lookup is safe.
                    let get_version = self.library
                        .get::<unsafe extern "C" fn() -> CXString>(b"clang_getClangVersion")
                        .ok()?;

                    // SAFETY: Function pointer is valid, takes no arguments.
                    let version_cxstring = get_version();

                    // Get the C string accessor function.
                    // SAFETY: Library is valid and loaded. Symbol lookup is safe.
                    let get_cstring = self.library
                        .get::<unsafe extern "C" fn(CXString) -> *const c_char>(b"clang_getCString")
                        .ok()?;

                    // SAFETY: version_cxstring is a valid CXString returned from libclang.
                    let c_str_ptr = get_cstring(version_cxstring);
                    if c_str_ptr.is_null() {
                        return None;
                    }

                    // SAFETY: c_str_ptr is non-null and points to a valid C string
                    // managed by libclang. The string remains valid until we dispose
                    // the CXString.
                    let version_str = CStr::from_ptr(c_str_ptr).to_str().ok()?;

                    // Parse "clang version 23.1.0" or similar.
                    // Expected format: "clang version MAJOR.MINOR.PATCH"
                    // We extract only the MAJOR version for our coarse-grained detection.
                    let major = version_str
                        .split_whitespace()
                        .nth(2)?  // Extract "23.1.0" from "clang version 23.1.0"
                        .split('.')
                        .next()?  // Extract "23" from "23.1.0"
                        .parse::<u32>()
                        .ok()?;

                    // Dispose the CXString to free libclang-managed memory.
                    // SAFETY: Library is valid. Symbol lookup is safe.
                    let dispose = self.library
                        .get::<unsafe extern "C" fn(CXString)>(b"clang_disposeString")
                        .ok()?;

                    // SAFETY: version_cxstring is a valid CXString that hasn't been
                    // disposed yet. This is the standard cleanup for CXString values.
                    dispose(version_cxstring);

                    // Map LLVM/Clang major version to our Version enum.
                    // Versions are grouped to match the granularity of our enum variants.
                    match major {
                        23.. => Some(Version::V23_0),      // Clang 23.x and newer
                        22 => Some(Version::V22_0),         // Clang 22.x
                        21 => Some(Version::V21_0),         // Clang 21.x
                        20 => Some(Version::V20_0),         // Clang 20.x
                        19 => Some(Version::V19_0),         // Clang 19.x
                        18 => Some(Version::V18_0),         // Clang 18.x
                        17 => Some(Version::V17_0),         // Clang 17.x
                        16 => Some(Version::V16_0),         // Clang 16.x
                        12..=15 => Some(Version::V12_0),    // Clang 12.x - 15.x
                        11 => Some(Version::V11_0),         // Clang 11.x
                        9 | 10 => Some(Version::V9_0),      // Clang 9.x - 10.x
                        8 => Some(Version::V8_0),           // Clang 8.x
                        7 => Some(Version::V7_0),           // Clang 7.x
                        6 => Some(Version::V6_0),           // Clang 6.x
                        5 => Some(Version::V5_0),           // Clang 5.x
                        4 => Some(Version::V4_0),           // Clang 4.x
                        _ => None,                          // Unsupported (3.x or unknown)
                    }
                }
            }
        }

        thread_local!(static LIBRARY: RefCell<Option<Arc<SharedLibrary>>> = RefCell::new(None));

        /// Returns whether a `libclang` shared library is loaded on this thread.
        pub fn is_loaded() -> bool {
            LIBRARY.with(|l| l.borrow().is_some())
        }

        fn with_library<T, F>(f: F) -> Option<T> where F: FnOnce(&SharedLibrary) -> T {
            LIBRARY.with(|l| {
                match l.borrow().as_ref() {
                    Some(library) => Some(f(&library)),
                    _ => None,
                }
            })
        }

        $(
            #[cfg_attr(clippy, allow(clippy::missing_safety_doc))]
            #[cfg_attr(clippy, allow(clippy::too_many_arguments))]
            $(#[doc=$doc] #[cfg($cfg)])*
            pub unsafe fn $name($($pname: $pty), *) $(-> $ret)* {
                let f = with_library(|library| {
                    if let Some(function) = library.functions.$name {
                        function
                    } else {
                        panic!(
                            r#"
A `libclang` function was called that is not supported by the loaded `libclang` instance.

    called function = `{0}`
    loaded `libclang` instance = {1}

The minimum `libclang` requirement for this particular function can be found here:
https://docs.rs/clang-sys/latest/clang_sys/{0}/index.html

Instructions for installing `libclang` can be found here:
https://rust-lang.github.io/rust-bindgen/requirements.html
"#,
                            stringify!($name),
                            library
                                .version()
                                .map(|v| format!("{}", v))
                                .unwrap_or_else(|| "unsupported version".into()),
                        );
                    }
                }).expect("a `libclang` shared library is not loaded on this thread");
                unsafe { f($($pname), *) }
            }

            $(#[doc=$doc] #[cfg($cfg)])*
            pub mod $name {
                pub fn is_loaded() -> bool {
                    super::with_library(|l| l.functions.$name.is_some()).unwrap_or(false)
                }
            }
        )+

        mod load {
            $(link!(@LOAD: $(#[cfg($cfg)])* fn $name($($pname: $pty), *) $(-> $ret)*);)+
        }

        /// Loads a `libclang` shared library and returns the library instance.
        ///
        /// This function does not attempt to load any functions from the shared library. The caller
        /// is responsible for loading the functions they require.
        ///
        /// # Failures
        ///
        /// * a `libclang` shared library could not be found
        /// * the `libclang` shared library could not be opened
        pub fn load_manually() -> Result<SharedLibrary, String> {
            #[allow(dead_code)]
            mod build {
                include!(concat!(env!("OUT_DIR"), "/macros.rs"));
                pub mod common { include!(concat!(env!("OUT_DIR"), "/common.rs")); }
                pub mod dynamic { include!(concat!(env!("OUT_DIR"), "/dynamic.rs")); }
            }

            let (directory, filename) = build::dynamic::find(true)?;
            let path = directory.join(filename);

            unsafe {
                let library = libloading::Library::new(&path).map_err(|e| {
                    format!(
                        "the `libclang` shared library at {} could not be opened: {}",
                        path.display(),
                        e,
                    )
                });

                let mut library = SharedLibrary::new(library?, path);
                $(load::$name(&mut library);)+
                Ok(library)
            }
        }

        /// Loads a `libclang` shared library for use in the current thread.
        ///
        /// This functions attempts to load all the functions in the shared library. Whether a
        /// function has been loaded can be tested by calling the `is_loaded` function on the
        /// module with the same name as the function (e.g., `clang_createIndex::is_loaded()` for
        /// the `clang_createIndex` function).
        ///
        /// # Failures
        ///
        /// * a `libclang` shared library could not be found
        /// * the `libclang` shared library could not be opened
        #[allow(dead_code)]
        pub fn load() -> Result<(), String> {
            let library = Arc::new(load_manually()?);
            LIBRARY.with(|l| *l.borrow_mut() = Some(library));
            Ok(())
        }

        /// Unloads the `libclang` shared library in use in the current thread.
        ///
        /// # Failures
        ///
        /// * a `libclang` shared library is not in use in the current thread
        pub fn unload() -> Result<(), String> {
            let library = set_library(None);
            if library.is_some() {
                Ok(())
            } else {
                Err("a `libclang` shared library is not in use in the current thread".into())
            }
        }

        /// Returns the library instance stored in TLS.
        ///
        /// This functions allows for sharing library instances between threads.
        pub fn get_library() -> Option<Arc<SharedLibrary>> {
            LIBRARY.with(|l| l.borrow_mut().clone())
        }

        /// Sets the library instance stored in TLS and returns the previous library.
        ///
        /// This functions allows for sharing library instances between threads.
        pub fn set_library(library: Option<Arc<SharedLibrary>>) -> Option<Arc<SharedLibrary>> {
            LIBRARY.with(|l| mem::replace(&mut *l.borrow_mut(), library))
        }
    )
}

#[cfg(not(feature = "runtime"))]
macro_rules! link {
    (
        $(
            $(#[doc=$doc:expr] #[cfg($cfg:meta)])*
            pub fn $name:ident($($pname:ident: $pty:ty), *) $(-> $ret:ty)*;
        )+
    ) => (
        unsafe extern "C" {
            $(
                $(#[doc=$doc] #[cfg($cfg)])*
                pub fn $name($($pname: $pty), *) $(-> $ret)*;
            )+
        }

        $(
            $(#[doc=$doc] #[cfg($cfg)])*
            pub mod $name {
                pub fn is_loaded() -> bool { true }
            }
        )+
    )
}
