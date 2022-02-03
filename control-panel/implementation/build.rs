use nativeshell_build::*;

fn main() {
    let options = FlutterOptions {
        ..Default::default()
    };

    Flutter::build(options).expect("Failed to build flutter");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib");
}
