use ratatui::prelude::Line;
use ratatui::widgets::ListItem;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub dir_name: String,
    pub info: String,
}

impl Theme {
    #[must_use]
    pub fn new(name: &str, dir_name: &str, info: &str) -> Self {
        Self {
            name: name.to_string(),
            dir_name: dir_name.to_string(),
            info: info.to_string(),
        }
    }
}

impl From<&Theme> for ListItem<'_> {
    fn from(value: &Theme) -> Self {
        ListItem::new(Line::from(value.name.clone()))
    }
}
