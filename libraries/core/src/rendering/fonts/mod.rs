use skia_safe::{Data, Typeface};

const NOTO_SANS_MONO: &[u8] = include_bytes!("NotoSansMono.ttf");
const ROBOTO_REGULAR: &[u8] = include_bytes!("RobotoRegular.ttf");

/// All embedded fonts.
pub enum Font {
    /// The NotoSansMono variable font
    NotoSansMono,

    /// The Roboto font regular variant
    RobotoRegular,
}

/// Loads an embedded font as a Skia typeface.
pub fn load_embedded_font(font: Font) -> skia_safe::Typeface {
    let data = unsafe { Data::new_bytes(get_embedded_font_bytes(font)) };
    Typeface::from_data(data, None).unwrap()
}

/// Retrieves the bytes for an embedded font
pub fn get_embedded_font_bytes(font: Font) -> &'static [u8] {
    match font {
        Font::NotoSansMono => NOTO_SANS_MONO,
        Font::RobotoRegular => ROBOTO_REGULAR,
    }
}
