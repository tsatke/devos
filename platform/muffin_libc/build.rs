use cbindgen::{Builder, Config};
use std::env;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_dir = Path::new(&crate_dir);

    let walker = WalkDir::new(crate_dir.join("src"));
    for entry in walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name() == "cbindgen.toml")
    {
        let path = entry.path();

        let parent = path.parent().unwrap();
        let out_name = Path::new(
            &parent
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('_', "/"),
        )
        .with_extension("h");

        Builder::new()
            .with_src(parent.join("mod.rs"))
            .with_config(Config::from_file(path).unwrap())
            .generate()
            .unwrap()
            .write_to_file(Path::new("headers").join(out_name));
    }
}
