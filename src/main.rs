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
use theme_picker::models::theme::Theme;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

struct App {
    should_exit: bool,
    theme_list: ThemeList,
}

struct ThemeList {
    themes: Vec<Theme>,
    state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            theme_list: ThemeList::from_iter([
                // TODO: Scan the .local/share/norlyk-themes directory (add metadata file to each theme with descriptions, etc.)
                (
                    "Kanagawa",
                    "kanagawa",
                    "Dark colorscheme inspired by the colors of the famous painting by Katsushika Hokusai.",
                ),
                ("Nord", "nord", "An arctic, north-bluish color palette."),
                (
                    "Gruvbox",
                    "gruvbox",
                    "A warm, retro color scheme with earthy tones designed for comfortable, long-term viewing.",
                ),
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

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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

        if let Err(e) = theme_picker::services::theme::set_theme(&selected_theme.dir_name) {
            eprintln!("Failed to set the theme: {}\n{}", selected_theme.name, e);
        }
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
