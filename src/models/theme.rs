use ratatui::prelude::Line;
use ratatui::widgets::ListItem;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub directory_path: PathBuf,
}

impl Theme {
    #[must_use]
    pub fn new(name: &str, description: &str, directory_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            directory_path,
        }
    }
}

impl From<&Theme> for ListItem<'_> {
    fn from(value: &Theme) -> Self {
        ListItem::new(Line::from(value.name.clone()))
    }
}
