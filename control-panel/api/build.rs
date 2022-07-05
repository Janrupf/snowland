use std::path::PathBuf;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let bindings = cbindgen::generate(crate_dir).expect("Failed to generate C bindings");

    let out_file = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("snowland_api.h");
    bindings.write_to_file(&out_file);

    #[cfg(debug_assertions)]
    println!(
        "cargo:warning=API headers written to {}",
        out_file.display()
    );
}
