use std::ptr;

use clang_sys::*;

fn parse() {
    unsafe {
        let index = clang_createIndex(0, 0);
        assert!(!index.is_null());

        let tu = clang_parseTranslationUnit(
            index,
            c"tests/header.h".as_ptr(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            0,
            0,
        );
        assert!(!tu.is_null());
    }
}

#[cfg(feature = "runtime")]
#[test]
fn test() {
    load().unwrap();
    let library = get_library().unwrap();
    println!("{:?} ({:?})", library.version(), library.path());
    parse();
    unload().unwrap();
}

#[cfg(not(feature = "runtime"))]
#[test]
fn test() {
    parse();
}

#[test]
fn test_support() {
    let clang = support::Clang::find(None, &[]).unwrap();
    println!("{:?}", clang);
}

#[test]
fn test_support_target() {
    let args = &["--target".into(), "x86_64-unknown-linux-gnu".into()];
    let clang = support::Clang::find(None, args).unwrap();
    println!("{:?}", clang);
}

#[cfg(feature = "runtime")]
#[test]
fn test_support_runtime() {
    load().unwrap();
    let library = get_library().unwrap();
    let clang = support::Clang::find(None, &[]).unwrap();
    println!("Library path: {}", library.path().display());
    println!("Clang path:   {}", clang.path.display());
    unload().unwrap();
}
