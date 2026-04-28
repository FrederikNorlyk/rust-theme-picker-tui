use crate::models::theme::ColorScheme;
use crate::services::themers::{ThemeContext, Themer};
use std::process::Command;

pub struct GtkThemer;

impl Themer for GtkThemer {
    fn apply(&self, context: &ThemeContext<'_>) -> Result<(), String> {
        let color_scheme: &str = match context.theme.color_scheme {
            ColorScheme::Light => "prefer-light",
            ColorScheme::Dark => "prefer-dark",
        };

        Command::new("gsettings")
            .arg("set")
            .arg("org.gnome.desktop.interface")
            .arg("color-scheme")
            .arg(color_scheme)
            .output()
            .map_err(|e| format!("Failed to set GTK color scheme: {e}"))?;

        Command::new("gsettings")
            .arg("set")
            .arg("org.gnome.desktop.interface")
            .arg("gtk-theme")
            .arg(&context.theme.gtk_theme)
            .output()
            .map_err(|e| format!("Failed to set GTK theme: {e}"))?;

        Ok(())
    }
}
