use ratatui::prelude::Line;
use ratatui::widgets::ListItem;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub directory_path: PathBuf,
    pub btop_theme_path: Option<PathBuf>,
    pub color_scheme: ColorScheme,
    pub gtk_theme: String,
}

impl Theme {
    #[must_use]
    pub fn new(
        name: &str,
        description: &str,
        directory_path: PathBuf,
        btop_theme_path: Option<PathBuf>,
        color_scheme: ColorScheme,
        gtk_theme: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            directory_path,
            btop_theme_path,
            color_scheme,
            gtk_theme: gtk_theme.to_string(),
        }
    }
}

impl From<&Theme> for ListItem<'_> {
    fn from(value: &Theme) -> Self {
        ListItem::new(Line::from(value.name.clone()))
    }
}
