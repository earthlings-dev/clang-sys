// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use glob::Pattern;

use super::common;

//================================================
// Searching
//================================================

/// Clang static libraries required to link to `libclang` 3.5 and later.
const CLANG_LIBRARIES: &[&str] = &[
    "clang",
    "clangAST",
    "clangAnalysis",
    "clangBasic",
    "clangDriver",
    "clangEdit",
    "clangFrontend",
    "clangIndex",
    "clangLex",
    "clangParse",
    "clangRewrite",
    "clangSema",
    "clangSerialization",
];

/// Gets the name of an LLVM or Clang static library from a path.
fn get_library_name(path: &Path) -> Option<String> {
    path.file_stem().map(|p| {
        let string = p.to_string_lossy();
        if let Some(name) = string.strip_prefix("lib") {
            name.to_owned()
        } else {
            string.to_string()
        }
    })
}

/// Gets the LLVM static libraries required to link to `libclang`.
fn get_llvm_libraries() -> Vec<String> {
    common::run_llvm_config(&["--libs", "--link-static"])
        .unwrap()
        .split_whitespace()
        .filter_map(|p| {
            // Depending on the version of `llvm-config` in use, listed
            // libraries may be in one of two forms, a full path to the library
            // or simply prefixed with `-l`.
            if let Some(path) = p.strip_prefix("-l") {
                Some(path.into())
            } else {
                get_library_name(Path::new(p))
            }
        })
        .collect()
}

/// Gets the Clang static libraries required to link to `libclang`.
fn get_clang_libraries<P: AsRef<Path>>(directory: P) -> Vec<String> {
    // Escape the directory in case it contains characters that have special
    // meaning in glob patterns (e.g., `[` or `]`).
    let directory = Pattern::escape(directory.as_ref().to_str().unwrap());
    let directory = Path::new(&directory);

    let pattern = directory.join("libclang*.a").to_str().unwrap().to_owned();
    if let Ok(libraries) = glob::glob(&pattern) {
        libraries
            .filter_map(|l| l.ok().and_then(|l| get_library_name(&l)))
            .collect()
    } else {
        CLANG_LIBRARIES.iter().map(|l| (*l).to_string()).collect()
    }
}

/// Finds a directory containing LLVM and Clang static libraries and returns the
/// path to that directory.
///
/// This function searches for static libraries using multiple strategies:
/// 1. Look for `libclang.a` (monolithic static library - older LLVM builds)
/// 2. Look for `libclangBasic.a` (component static library - modern LLVM builds)
/// 3. Use `LIBCLANG_STATIC_PATH` environment variable if set
///
/// Modern LLVM installations (especially from package managers like Homebrew)
/// split libclang into component libraries rather than providing a monolithic
/// `libclang.a`. This function handles both styles transparently.
fn find() -> PathBuf {
    // Try to find either the monolithic library or a component library that
    // always exists in Clang static builds.
    let candidates = if target_os!("windows") {
        vec!["libclang.lib", "clangBasic.lib"]
    } else {
        vec!["libclang.a", "libclangBasic.a"]
    };

    let files = common::search_libclang_directories(
        &candidates.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "LIBCLANG_STATIC_PATH",
    );

    if let Some((directory, filename)) = files.into_iter().next() {
        // Log which marker file we found for debugging
        println!(
            "cargo:warning=found Clang static libraries using marker: {}",
            filename
        );
        directory
    } else {
        panic!(
            "could not find Clang static libraries (searched for {} or component libraries), \
            set LIBCLANG_STATIC_PATH to the directory containing libclang*.a files, see the \
            README for more information: \
            https://github.com/KyleMayes/clang-sys?tab=readme-ov-file#static",
            candidates.join(" or ")
        );
    }
}

//================================================
// Linking
//================================================

/// Finds and links to `libclang` static libraries.
pub fn link() {
    let cep = common::CommandErrorPrinter::default();

    let directory = find();

    // Specify required Clang static libraries.
    println!("cargo:rustc-link-search=native={}", directory.display());
    for library in get_clang_libraries(directory) {
        println!("cargo:rustc-link-lib=static={}", library);
    }

    // Determine the shared mode used by LLVM.
    let mode = common::run_llvm_config(&["--shared-mode"]).map(|m| m.trim().to_owned());
    let prefix = if mode.is_some_and(|m| m == "static") {
        "static="
    } else {
        ""
    };

    // Specify required LLVM static libraries.
    println!(
        "cargo:rustc-link-search=native={}",
        common::run_llvm_config(&["--libdir"]).unwrap().trim_end()
    );
    for library in get_llvm_libraries() {
        println!("cargo:rustc-link-lib={}{}", prefix, library);
    }

    // Specify required system libraries.
    // MSVC doesn't need this, as it tracks dependencies inside `.lib` files.
    if cfg!(target_os = "freebsd") {
        println!("cargo:rustc-flags=-l ffi -l ncursesw -l c++ -l z");
    } else if cfg!(any(target_os = "haiku", target_os = "linux")) {
        if cfg!(feature = "libcpp") {
            println!("cargo:rustc-flags=-l c++");
        } else {
            println!("cargo:rustc-flags=-l ffi -l ncursesw -l stdc++ -l z");
        }
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-flags=-l ffi -l ncurses -l c++ -l z");
    }

    cep.discard();
}
