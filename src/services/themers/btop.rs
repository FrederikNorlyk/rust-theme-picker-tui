use crate::services::themers::{ThemeContext, Themer};
use crate::utils::paths::Paths;
use regex::Regex;
use std::fs;

pub struct BtopThemer;

impl Themer for BtopThemer {
    fn apply(&self, context: &ThemeContext<'_>) -> Result<(), String> {
        let color_theme = match context.theme.btop_theme_path.as_deref() {
            Some(path) => path.display().to_string(),
            None => String::from("Default"),
        };

        let btop_conf_path = Paths::user_home()?.join(".config/btop/btop.conf");
        let theme_line = format!(r#"color_theme = "{color_theme}""#);

        if btop_conf_path.exists() {
            let Ok(btop_conf) = fs::read_to_string(&btop_conf_path) else {
                return Err(format!("Could not read: {}", btop_conf_path.display()));
            };

            let color_theme_regex =
                Regex::new(r#"(?m)^color_theme = ".*"$"#).map_err(|e| e.to_string())?;

            let updated_content = if color_theme_regex.is_match(&btop_conf) {
                color_theme_regex
                    .replace_all(&btop_conf, theme_line.as_str())
                    .to_string()
            } else {
                let mut new_content = btop_conf;
                new_content.push('\n');
                new_content.push_str(&theme_line);
                new_content.push('\n');
                new_content
            };

            fs::write(&btop_conf_path, updated_content)
                .map_err(|e| format!("Could not write: {} ({e})", btop_conf_path.display()))?;
        } else {
            if let Some(parent) = btop_conf_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Could not create config directory: {e}"))?;
            }

            fs::write(&btop_conf_path, format!("{theme_line}\n"))
                .map_err(|e| format!("Could not write: {} ({e})", btop_conf_path.display()))?;
        }

        Ok(())
    }
}
