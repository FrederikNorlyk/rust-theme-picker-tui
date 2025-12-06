use crate::models::rgba_color::RGBAColor;
use easy_color::Hex;

/// Wrapper for `easy_color`'s `Hex`, so that traits can be implemented
pub struct HexColor(pub Hex);

impl TryFrom<&String> for HexColor {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let rgba_color: RGBAColor = value.try_into()?;
        let hex_value: Hex = rgba_color.0.into();

        Ok(HexColor(hex_value))
    }
}

impl From<HexColor> for String {
    fn from(value: HexColor) -> Self {
        value.0.to_string()
    }
}
