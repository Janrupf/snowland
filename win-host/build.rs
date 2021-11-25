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

fn setup_resources() {
    WindowsResource::new()
        .set_icon("res/snowflake.ico")
        .compile()
        .expect("Faled to compile resource file!");
}
