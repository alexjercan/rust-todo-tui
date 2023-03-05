use chrono::{self, Days};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    fmt::Display,
    fs::{self, OpenOptions},
    io::{self, Read},
    path::Path,
    slice::Iter,
    str::FromStr,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Debug)]
enum Error {
    ParseError,
    IOError(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        return Self::IOError(value);
    }
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        return StatefulList {
            state: ListState::default(),
            items: Vec::default(),
        };
    }
}

impl<T> IntoIterator for StatefulList<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        return self.items.into_iter();
    }
}

impl<T> FromIterator<T> for StatefulList<T> {
    fn from_iter<V: IntoIterator<Item = T>>(iter: V) -> Self {
        let mut items = StatefulList::default();

        for item in iter {
            items.push(item);
        }

        return items;
    }
}

impl<T> StatefulList<T> {
    fn push(&mut self, item: T) {
        if self.items.len() == 0 {
            self.state.select(Some(0));
        }

        self.items.push(item);
    }

    fn iter(&self) -> Iter<'_, T> {
        return self.items.iter();
    }

    fn next(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + self.items.len() - 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn remove(&mut self) {
        if self.items.len() == 0 {
            return;
        }

        let i = self.state.selected().unwrap();

        if self.items.len() - 1 == i {
            self.prev();
        }

        self.items.remove(i);
    }

    fn selected_mut(&mut self) -> Option<&mut T> {
        let i = self.state.selected()?;

        return Some(&mut self.items[i]);
    }
}

#[derive(Debug)]
struct Item {
    text: String,
    completed: bool,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.completed { "[X]" } else { "[ ]" };
        return write!(f, "{} {}", status, self.text);
    }
}

impl FromStr for Item {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let text = &s[4..];

        match &s[..3] {
            "[ ]" => {
                return Ok(Item {
                    text: text.to_string(),
                    completed: false,
                })
            }
            "[X]" => {
                return Ok(Item {
                    text: text.to_string(),
                    completed: true,
                })
            }
            _ => return Err(Self::Err::ParseError),
        }
    }
}

impl Item {
    fn new(text: String) -> Self {
        return Item {
            text,
            completed: false,
        };
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

enum InputMode {
    Normal,
    Editing,
}

impl Default for InputMode {
    fn default() -> Self {
        return InputMode::Normal;
    }
}

struct App {
    items: StatefulList<Item>,

    input: String,
    input_mode: InputMode,
}

impl Default for App {
    fn default() -> Self {
        return App {
            items: StatefulList::default(),
            input: String::default(),
            input_mode: InputMode::default(),
        };
    }
}

impl Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.items.iter() {
            writeln!(f, "{}", item)?;
        }

        return Ok(());
    }
}

impl FromStr for App {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .lines()
            .map(|line| line.parse::<Item>())
            .collect::<Result<_, _>>()?;

        return Ok(App::with_items(items));
    }
}

impl App {
    fn with_items(items: StatefulList<Item>) -> Self {
        let mut app = App::default();
        app.items = items;
        return app;
    }

    fn save<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        fs::write(path, self.to_string())?;

        return Ok(());
    }

    fn load<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut data = String::new();

        let _ = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?
            .read_to_string(&mut data)?;

        return data.parse();
    }

    fn on_tick(&mut self) {}

    fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        tick_rate: Duration,
    ) -> Result<(), Error> {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('j') => self.items.next(),
                            KeyCode::Char('k') => self.items.prev(),
                            KeyCode::Char('x') => match self.items.selected_mut() {
                                Some(item) => item.toggle(),
                                None => {}
                            },
                            KeyCode::Char('a') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('d') => {
                                self.items.remove();
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                self.items.push(Item::new(self.input.drain(..).collect()));
                                self.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                self.input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(90),
                    Constraint::Percentage(5),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(f.size());

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| -> ListItem { ListItem::new(i.to_string()).style(Style::default()) })
            .collect();

        let items = List::new(items)
            .block(Block::default().title("TODO App").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("k", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to move up, "),
                    Span::styled("j", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to move down, "),
                    Span::styled("x", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to toggle, "),
                    Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to add new todo,"),
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
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text).wrap(Wrap { trim: true });

        f.render_stateful_widget(items, chunks[0], &mut self.items.state);
        f.render_widget(help_message, chunks[1]);

        match self.input_mode {
            InputMode::Normal => {}
            _ => {
                let input = Paragraph::new(self.input.as_ref());
                f.render_widget(input, chunks[2]);
                f.set_cursor(chunks[2].x + self.input.len() as u16, chunks[2].y);
            }
        }
    }
}

/// Simple TUI TODO Application for daily tasks.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Name of the todo file to use
    #[arg(short, long)]
    name: Option<String>,

    /// Import a list of todos from a file as a template
    #[arg(short, long)]
    import: Option<String>,

    /// Creates a todo list for tomorrow
    #[arg(short, long)]
    tomorrow: bool,
}

fn main() -> Result<(), Error> {
    let xdg_config_home =
        env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME").unwrap_or(".".to_string()));
    let todo_path = Path::new(&xdg_config_home).join("todo-tui").join("todo");
    fs::create_dir_all(&todo_path)?;

    let args = Args::parse();

    let name = match args.name {
        Some(name) => name,
        None => {
            let date = chrono::Utc::now().date_naive();

            let date = if args.tomorrow {
                date.checked_add_days(Days::new(1))
                    .expect("to be able to compute next day")
            } else {
                date
            };

            format!("{}", date)
        }
    };
    let app_path = todo_path.join(name);
    let mut app = App::load(&app_path)?;

    if let Some(import) = args.import {
        let import_path = todo_path.join(import);
        let import = App::load(&import_path)?;
        for item in import.items.into_iter() {
            app.items.push(item);
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let res = app.run(&mut terminal, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    app.save(&app_path)?;

    return Ok(());
}
