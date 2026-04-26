use crate::services::themers::{ThemeContext, Themer};
use crate::utils::paths::Paths;
use std::process::{Command, Stdio};

pub struct WaybarThemer;

impl Themer for WaybarThemer {
    fn apply(&self, _context: &ThemeContext<'_>) -> Result<(), String> {
        let config_path = Paths::config_path()?;
        let theme_waybar_style_path = config_path.join("waybar-style.scss");
        let actual_waybar_style_path = Paths::user_home()?.join(".config/waybar/style.css");

        Command::new("sass")
            .arg("--no-source-map")
            .arg(theme_waybar_style_path)
            .arg(actual_waybar_style_path)
            .output()
            .map_err(|e| format!("Failed to compile .css file: {e}"))?;

        Command::new("pkill")
            .arg("waybar")
            .output()
            .map_err(|e| format!("Failed to stop waybar: {e}"))?;

        Command::new("nohup")
            .arg("waybar")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start waybar: {e}"))?;

        Ok(())
    }
}
