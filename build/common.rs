// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use glob::{MatchOptions, Pattern};

//================================================
// Commands
//================================================

thread_local! {
    /// The errors encountered by the build script while executing commands.
    static COMMAND_ERRORS: RefCell<HashMap<String, Vec<String>>> = RefCell::default();
}

/// Adds an error encountered by the build script while executing a command.
fn add_command_error(name: &str, path: &str, arguments: &[&str], message: String) {
    COMMAND_ERRORS.with(|e| {
        e.borrow_mut().entry(name.into()).or_default().push(format!(
            "couldn't execute `{} {}` (path={}) ({})",
            name,
            arguments.join(" "),
            path,
            message,
        ))
    });
}

/// A struct that prints the errors encountered by the build script while
/// executing commands when dropped (unless explictly discarded).
///
/// This is handy because we only want to print these errors when the build
/// script fails to link to an instance of `libclang`. For example, if
/// `llvm-config` couldn't be executed but an instance of `libclang` was found
/// anyway we don't want to pollute the build output with irrelevant errors.
#[derive(Default)]
pub struct CommandErrorPrinter {
    discard: bool,
}

impl CommandErrorPrinter {
    pub fn discard(mut self) {
        self.discard = true;
    }
}

impl Drop for CommandErrorPrinter {
    fn drop(&mut self) {
        if self.discard {
            return;
        }

        let errors = COMMAND_ERRORS.with(|e| e.borrow().clone());

        if let Some(errors) = errors.get("llvm-config") {
            println!(
                "cargo:warning=could not execute `llvm-config` one or more \
                times, if the LLVM_CONFIG_PATH environment variable is set to \
                a full path to valid `llvm-config` executable it will be used \
                to try to find an instance of `libclang` on your system: {}",
                errors
                    .iter()
                    .map(|e| format!("\"{}\"", e))
                    .collect::<Vec<_>>()
                    .join("\n  "),
            )
        }

        if let Some(errors) = errors.get("xcode-select") {
            println!(
                "cargo:warning=could not execute `xcode-select` one or more \
                times, if a valid instance of this executable is on your PATH \
                it will be used to try to find an instance of `libclang` on \
                your system: {}",
                errors
                    .iter()
                    .map(|e| format!("\"{}\"", e))
                    .collect::<Vec<_>>()
                    .join("\n  "),
            )
        }
    }
}

#[cfg(test)]
type RunCommandFn = Box<dyn Fn(&str, &str, &[&str]) -> Option<String> + Send + Sync + 'static>;

#[cfg(test)]
lazy_static::lazy_static! {
    pub static ref RUN_COMMAND_MOCK: std::sync::Mutex<Option<RunCommandFn>> =
        std::sync::Mutex::new(None);
}

/// Executes a command and returns the `stdout` output if the command was
/// successfully executed (errors are added to `COMMAND_ERRORS`).
fn run_command(name: &str, path: &str, arguments: &[&str]) -> Option<String> {
    #[cfg(test)]
    if let Some(command) = &*RUN_COMMAND_MOCK.lock().unwrap() {
        return command(name, path, arguments);
    }

    let output = match Command::new(path).args(arguments).output() {
        Ok(output) => output,
        Err(error) => {
            let message = format!("error: {}", error);
            add_command_error(name, path, arguments, message);
            return None;
        }
    };

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let message = format!("exit code: {}", output.status);
        add_command_error(name, path, arguments, message);
        None
    }
}

/// Resolves the path to the `llvm-config` executable.
///
/// Uses the following strategy in order:
/// 1. `LLVM_CONFIG_PATH` environment variable (if set)
/// 2. Auto-detection in well-known platform-specific directories (cached)
/// 3. Falls back to `"llvm-config"` (relying on PATH lookup)
fn resolve_llvm_config_path() -> String {
    if let Ok(path) = env::var("LLVM_CONFIG_PATH") {
        return path;
    }

    if let Some(path) = find_llvm_config() {
        return path;
    }

    "llvm-config".into()
}

thread_local! {
    /// Cached result from `llvm-config` auto-detection.
    /// - `None`: not yet searched
    /// - `Some(None)`: searched but not found
    /// - `Some(Some(path))`: found at the given path
    static LLVM_CONFIG_PATH_CACHE: RefCell<Option<Option<String>>> = const { RefCell::new(None) };
}

/// Returns the target Clang major version derived from the highest enabled
/// `clang_X_0` feature flag. Returns `None` if no version feature is enabled.
fn get_target_clang_version() -> Option<u32> {
    // Features are cumulative (clang_21_0 implies clang_20_0, etc.), so the
    // highest enabled feature determines the target version.
    if cfg!(feature = "clang_23_0") {
        Some(23)
    } else if cfg!(feature = "clang_22_0") {
        Some(22)
    } else if cfg!(feature = "clang_21_0") {
        Some(21)
    } else if cfg!(feature = "clang_20_0") {
        Some(20)
    } else if cfg!(feature = "clang_19_0") {
        Some(19)
    } else if cfg!(feature = "clang_18_0") {
        Some(18)
    } else if cfg!(feature = "clang_17_0") {
        Some(17)
    } else if cfg!(feature = "clang_16_0") {
        Some(16)
    } else if cfg!(feature = "clang_15_0") {
        Some(15)
    } else if cfg!(feature = "clang_14_0") {
        Some(14)
    } else if cfg!(feature = "clang_13_0") {
        Some(13)
    } else if cfg!(feature = "clang_12_0") {
        Some(12)
    } else if cfg!(feature = "clang_11_0") {
        Some(11)
    } else if cfg!(feature = "clang_10_0") {
        Some(10)
    } else if cfg!(feature = "clang_9_0") {
        Some(9)
    } else if cfg!(feature = "clang_8_0") {
        Some(8)
    } else if cfg!(feature = "clang_7_0") {
        Some(7)
    } else if cfg!(feature = "clang_6_0") {
        Some(6)
    } else if cfg!(feature = "clang_5_0") {
        Some(5)
    } else if cfg!(feature = "clang_4_0") {
        Some(4)
    } else if cfg!(any(
        feature = "clang_3_9",
        feature = "clang_3_8",
        feature = "clang_3_7",
        feature = "clang_3_6",
        feature = "clang_3_5",
    )) {
        Some(3)
    } else {
        None
    }
}

/// Searches well-known platform-specific directories for an `llvm-config`
/// executable. Results are cached across calls.
///
/// Prefers the installation matching the target Clang version (derived from
/// the highest enabled `clang_X_0` feature flag). Falls back to the highest
/// available version if no exact match is found.
fn find_llvm_config() -> Option<String> {
    LLVM_CONFIG_PATH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(ref cached) = *cache {
            return cached.clone();
        }

        let result = find_llvm_config_uncached();
        *cache = Some(result.clone());
        result
    })
}

/// Performs the actual filesystem search for `llvm-config`.
fn find_llvm_config_uncached() -> Option<String> {
    // Don't auto-detect during tests, which use mocked commands.
    if test!() {
        return None;
    }

    let target_version = get_target_clang_version();

    // If llvm-config is already findable on PATH, check if its version
    // matches our target before accepting it.
    if let Ok(output) = Command::new("llvm-config").arg("--version").output()
        && output.status.success()
    {
        let version_str = String::from_utf8_lossy(&output.stdout);
        let path_major = version_str
            .trim()
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok());

        match (target_version, path_major) {
            // No feature flag set, or version matches -> use PATH.
            (None, _) | (_, None) => return Some("llvm-config".into()),
            (Some(target), Some(found)) if target == found => {
                return Some("llvm-config".into());
            }
            // Version mismatch -> fall through to search for the right one.
            _ => {}
        }
    }

    let patterns: Vec<&str> = if target_os!("macos") {
        vec![
            // Homebrew on Apple Silicon (arm64)
            "/opt/homebrew/opt/llvm/bin/llvm-config",
            "/opt/homebrew/opt/llvm@*/bin/llvm-config",
            // Homebrew on Intel (x86_64)
            "/usr/local/opt/llvm/bin/llvm-config",
            "/usr/local/opt/llvm@*/bin/llvm-config",
            // MacPorts
            "/opt/local/libexec/llvm-*/bin/llvm-config",
        ]
    } else if target_os!("linux") || target_os!("freebsd") {
        vec![
            // Versioned executables (Debian/Ubuntu packages)
            "/usr/bin/llvm-config-*",
            // Standard LLVM installations
            "/usr/lib/llvm-*/bin/llvm-config",
            // Manual /usr/local installations
            "/usr/local/llvm*/bin/llvm-config",
        ]
    } else if target_os!("windows") {
        vec![
            "C:\\Program Files\\LLVM\\bin\\llvm-config.exe",
            "C:\\Program Files*\\LLVM\\bin\\llvm-config.exe",
        ]
    } else if target_os!("illumos") {
        vec!["/opt/ooce/llvm-*/bin/llvm-config"]
    } else {
        vec![]
    };

    let mut candidates: Vec<(PathBuf, Vec<u32>)> = Vec::new();

    for pattern in patterns {
        if let Ok(paths) = glob::glob(pattern) {
            for path in paths.filter_map(Result::ok) {
                if path.exists() {
                    let mut version = extract_version_from_llvm_path(&path);

                    // For unversioned paths (e.g., Homebrew's `llvm` formula),
                    // resolve the actual version by running the executable.
                    if version == [999]
                        && let Some(real) = query_llvm_config_version(&path)
                    {
                        version = vec![real];
                    }

                    candidates.push((path, version));
                }
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // If a target version is specified via feature flags, require a
    // candidate whose major version matches. Hard-fails if the
    // requested version is not installed.
    if let Some(target) = target_version {
        if let Some((path, _)) = candidates
            .iter()
            .find(|(_, v)| v.first().copied() == Some(target))
        {
            let path_str = path.to_string_lossy().into_owned();
            println!(
                "cargo:warning=clang-sys: auto-detected llvm-config (v{}) at: {}",
                target, path_str
            );
            return Some(path_str);
        }

        // No matching version found. Don't fall back to a different version.
        let available: Vec<String> = candidates
            .iter()
            .filter_map(|(_, v)| v.first().map(|n| n.to_string()))
            .collect();
        println!(
            "cargo:warning=clang-sys: could not find llvm-config for v{} \
             (available: {}). Install LLVM {} or set LLVM_CONFIG_PATH.",
            target,
            if available.is_empty() {
                "none".into()
            } else {
                available.join(", ")
            },
            target,
        );
        None
    } else {
        // No version feature set. Use the highest available.
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        let (path, _) = &candidates[0];
        let path_str = path.to_string_lossy().into_owned();
        println!(
            "cargo:warning=clang-sys: auto-detected llvm-config at: {}",
            path_str
        );
        Some(path_str)
    }
}

/// Queries an `llvm-config` executable for its major version number.
fn query_llvm_config_version(path: &Path) -> Option<u32> {
    Command::new(path)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
        })
}

/// Extracts a version number from an `llvm-config` path for sorting purposes.
///
/// Recognizes patterns in path components and filenames:
/// - `llvm@17` → `[17]`
/// - `llvm-17` → `[17]`
/// - `llvm-config-17` → `[17]`
/// - Unversioned `llvm` → `[999]` (highest priority, typically the latest)
fn extract_version_from_llvm_path(path: &Path) -> Vec<u32> {
    // Check the filename for versioned llvm-config (e.g., llvm-config-17).
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && let Some(rest) = name.strip_prefix("llvm-config-")
    {
        let version: Vec<u32> = rest.split('.').filter_map(|p| p.parse().ok()).collect();
        if !version.is_empty() {
            return version;
        }
        // Has a suffix but it's not a version number; deprioritize.
        return vec![0];
    }

    // Check path components for versioned directory names.
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();

        // Homebrew-style: llvm@17
        if let Some(rest) = s.strip_prefix("llvm@") {
            let version: Vec<u32> = rest.split('.').filter_map(|p| p.parse().ok()).collect();
            if !version.is_empty() {
                return version;
            }
        }

        // Package/directory-style: llvm-17
        if let Some(rest) = s.strip_prefix("llvm-") {
            let version: Vec<u32> = rest.split('.').filter_map(|p| p.parse().ok()).collect();
            if !version.is_empty() {
                return version;
            }
        }
    }

    // Unversioned "llvm" directory (e.g., Homebrew's latest formula) gets
    // highest priority since it typically represents the most recent version.
    vec![999]
}

/// Executes the `llvm-config` command and returns the `stdout` output if the
/// command was successfully executed (errors are added to `COMMAND_ERRORS`).
pub fn run_llvm_config(arguments: &[&str]) -> Option<String> {
    let path = resolve_llvm_config_path();
    run_command("llvm-config", &path, arguments)
}

/// Executes the `xcode-select` command and returns the `stdout` output if the
/// command was successfully executed (errors are added to `COMMAND_ERRORS`).
pub fn run_xcode_select(arguments: &[&str]) -> Option<String> {
    run_command("xcode-select", "xcode-select", arguments)
}

//================================================
// Search Directories
//================================================
// These search directories are listed in order of
// preference, so if multiple `libclang` instances
// are found when searching matching directories,
// the `libclang` instances from earlier
// directories will be preferred (though version
// takes precedence over location).
//================================================

/// `libclang` directory patterns for Haiku.
const DIRECTORIES_HAIKU: &[&str] = &[
    "/boot/home/config/non-packaged/develop/lib",
    "/boot/home/config/non-packaged/lib",
    "/boot/system/non-packaged/develop/lib",
    "/boot/system/non-packaged/lib",
    "/boot/system/develop/lib",
    "/boot/system/lib",
];

/// `libclang` directory patterns for Linux (and FreeBSD).
const DIRECTORIES_LINUX: &[&str] = &[
    "/usr/local/llvm*/lib*",
    "/usr/local/lib*/*/*",
    "/usr/local/lib*/*",
    "/usr/local/lib*",
    "/usr/lib*/*/*",
    "/usr/lib*/*",
    "/usr/lib*",
];

/// `libclang` directory patterns for macOS.
const DIRECTORIES_MACOS: &[&str] = &[
    // Homebrew on Apple Silicon (arm64)
    "/opt/homebrew/opt/llvm*/lib",
    "/opt/homebrew/opt/llvm*/lib/llvm*/lib",
    // Homebrew on Intel (x86_64)
    "/usr/local/opt/llvm*/lib",
    "/usr/local/opt/llvm*/lib/llvm*/lib",
    // Apple Command Line Tools and Xcode
    "/Library/Developer/CommandLineTools/usr/lib",
    "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib",
    // MacPorts
    "/opt/local/libexec/llvm-*/lib",
];

/// `libclang` directory patterns for Windows.
///
/// The boolean indicates whether the directory pattern should be used when
/// compiling for an MSVC target environment.
const DIRECTORIES_WINDOWS: &[(&str, bool)] = &[
    // LLVM + Clang can be installed using Scoop (https://scoop.sh).
    // Other Windows package managers install LLVM + Clang to other listed
    // system-wide directories.
    ("C:\\Users\\*\\scoop\\apps\\llvm\\current\\lib", true),
    ("C:\\MSYS*\\MinGW*\\lib", false),
    ("C:\\MSYS*\\clang*\\lib", false),
    ("C:\\Program Files*\\LLVM\\lib", true),
    ("C:\\LLVM\\lib", true),
    // LLVM + Clang can be installed as a component of Visual Studio.
    // https://github.com/KyleMayes/clang-sys/issues/121
    (
        "C:\\Program Files*\\Microsoft Visual Studio\\*\\VC\\Tools\\Llvm\\**\\lib",
        true,
    ),
];

/// `libclang` directory patterns for illumos
const DIRECTORIES_ILLUMOS: &[&str] = &["/opt/ooce/llvm-*/lib", "/opt/ooce/clang-*/lib"];

//================================================
// Searching
//================================================

/// Finds the files in a directory that match one or more filename glob patterns
/// and returns the paths to and filenames of those files.
fn search_directory(directory: &Path, filenames: &[String]) -> Vec<(PathBuf, String)> {
    // Escape the specified directory in case it contains characters that have
    // special meaning in glob patterns (e.g., `[` or `]`).
    let directory = Pattern::escape(directory.to_str().unwrap());
    let directory = Path::new(&directory);

    // Join the escaped directory to the filename glob patterns to obtain
    // complete glob patterns for the files being searched for.
    let paths = filenames
        .iter()
        .map(|f| directory.join(f).to_str().unwrap().to_owned());

    // Prevent wildcards from matching path separators to ensure that the search
    // is limited to the specified directory.
    let mut options = MatchOptions::new();
    options.require_literal_separator = true;

    paths
        .map(|p| glob::glob_with(&p, options))
        .filter_map(Result::ok)
        .flatten()
        .filter_map(|p| {
            let path = p.ok()?;
            let filename = path.file_name()?.to_str().unwrap();

            // The `libclang_shared` library has been renamed to `libclang-cpp`
            // in Clang 10. This can cause instances of this library (e.g.,
            // `libclang-cpp.so.10`) to be matched by patterns looking for
            // instances of `libclang`.
            if filename.contains("-cpp.") {
                return None;
            }

            Some((path.parent().unwrap().to_owned(), filename.into()))
        })
        .collect::<Vec<_>>()
}

/// Finds the files in a directory (and any relevant sibling directories) that
/// match one or more filename glob patterns and returns the paths to and
/// filenames of those files.
fn search_directories(directory: &Path, filenames: &[String]) -> Vec<(PathBuf, String)> {
    let mut results = search_directory(directory, filenames);

    // On Windows, `libclang.dll` is usually found in the LLVM `bin` directory
    // while `libclang.lib` is usually found in the LLVM `lib` directory. To
    // keep things consistent with other platforms, only LLVM `lib` directories
    // are included in the backup search directory globs so we need to search
    // the LLVM `bin` directory here.
    if target_os!("windows") && directory.ends_with("lib") {
        let sibling = directory.parent().unwrap().join("bin");
        results.extend(search_directory(&sibling, filenames));
    }

    results
}

/// Finds the `libclang` static or dynamic libraries matching one or more
/// filename glob patterns and returns the paths to and filenames of those files.
pub fn search_libclang_directories(filenames: &[String], variable: &str) -> Vec<(PathBuf, String)> {
    // Search only the path indicated by the relevant environment variable
    // (e.g., `LIBCLANG_PATH`) if it is set.
    if let Ok(path) = env::var(variable).map(|d| Path::new(&d).to_path_buf()) {
        // Check if the path is a matching file.
        if let Some(parent) = path.parent() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let libraries = search_directories(parent, filenames);
            if libraries.iter().any(|(_, f)| f == filename) {
                return vec![(parent.into(), filename.into())];
            }
        }

        // Check if the path is directory containing a matching file.
        return search_directories(&path, filenames);
    }

    let mut found = vec![];

    // Search the `bin` and `lib` directories in the directory returned by
    // `llvm-config --prefix`.
    if let Some(output) = run_llvm_config(&["--prefix"]) {
        let directory = Path::new(output.lines().next().unwrap()).to_path_buf();
        found.extend(search_directories(&directory.join("bin"), filenames));
        found.extend(search_directories(&directory.join("lib"), filenames));
        found.extend(search_directories(&directory.join("lib64"), filenames));
    }

    // Search the toolchain directory in the directory returned by
    // `xcode-select --print-path`.
    if target_os!("macos")
        && let Some(output) = run_xcode_select(&["--print-path"])
    {
        let directory = Path::new(output.lines().next().unwrap()).to_path_buf();
        let directory = directory.join("Toolchains/XcodeDefault.xctoolchain/usr/lib");
        found.extend(search_directories(&directory, filenames));
    }

    // Search the directories in the `LD_LIBRARY_PATH` environment variable.
    if let Ok(path) = env::var("LD_LIBRARY_PATH") {
        for directory in env::split_paths(&path) {
            found.extend(search_directories(&directory, filenames));
        }
    }

    // Determine the `libclang` directory patterns.
    let directories: Vec<&str> = if target_os!("haiku") {
        DIRECTORIES_HAIKU.into()
    } else if target_os!("linux") || target_os!("freebsd") {
        DIRECTORIES_LINUX.into()
    } else if target_os!("macos") {
        DIRECTORIES_MACOS.into()
    } else if target_os!("windows") {
        let msvc = target_env!("msvc");
        DIRECTORIES_WINDOWS
            .iter()
            .filter(|d| d.1 || !msvc)
            .map(|d| d.0)
            .collect()
    } else if target_os!("illumos") {
        DIRECTORIES_ILLUMOS.into()
    } else {
        vec![]
    };

    // We use temporary directories when testing the build script so we'll
    // remove the prefixes that make the directories absolute.
    let directories = if test!() {
        directories
            .iter()
            .map(|d| {
                d.strip_prefix('/')
                    .or_else(|| d.strip_prefix("C:\\"))
                    .unwrap_or(d)
            })
            .collect::<Vec<_>>()
    } else {
        directories
    };

    // Search the directories provided by the `libclang` directory patterns.
    let mut options = MatchOptions::new();
    options.case_sensitive = false;
    options.require_literal_separator = true;
    for directory in directories.iter() {
        if let Ok(directories) = glob::glob_with(directory, options) {
            for directory in directories.filter_map(Result::ok).filter(|p| p.is_dir()) {
                found.extend(search_directories(&directory, filenames));
            }
        }
    }

    found
}
