use skia_safe::{Data, Typeface};

const NOTO_SANS_MONO: &[u8] = include_bytes!("NotoSansMono.ttf");

/// All embedded fonts.
pub enum Font {
    /// The NotoSansMono variable font
    NotoSansMono,
}

/// Loads an embedded font.
pub fn load_embedded_font(font: Font) -> skia_safe::Typeface {
    let data = unsafe {
        match font {
            Font::NotoSansMono => Data::new_bytes(NOTO_SANS_MONO),
        }
    };

    Typeface::from_data(data, None).unwrap()
}
