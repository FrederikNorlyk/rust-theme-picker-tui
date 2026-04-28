pub mod btop;
pub mod gtk;
pub mod hypr;
pub mod kitty;
pub mod nvim;
pub mod waybar;

use crate::models::theme::Theme;

pub trait Themer {
    /// Applies a theme to a specific target application or service.
    ///
    /// Implementations may update configuration files, create directories,
    /// execute external commands, reload applications, or transform theme
    /// variables into the format required by the target service.
    ///
    /// # Errors
    ///
    /// Returns an error if applying the theme fails. This can happen when:
    /// - Required configuration paths cannot be resolved.
    /// - Required files cannot be read or written.
    /// - Theme variables are missing or invalid.
    /// - An external command cannot be executed or reports a failure.
    fn apply(&self, context: &ThemeContext<'_>) -> Result<(), String>;
}

pub struct ThemeContext<'a> {
    pub theme: &'a Theme,
    pub variables: Vec<(String, String)>,
}
