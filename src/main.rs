use std::{fmt::Display, io};

use tui::widgets::ListState;

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

impl<T> Display for StatefulList<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.items.len();

        if n == 0 {
            return Ok(());
        }

        let m = match self.state.selected() {
            Some(i) => i,
            None => 0,
        };

        for i in 0..m {
            writeln!(f, "{}", self.items[i])?;
        }

        if m < n {
            writeln!(f, "{}<<<", self.items[m])?;
        }

        for i in m+1..n {
            writeln!(f, "{}", self.items[i])?;
        }

        return Ok(());
    }
}

impl<T> StatefulList<T> {
    fn push(&mut self, item: T) {
        if self.items.len() == 0 {
            self.state.select(Some(0));
        }

        self.items.push(item);
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + self.items.len() - 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct Item<'a> {
    text: &'a str,
    completed: bool,
}

impl<'a> Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.completed { "[X]" } else { "[ ]" };
        return write!(f, "{} {}", status, self.text);
    }
}

impl<'a> Item<'a> {
    fn new(text: &'a str) -> Self {
        return Item { text, completed: false };
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
    }

    fn set(&mut self, value: bool) {
        self.completed = value;
    }
}

struct App<'a> {
    items: StatefulList<Item<'a>>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        return App { items: StatefulList::default() };
    }
}

fn main() -> Result<(), io::Error> {
    let mut app = App::default();

    app.items.push(Item::new("Use TUI to display items"));
    app.items.push(Item::new("Read and write lines to a file"));
    app.items.push(Item::new("Make the file be date based by default current date"));
    app.items.push(Item::new("be able to move trough tasks using j and k"));
    app.items.push(Item::new("The selected item should be colored or something"));
    app.items.push(Item::new("be able to insert a task at current position using i"));
    app.items.push(Item::new("be able to append a task at the end using a"));
    app.items.push(Item::new("be able to toggle a task using x"));
    app.items.push(Item::new("maybe I will have default tasks in a .config file"));

    println!("{}", app.items);

    return Ok(());
}
