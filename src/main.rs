use chrono::{self, Days, NaiveDate};
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
    widgets::{Block, Borders, List, ListItem, ListState},
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
    fn new(text: &str) -> Self {
        return Item {
            text: text.to_string(),
            completed: false,
        };
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}

struct App {
    items: StatefulList<Item>,
}

impl Default for App {
    fn default() -> Self {
        return App {
            items: StatefulList::default(),
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

        return Ok(App { items });
    }
}

impl App {
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
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') => self.items.next(),
                        KeyCode::Char('k') => self.items.prev(),
                        KeyCode::Char('x') => match self.items.selected_mut() {
                            Some(item) => item.toggle(),
                            None => {}
                        },
                        _ => {}
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
            .constraints([Constraint::Percentage(100)].as_ref())
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

        f.render_stateful_widget(items, chunks[0], &mut self.items.state);
    }
}

/// Simple TUI TODO Application for daily tasks.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Name of the todo file to use
    #[arg(short, long)]
    name: Option<String>,

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
    let todo_path = todo_path.join(name);

    let mut app = App::load(&todo_path)?;

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

    app.save(&todo_path)?;

    return Ok(());
}
