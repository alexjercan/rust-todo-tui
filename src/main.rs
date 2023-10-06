mod args;
mod config;

use anyhow::{bail, Error, Result};
use chrono::Days;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    fs::{self, OpenOptions},
    io::{stdout, Read},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Default)]
pub struct Item {
    text: String,
    completed: bool,
}

impl ToString for Item {
    fn to_string(&self) -> String {
        let status = if self.completed { "- [x]" } else { "- [ ]" };
        return format!("{} {}", status, self.text);
    }
}

impl FromStr for Item {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("- [ ]") && !s.starts_with("- [x]") {
            bail!("Invalid item format");
        }

        // HACK: This is a hack to parse todo items from a string.
        let text = &s[6..];

        match &s[..5] {
            "- [ ]" => {
                return Ok(Item {
                    text: text.to_string(),
                    completed: false,
                })
            }
            "- [x]" => {
                return Ok(Item {
                    text: text.to_string(),
                    completed: true,
                })
            }
            _ => bail!("Invalid item format"),
        }
    }
}

impl Item {
    pub fn new(text: String) -> Self {
        return Item {
            text,
            completed: false,
        };
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Insert,
}

impl Default for InputMode {
    fn default() -> Self {
        return InputMode::Normal;
    }
}

pub fn write_items<P>(items: &Vec<Item>, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fs::write(
        path,
        items
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;

    return Ok(());
}

pub fn read_items<P>(path: P, default_items: &Vec<String>) -> Result<Vec<Item>>
where
    P: AsRef<Path>,
{
    let mut items = Vec::new();

    if !path.as_ref().exists() {
        items.extend(default_items.iter().map(|i| Item::new(i.to_string())));
    }

    let mut data = String::new();

    let _ = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?
        .read_to_string(&mut data)?;

    items.extend(data.lines().filter_map(|line| line.parse::<Item>().ok()));

    return Ok(items);
}

pub fn date(offset: i64, format: &str) -> String {
    if offset >= 0 {
        chrono::Utc::now().checked_add_days(Days::new(offset.unsigned_abs()))
    } else {
        chrono::Utc::now().checked_sub_days(Days::new(offset.unsigned_abs()))
    }
    .expect("Buy more bits")
    .format(format)
    .to_string()
}

fn main() -> Result<()> {
    let args = args::Args::parse();
    let config = match args.config {
        Some(path) => config::Config::from_file(&path)?,
        None => config::Config::parse()?,
    };

    fs::create_dir_all(&config.path)?;

    match args.subcmd {
        Some(args::SubCommand::Status) => status(config),
        Some(args::SubCommand::Details) => details(config),
        None => tui(config),
    }
}

fn status(config: config::Config) -> Result<()> {
    let day_offset = 0;
    let day_name = date(day_offset, &config.date_format);
    let day_path = Path::new(&config.path).join(format!("{}.md", day_name));

    let items = read_items(&day_path, &config.habits)?;

    let completed = items.iter().filter(|i| i.completed).count();
    let total = items.len();

    println!("{} / {}", completed, total);

    return Ok(());
}

fn details(config: config::Config) -> Result<()> {
    let day_offset = 0;
    let day_name = date(day_offset, &config.date_format);
    let day_path = Path::new(&config.path).join(format!("{}.md", day_name));

    let items = read_items(&day_path, &config.habits)?;

    for item in items {
        println!("{}", item.to_string());
    }

    return Ok(());
}

fn tui(config: config::Config) -> Result<()> {
    let mut input_text = String::default();
    let mut input_mode = InputMode::default();
    let mut day_offset = 0;
    let mut day_name = date(day_offset, &config.date_format);
    let mut day_path = Path::new(&config.path).join(format!("{}.md", day_name));
    let mut items = read_items(&day_path, &config.habits)?;
    let mut items_state = ListState::default();

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(10), Constraint::Max(2), Constraint::Max(1)].as_ref())
                .split(f.size());

            let items = List::new(
                items
                    .iter()
                    .map(|i| -> ListItem { ListItem::new(i.to_string()).style(Style::default()) })
                    .collect::<Vec<_>>(),
            )
            .block(
                Block::default()
                    .title(day_name.clone())
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

            let (msg, style) = match input_mode {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to go to today, "),
                        Span::styled("h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to go yesterday, "),
                        Span::styled("l", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to go tomorrow, "),
                        Span::styled("k", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to move up, "),
                        Span::styled("j", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to move down, "),
                        Span::styled("x", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to toggle, "),
                        Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to add new todo, "),
                        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to remove."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                _ => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to write the todo."),
                    ],
                    Style::default(),
                ),
            };

            let mut text = Text::from(Line::from(msg));
            text.patch_style(style);

            let help_message = Paragraph::new(text).wrap(Wrap { trim: true });

            f.render_stateful_widget(items, chunks[0], &mut items_state);
            f.render_widget(help_message, chunks[1]);

            match input_mode {
                InputMode::Normal => {}
                _ => {
                    let p = Paragraph::new(Span::raw(input_text.as_str()));
                    f.render_widget(p, chunks[2]);
                    f.set_cursor(chunks[2].x + input_text.len() as u16, chunks[2].y);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            write_items(&items, &day_path)?;
                            break;
                        }
                        KeyCode::Char('t') => {
                            write_items(&items, &day_path)?;

                            day_offset = 0;
                            day_name = date(day_offset, &config.date_format);
                            day_path = Path::new(&config.path).join(format!("{}.md", day_name));
                            items = read_items(&day_path, &config.habits)?;
                            items_state = ListState::default();
                        }
                        KeyCode::Char('h') => {
                            write_items(&items, &day_path)?;

                            day_offset -= 1;
                            day_name = date(day_offset, &config.date_format);
                            day_path = Path::new(&config.path).join(format!("{}.md", day_name));
                            items = read_items(&day_path, &config.habits)?;
                            items_state = ListState::default();
                        }
                        KeyCode::Char('l') => {
                            write_items(&items, &day_path)?;

                            day_offset += 1;
                            day_name = date(day_offset, &config.date_format);
                            day_path = Path::new(&config.path).join(format!("{}.md", day_name));
                            items = read_items(&day_path, &config.habits)?;
                            items_state = ListState::default();
                        }
                        KeyCode::Char('j') => {
                            if items.len() > 0 {
                                let i = match items_state.selected() {
                                    Some(i) => (i + 1) % items.len(),
                                    None => 0,
                                };

                                items_state.select(Some(i));
                            }
                        }
                        KeyCode::Char('k') => {
                            if items.len() > 0 {
                                let i = match items_state.selected() {
                                    Some(i) => (i + items.len() - 1) % items.len(),
                                    None => items.len() - 1,
                                };

                                items_state.select(Some(i));
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some(i) = items_state.selected() {
                                items[i].toggle();
                            }

                            write_items(&items, &day_path)?;
                        }
                        KeyCode::Char('a') => {
                            input_mode = InputMode::Insert;
                        }
                        KeyCode::Char('d') => {
                            if let Some(i) = items_state.selected() {
                                items.remove(i);

                                if items.len() == 0 {
                                    items_state.select(None);
                                } else {
                                    items_state.select(Some((i + items.len() - 1) % items.len()));
                                }
                            }

                            write_items(&items, &day_path)?;
                        }
                        _ => {}
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Enter => {
                            items.push(Item::new(input_text.drain(..).collect()));
                            input_mode = InputMode::Normal;

                            write_items(&items, &day_path)?;
                        }
                        KeyCode::Char(c) => input_text.push(c),
                        KeyCode::Backspace => {
                            input_text.pop();
                        }
                        KeyCode::Esc => {
                            input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;

    Ok(())
}
