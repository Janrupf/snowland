use nativeshell_build::*;
#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    let options = FlutterOptions {
        ..Default::default()
    };

    Flutter::build(options).expect("Failed to build flutter");

    #[cfg(windows)]
    setup_resources();

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib");
}

#[cfg(windows)]
fn setup_resources() {
    WindowsResource::new()
        .set_icon("../../res/snowflake.ico")
        .set_manifest_file("res/snowland-control-panel-win.manifest")
        .compile()
        .expect("Failed to compile resource file!");
}
