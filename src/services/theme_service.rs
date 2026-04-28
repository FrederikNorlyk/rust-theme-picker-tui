use crate::models::theme::{ColorScheme, Theme};
use crate::services::themers::btop::BtopThemer;
use crate::services::themers::gtk::GtkThemer;
use crate::services::themers::hypr::HyprThemer;
use crate::services::themers::kitty::KittyThemer;
use crate::services::themers::nvim::NvimThemer;
use crate::services::themers::waybar::WaybarThemer;
use crate::services::themers::{ThemeContext, Themer};
use crate::utils::paths::Paths;
use crate::utils::symlink::Symlink;
use rand::prelude::IndexedRandom;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct RawThemeMetadata {
    name: String,
    description: String,
    btop_theme_path: Option<String>,
    color_scheme: ColorScheme,
    gtk_theme: String,
}

pub struct ThemeService;

impl ThemeService {
    /// Sets the theme by configuring Hypr, Waybar, and wallpaper settings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `HOME` environment variable is not set or inaccessible.
    /// - The theme directory or theme variables file cannot be found.
    /// - The SCSS variables cannot be parsed from the theme file.
    /// - Application of a theme to a program failed
    /// - Setting the wallpaper fails after multiple retry attempts.
    pub fn set_current_theme(theme: &Theme) -> Result<(), String> {
        let context = Self::create_context(theme)
            .map_err(|e| format!("Could not create theme context: {e}"))?;

        Symlink::create(&theme.directory_path, &Paths::current_theme()?)?;

        for themer in Self::themers() {
            themer
                .apply(&context)
                .map_err(|e| format!("Could not apply theme: {e}"))?;
        }

        Self::change_wallpaper()?;

        Ok(())
    }

    fn create_context(theme: &Theme) -> Result<ThemeContext<'_>, String> {
        let path = &theme.get_theme_variables_css_file_path();
        let variables = Self::collect_variables(path)?;

        Ok(ThemeContext { theme, variables })
    }

    fn themers() -> Vec<Box<dyn Themer>> {
        vec![
            Box::new(HyprThemer),
            Box::new(KittyThemer),
            Box::new(WaybarThemer),
            Box::new(BtopThemer),
            Box::new(GtkThemer),
            Box::new(NvimThemer),
        ]
    }

    /// Get all available themes by reading the directory returned by [`Paths::config_path()`].
    ///
    /// # Errors
    ///
    /// The theme directory cannot be found.
    ///
    pub fn get_available_themes() -> Result<Vec<Theme>, String> {
        let config_path = &Paths::config_path()?;

        let files = fs::read_dir(config_path)
            .map_err(|e| format!("Failed to read files in the config directory: {e}"))?;

        let mut themes: Vec<Theme> = files
            .filter_map(|file| {
                let entry = file.ok()?;
                let path = entry.path();

                if !path.is_dir() || path.is_symlink() {
                    return None;
                }

                let meta_file_path = path.join("meta.toml");
                let contents = fs::read_to_string(meta_file_path).ok()?;
                let meta: RawThemeMetadata = toml::from_str(&contents).ok()?;

                Some(Theme::new(
                    meta.name.as_str(),
                    meta.description.as_str(),
                    path,
                    meta.btop_theme_path.map(PathBuf::from),
                    meta.color_scheme,
                    meta.gtk_theme.as_str(),
                ))
            })
            .collect();

        themes.sort_by(|t1, t2| t1.name.cmp(&t2.name));

        Ok(themes)
    }

    fn collect_variables(path: &Path) -> Result<Vec<(String, String)>, String> {
        let Ok(content) = fs::read_to_string(path) else {
            return Err(format!("Could not read file: {}", path.display()));
        };

        let mut variables: Vec<(String, String)> = Vec::new();
        let use_import_regex = Regex::new(r#"^@use "(:?.*)";$"#).unwrap();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("@use") {
                let Some(relative_used_path) = use_import_regex
                    .captures(line)
                    .and_then(|caps| caps.get(1))
                    .map(|m| format!("{}.scss", m.as_str()))
                else {
                    continue;
                };

                let parent = Path::new(path).parent().unwrap();
                let absolute_used_path = parent.join(relative_used_path);

                variables.extend(Self::collect_variables(&absolute_used_path)?);

                continue;
            }

            // Match pattern: $variableName: value;
            let Some(dollar_pos) = trimmed.find('$') else {
                continue;
            };

            let Some(colon_pos) = trimmed.find(':') else {
                continue;
            };

            let Some(semicolon_pos) = trimmed.find(';') else {
                continue;
            };

            let var_name = trimmed[dollar_pos + 1..colon_pos].trim().to_string();

            let var_value = trimmed[colon_pos + 1..semicolon_pos]
                .trim()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>();

            variables.push((var_name, var_value));
        }

        if variables.is_empty() {
            return Err("No SCSS variables found".to_string());
        }

        Ok(variables)
    }

    /// Reloads the wallpaper by selecting a random image from the current theme's wallpaper directory
    /// and setting it using hyprpaper.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `HOME` environment variable is not set or inaccessible.
    /// - The wallpaper directory cannot be read or contains no valid image files.
    /// - The hyprctl command fails to execute or returns an error after multiple retry attempts.
    pub fn change_wallpaper() -> Result<(), String> {
        let wallpaper_dir_path = Paths::current_theme()?.join("wallpapers");
        let wallpaper_file_path = Self::get_random_image_file(&wallpaper_dir_path)?;

        let max_attempts = 5;
        let mut error: Option<Error> = None;

        let wallpaper_arg = format!(",{}", wallpaper_file_path.display());

        for _ in 1..=max_attempts {
            let output = Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(&wallpaper_arg)
                .output();

            match output {
                Ok(result) => {
                    let response = String::from_utf8_lossy(&result.stdout).trim().to_string();
                    if response.is_empty() {
                        return Ok(());
                    }

                    error = Some(Error::other(response));
                }
                Err(e) => error = Some(e),
            }

            thread::sleep(Duration::from_secs(1));
        }

        if let Some(error) = error {
            return Err(format!("Failed to set wallpaper: {error}"));
        }

        Err("Unknown error".to_string())
    }

    fn get_random_image_file(path: &Path) -> Result<PathBuf, String> {
        // Read all image files from the theme directory
        let entries =
            fs::read_dir(path).map_err(|e| format!("Failed to read theme directory: {e}"))?;

        let image_files: Vec<PathBuf> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                // Check if it's a file with common image extensions
                if path.is_file()
                    && let Some(extension) = path.extension()
                {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "bmp") {
                        return Some(path);
                    }
                }
                None
            })
            .collect();

        if image_files.is_empty() {
            return Err("No image files found in theme directory".to_string());
        }

        // Select a random image
        let mut rng = rand::rng();

        image_files
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| "Failed to select random image".to_string())
    }
}
