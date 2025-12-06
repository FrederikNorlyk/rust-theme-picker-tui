use easy_color::RGBA;

/// Wrapper for `easy_color`'s `RGBA`, so that traits can be implemented
pub struct RGBAColor(pub RGBA);

impl TryFrom<&String> for RGBAColor {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let rgba_tuple: (u8, u8, u8, f32) = {
            let parts: Vec<u8> = value
                .replace("rgba(", "")
                .replace(')', "")
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();

            // We don't care about the 4th part (the alpha value). In Kitty, we force this to be '1'
            if parts.len() < 3 {
                return Err(format!("Invalid string: {value}"));
            }

            (parts[0], parts[1], parts[2], 1f32)
        };

        let rgba_value: RGBA = rgba_tuple.try_into().unwrap();

        Ok(RGBAColor(rgba_value))
    }
}
