use std::path::PathBuf;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let bindings = cbindgen::generate(crate_dir).expect("Failed to generate C bindings");

    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings.write_to_file(path.join("snowland_api.h"));
}
