use std::fmt::Formatter;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use skia_safe::{paint::Style, scalar, Color4f, Paint};

use crate::scene::module::ModuleConfig;

#[derive(Debug, Clone)]
pub struct ColorSetting(Color4f);

impl ColorSetting {
    pub fn get_color(&self) -> Color4f {
        self.0
    }
}

impl Default for ColorSetting {
    fn default() -> Self {
        let color = Color4f::new(1.0, 1.0, 1.0, 1.0);
        Self(color)
    }
}

impl ModuleConfig for ColorSetting {}

impl<'de> Deserialize<'de> for ColorSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorSettingVisitor;

        impl<'de> Visitor<'de> for ColorSettingVisitor {
            type Value = ColorSetting;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct ColorSetting")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let r = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let g = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let b = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let a = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;

                Ok(ColorSetting(Color4f::new(r, g, b, a)))
            }
        }

        deserializer.deserialize_seq(ColorSettingVisitor)
    }
}

impl Serialize for ColorSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let [r, g, b, a] = self.0.as_array();

        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(r)?;
        seq.serialize_element(g)?;
        seq.serialize_element(b)?;
        seq.serialize_element(a)?;
        seq.end()
    }
}

impl From<ColorSetting> for Color4f {
    fn from(setting: ColorSetting) -> Self {
        setting.0
    }
}

#[derive(Debug, Clone)]
pub struct PaintSetting(Paint);

impl PaintSetting {
    pub fn get_paint(&self) -> &Paint {
        &self.0
    }
}

impl Default for PaintSetting {
    fn default() -> Self {
        let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        Self(paint)
    }
}

impl ModuleConfig for PaintSetting {}

#[derive(Serialize, Deserialize)]
struct StrokeSetting {
    pub width: scalar,
    pub miter: scalar,
}

impl<'de> Deserialize<'de> for PaintSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PaintSettingVisitor;

        impl<'de> Visitor<'de> for PaintSettingVisitor {
            type Value = PaintSetting;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct PaintSetting")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    Color,
                    AntiAlias,
                    Dither,
                    Stroke,
                }

                let mut color = None;
                let mut anti_alias = None;
                let mut dither = None;
                let mut stroke = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Color => {
                            if color.is_some() {
                                return Err(serde::de::Error::duplicate_field("color"));
                            }

                            let setting: ColorSetting = map.next_value()?;
                            color = Some(setting.get_color());
                        }
                        Field::AntiAlias => {
                            if anti_alias.is_some() {
                                return Err(serde::de::Error::duplicate_field("anti_alias"));
                            }

                            anti_alias = Some(map.next_value()?);
                        }
                        Field::Dither => {
                            if dither.is_some() {
                                return Err(serde::de::Error::duplicate_field("dither"));
                            }

                            dither = Some(map.next_value()?);
                        }
                        Field::Stroke => {
                            if stroke.is_some() {
                                return Err(serde::de::Error::duplicate_field("stroke"));
                            }

                            stroke = map.next_value::<Option<StrokeSetting>>()?;
                        }
                    }
                }

                let color = color.ok_or_else(|| serde::de::Error::missing_field("color"))?;
                let anti_alias =
                    anti_alias.ok_or_else(|| serde::de::Error::missing_field("anti_alias"))?;
                let dither = dither.ok_or_else(|| serde::de::Error::missing_field("dither"))?;

                let mut paint = Paint::new(color, None);
                paint.set_anti_alias(anti_alias);
                paint.set_dither(dither);

                if let Some(StrokeSetting { width, miter }) = stroke {
                    paint.set_style(Style::Stroke);
                    paint.set_stroke_width(width);
                    paint.set_stroke_miter(miter);
                } else {
                    paint.set_style(Style::Fill);
                }

                Ok(PaintSetting(paint))
            }
        }

        deserializer.deserialize_struct(
            "PaintSetting",
            &["color", "anti_alias", "dither", "stroke"],
            PaintSettingVisitor,
        )
    }
}

impl Serialize for PaintSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let paint = &self.0;

        let mut serializer = serializer.serialize_struct("PaintSetting", 4)?;
        serializer.serialize_field("color", &ColorSetting(paint.color4f()))?;
        serializer.serialize_field("anti_alias", &paint.is_anti_alias())?;
        serializer.serialize_field("dither", &paint.is_dither())?;

        if paint.style() == Style::Stroke {
            serializer.serialize_field(
                "stroke",
                &Some(StrokeSetting {
                    width: paint.stroke_width(),
                    miter: paint.stroke_miter(),
                }),
            )?;
        } else {
            serializer.serialize_field("stroke", &(None as Option<StrokeSetting>))?;
        }

        serializer.end()
    }
}

impl From<PaintSetting> for Paint {
    fn from(setting: PaintSetting) -> Self {
        setting.0
    }
}
