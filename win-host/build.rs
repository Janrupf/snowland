#[cfg(windows)]
use winres::WindowsResource;

fn main() {
    #[cfg(not(windows))]
    compile_error!("Can't compile the win-host for non Win32!");

    #[cfg(windows)]
    {
        setup_resources();
    }
}

#[cfg(windows)]
fn setup_resources() {
    WindowsResource::new()
        .set_icon("res/snowflake.ico")
        .set_manifest_file("res/snowland-host-win.manifest")
        .compile()
        .expect("Failed to compile resource file!");
}
