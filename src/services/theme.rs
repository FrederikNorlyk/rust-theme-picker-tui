use rand::prelude::IndexedRandom;
use regex::Regex;
use std::fmt::Write;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, thread};

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
pub fn set_theme(theme_dir_name: &str) -> Result<(), String> {
    compile_theme(theme_dir_name)?;
    reload_waybar()?;
    change_wallpaper()?;

    Ok(())
}

fn get_home_path() -> Result<PathBuf, String> {
    let Ok(home) = env::var("HOME") else {
        return Err("Could not get home dir".to_string());
    };

    Ok(PathBuf::from(home))
}

fn get_root_path() -> Result<PathBuf, String> {
    let home_path = get_home_path()?;
    Ok(home_path.join(".local/share/norlyk-themes"))
}

fn compile_theme(theme_dir_name: &str) -> Result<(), String> {
    let root_path = &get_root_path()?;
    let theme_dir_path = root_path.join(theme_dir_name);
    let theme_file_path = theme_dir_path.join("theme-variables.scss");
    let selected_theme_dir_path = &root_path.join(theme_dir_name);
    let current_theme_dir_path = &root_path.join("current");

    let variables = collect_variables(&theme_file_path)?;

    if let Err(e) = write_hypr_config(&variables, &theme_file_path) {
        return Err(format!("Failed to write Hypr config: {e}"));
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
        .arg(selected_theme_dir_path)
        .arg(current_theme_dir_path)
        .output()
        .map_err(|e| format!("Failed to create symlink: {e}"))?;

    Ok(())
}

fn reload_waybar() -> Result<(), String> {
    let home_path = get_home_path()?;
    let root_path = get_root_path()?;
    let theme_waybar_style_path = root_path.join("waybar-style.scss");
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

            variables.extend(collect_variables(&absolute_used_path)?);

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
    let home_path = get_home_path()?;
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
    let root_path = get_root_path()?;
    let wallpaper_dir_path = root_path.join("current/wallpapers");
    let wallpaper_path = get_random_file(&wallpaper_dir_path)?;

    let max_attempts = 5;
    let mut error: Option<Error> = None;

    for _ in 1..=max_attempts {
        let output = Command::new("hyprctl")
            .arg("hyprpaper")
            .arg("reload")
            .arg(format!(",{}", wallpaper_path.display()))
            .output();

        match output {
            Ok(result) => {
                let response = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if response == "ok" {
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

fn get_random_file(path: &Path) -> Result<PathBuf, String> {
    // Read all image files from the theme directory
    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read theme directory: {e}"))?;

    let image_files: Vec<PathBuf> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            // Check if it's a file with common image extensions
            if path.is_file()
                && let Some(ext) = path.extension()
            {
                let ext_str = ext.to_string_lossy().to_lowercase();
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
