mod util;

use ratatui::style::Color;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};
use std::io;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    should_exit: bool,
    theme_list: ThemeList,
}

struct ThemeList {
    themes: Vec<Theme>,
    state: ListState,
}

#[derive(Debug)]
struct Theme {
    name: String,
    dir_name: String,
    info: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            theme_list: ThemeList::from_iter([
                (
                    "Kanagawa",
                    "kanagawa",
                    "Dark colorscheme inspired by the colors of the famous painting by Katsushika Hokusai.",
                ),
                ("Nord", "nord", "An arctic, north-bluish color palette."),
            ]),
        }
    }
}

impl FromIterator<(&'static str, &'static str, &'static str)> for ThemeList {
    fn from_iter<I: IntoIterator<Item = (&'static str, &'static str, &'static str)>>(
        iter: I,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(name, dir_name, info)| Theme::new(name, dir_name, info))
            .collect();

        Self {
            themes: items,
            state: ListState::default(),
        }
    }
}

impl Theme {
    fn new(name: &str, dir_name: &str, info: &str) -> Self {
        Self {
            name: name.to_string(),
            dir_name: dir_name.to_string(),
            info: info.to_string(),
        }
    }
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => self.toggle_theme(),
            _ => {}
        }
    }

    fn select_next(&mut self) {
        self.theme_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.theme_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.theme_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.theme_list.state.select_last();
    }

    fn toggle_theme(&mut self) {
        let Some(selected_theme) = self.get_selected_theme() else {
            return;
        };

        util::theme::set_theme(selected_theme);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Theme Picker ");

        let instructions = Line::from(vec![
            " Use ".into(),
            "g/G".blue().bold(),
            " to go top/bottom, ".into(),
            "enter".blue().bold(),
            " to select, ".into(),
            "q ".blue().bold(),
            " to quit".into(),
        ]);

        let block = Block::new()
            .borders(Borders::ALL)
            .title(title.centered())
            .title_bottom(instructions.centered());
        let inner = block.inner(area);

        let [list_area, info_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(5)]).areas(inner);

        self.render_list(list_area, buf);
        self.render_info(info_area, buf);
        block.render(area, buf);
    }
}

impl App {
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.theme_list.themes.iter().map(ListItem::from).collect();

        let list = List::new(items)
            .highlight_style(Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.theme_list.state);
    }

    fn get_selected_theme(&self) -> Option<&Theme> {
        let index = self.theme_list.state.selected()?;

        Some(&self.theme_list.themes[index])
    }

    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        let Some(selected_theme) = self.get_selected_theme() else {
            return;
        };

        let info = &selected_theme.info;
        let block = Block::new().borders(Borders::ALL);

        Paragraph::new(info.as_str())
            .wrap(Wrap { trim: false })
            .block(block)
            .render(area, buf);
    }
}

impl From<&Theme> for ListItem<'_> {
    fn from(value: &Theme) -> Self {
        ListItem::new(Line::from(value.name.clone()))
    }
}
