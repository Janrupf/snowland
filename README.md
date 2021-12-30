# Snowland Desktop Renderer

Snowland is a custom desktop background renderer, similar to Wallpaper Engine - but way
more primitive. The project developed as a fun idea and interest in drawing on the 
Windows wallpaper without overriding the icons. At the moment Snowland implements a very
basic rendering module system and a GUI written using 
[ImGui-rs](https://github.com/imgui-rs/imgui-rs).

See the chapters below for instructions on building and usage.

## Supported platforms

- Windows
- Linux

## Building Snowland
Snowland is written in Rust and thus requires the Rust toolchain in order to build. It is
recommended to use [Rustup](https://rustup.rs/) for managing the Rust toolchain. The 
required toolchain version can be found in the [rust-toolchain](./rust-toolchain) file,
cargo should automatically download the correct toolchain though when first running it
inside the project.

After Rust has been installed building and running Snowland is straight forward and not
different from any other Rust program:

```powershell
cargo build # Create a debug build in target/debug
cargo run   # Build and run snowland
```

To create a release build do the following:
````powershell
cargo build --release
````

The resulting binary can be found in `target/release`, look for a file called 
`snowland-system-host`, where `system` is the name of the operating system you are on.

## Project structure

The Snowland project structure is split into multiple parts:
- [universal](./universal) - The snowland core implementation, OS independent
- [win-host](./win-host) - Windows specific implementation, responsible for bootstrapping
  on Windows

The `*-host` modules contain the main function and are executables, whereas the 
`universal` module is a library which then is linked into the `*-host` modules.

## Inner Workings

### universal

The `universal` library provides the core drawing routines and hosts the user interface
as well as configuration management. It uses [Skia](https://skia.org/) with the help of
[skia-safe](https://github.com/rust-skia/rust-skia) to perform the actual drawing without
knowing about the rendering library.

### win-host

The `win-host` executable provides the entry point and shell integration on Windows. The
shell integration consists of a simple system tray icon and the code responsible for 
acquiring a drawing context on the desktop background.

The Windows shell (specifically `explorer.exe`) provides an undocumented window message 
for separating the desktop icons and desktop background into separate windows. Snowland
uses this message to first perform this split and then acquires a window handle to the
freshly created background window. With the help of WGL (the Windows OpenGL
implementation) an OpenGL context is created and then passed to a new Skia context. From
here on the `universal` library takes over the Skia context and performs the common 
drawing routines.

See [here](https://www.codeproject.com/Articles/856020/Draw-Behind-Desktop-Icons-in-Windows-plus)
([WebArchive version](https://web.archive.org/web/20211001000000*/https://www.codeproject.com/Articles/856020/Draw-Behind-Desktop-Icons-in-Windows-plus))
for more details on drawing behind the desktop icons.
