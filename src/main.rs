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
            write!(f, "{}\n", self.items[i])?;
        }

        if m < n {
            write!(f, "*{}*\n", self.items[m])?;
        }

        for i in m+1..n {
            write!(f, "{}\n", self.items[i])?;
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
}

impl<'a> Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.text);
    }
}

impl<'a> Item<'a> {
    fn new(text: &'a str) -> Self {
        return Item { text };
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

    app.items.push(Item::new("item1"));
    app.items.push(Item::new("item2"));
    app.items.push(Item::new("item3"));

    println!("{}", app.items);

    return Ok(());
}
