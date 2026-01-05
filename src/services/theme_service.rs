use crate::models::hex_color::HexColor;
use crate::models::theme::Theme;
use crate::utils::paths::Paths;
use rand::prelude::IndexedRandom;
use regex::Regex;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Deserialize)]
struct RawThemeMetadata {
    name: String,
    description: String,
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
    /// - Writing to the Hypr configuration fails.
    /// - Reloading Waybar fails (symlink creation, SASS compilation, or process restart).
    /// - Setting the wallpaper fails after multiple retry attempts.
    pub fn set_current_theme(theme_directory_path: &PathBuf) -> Result<(), String> {
        Self::compile_theme(theme_directory_path)?;
        Self::reload_waybar()?;
        Self::change_wallpaper()?;

        Ok(())
    }

    /// Get all available themes by reading the directory returned by [`Paths::get_config_path()`].
    ///
    /// # Errors
    ///
    /// The theme directory cannot be found.
    ///
    pub fn get_available_themes() -> Result<Vec<Theme>, String> {
        let config_path = &Paths::get_config_path()?;

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
                ))
            })
            .collect();

        themes.sort_by(|t1, t2| t1.name.cmp(&t2.name));

        Ok(themes)
    }

    fn compile_theme(theme_directory_path: &PathBuf) -> Result<(), String> {
        let config_path = Paths::get_config_path()?;
        let theme_file_path = &theme_directory_path.join("theme-variables.scss");
        let current_theme_dir_path = &config_path.join("current");

        let variables = &Self::collect_variables(theme_file_path)?;

        if let Err(e) = Self::write_hypr_config(variables, theme_file_path) {
            return Err(format!("Failed to write Hypr config: {e}"));
        }

        if let Err(e) = Self::write_kitty_config(variables, theme_file_path) {
            return Err(format!("Failed to write kitty config: {e}"));
        }

        match fs::exists(current_theme_dir_path) {
            Ok(does_exist) => {
                if does_exist {
                    // The current theme dir is a symbolic link
                    fs::remove_file(current_theme_dir_path)
                        .map_err(|e| format!("Failed to remove current theme dir: {e}"))?;
                }
            }
            Err(e) => {
                return Err(format!(
                    "Could not check existence of current theme dir: {e}"
                ));
            }
        }

        Command::new("ln")
            .arg("-s")
            .arg(theme_directory_path)
            .arg(current_theme_dir_path)
            .output()
            .map_err(|e| format!("Failed to create symlink: {e}"))?;

        Ok(())
    }

    fn reload_waybar() -> Result<(), String> {
        let home_path = Paths::get_home_path()?;
        let config_path = Paths::get_config_path()?;
        let theme_waybar_style_path = config_path.join("waybar-style.scss");
        let actual_waybar_style_path = home_path.join(".config/waybar/style.css");

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

    fn write_hypr_config(
        variables: &[(String, String)],
        theme_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let home_path = Paths::get_home_path()?;
        let config_path = home_path.join(".config/hypr/style-variables.conf");

        let mut output = String::new();
        writeln!(output, "# Autogenerated from {}", theme_dir.display())?;

        for (name, value) in variables {
            writeln!(output, "${name} = {value}")?;
        }

        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, output)?;

        Ok(())
    }

    fn write_kitty_config(
        variables: &[(String, String)],
        theme_dir: &Path,
    ) -> Result<(), ThemeError> {
        let home_path = Paths::get_home_path()?;
        let theme_file_path = &home_path.join(".config/kitty/theme.conf");
        let theme_template_file_path = &home_path.join(".config/kitty/theme-template.conf");
        let content = fs::read_to_string(theme_template_file_path)?;
        let replacement_variable_regex = Regex::new(r"__(:?.*)__").unwrap();
        let mut output = String::new();

        writeln!(output, "# Autogenerated from {}", theme_dir.display())?;

        for line in content.lines() {
            let Some(captures) = replacement_variable_regex.captures(line) else {
                writeln!(output, "{line}")?;
                continue;
            };

            if captures.len() != 2 {
                writeln!(output, "{line}")?;
                continue;
            }

            let replacement_variable = captures[0].to_string();
            let variable_name = captures[1].to_string();

            let Some(variable) = variables.iter().find(|v| v.0 == variable_name) else {
                writeln!(output, "{line}")?;
                continue;
            };

            let variable_value = &variable.1;

            let hex_color: HexColor = variable_value.try_into()?;
            let hex_string: String = hex_color.into();
            let new_line = line.replace(&replacement_variable, &hex_string);

            writeln!(output, "{new_line}")?;
        }

        fs::write(theme_file_path, output)?;

        Command::new("kitty")
            .arg("@")
            .arg("--no-response")
            .arg("load-config")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(())
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
        let config_path = Paths::get_config_path()?;
        let wallpaper_dir_path = config_path.join("current/wallpapers");
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

#[derive(Debug)]
enum ThemeError {
    Unknown(String),
    Io(Error),
}

impl Display for ThemeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::Unknown(message) => write!(f, "{message}"),
            ThemeError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ThemeError {}

impl From<Error> for ThemeError {
    fn from(value: Error) -> Self {
        ThemeError::Io(value)
    }
}

impl From<std::fmt::Error> for ThemeError {
    fn from(value: std::fmt::Error) -> Self {
        ThemeError::Unknown(format!("{value}"))
    }
}

impl From<String> for ThemeError {
    fn from(value: String) -> Self {
        ThemeError::Unknown(value)
    }
}
