use std::env;
use std::path::PathBuf;

pub struct Paths;

impl Paths {
    /// Gets the path to the users home directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable `HOME` is not set.
    ///
    pub fn get_home_path() -> Result<PathBuf, String> {
        let Ok(home) = env::var("HOME") else {
            return Err("Could not get home dir".to_string());
        };

        Ok(PathBuf::from(home))
    }

    /// Gets the path to the directory containing the theme picker's configuration files, located at
    /// `~/.local/share/norlyk-themes/`.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable `HOME` is not set.
    ///
    pub fn get_config_path() -> Result<PathBuf, String> {
        let home_path = Self::get_home_path()?;
        Ok(home_path.join(".local/share/norlyk-themes"))
    }
}
