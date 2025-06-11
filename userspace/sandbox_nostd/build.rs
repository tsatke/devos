use std::env::var;
use std::fs;
use std::path::Path;

fn main() {
    let libmuffin_artifact_dir = var("CARGO_STATICLIB_DIR_LIBMUFFIN").unwrap();
    let libmuffin_artifact = var("CARGO_STATICLIB_FILE_LIBMUFFIN_muffin").unwrap();
    let libmuffin_path = Path::new(&libmuffin_artifact);
    let libmuffin_newpath = libmuffin_path.with_file_name("libmuffin.a");
    fs::copy(&libmuffin_path, &libmuffin_newpath).unwrap_or_else(|_| {
        panic!(
            "should be able to copy from {} to {}",
            libmuffin_path.display(),
            libmuffin_newpath.display(),
        )
    });
    println!("cargo:rustc-link-search={}", libmuffin_artifact_dir);

    println!("cargo:rustc-link-lib=muffin");
}
