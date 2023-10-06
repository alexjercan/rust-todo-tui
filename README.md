<div align="center">

# TODO TUI

#### Simple TUI TODO Application for daily tasks.

![todo](https://i.imgur.com/58yCwly.png)

</div>

## ⇁ QuickStart

```console
cargo run
```

## ⇁ Subcommands

By default, the application will start in TUI mode, but you can also visualize
statistics of your tasks using sub-commands.

- `status` this sub-commands will display the number of tasks done out of the
  total, for example `2/8`.
- `details` this sub-command will display the list of items to stdout as
  Markdown

## ⇁ Installation

Build the cargo project:

```console
cargo build --reslease
cp ./target/release/todo-tui ~/.local/bin/todo-tui
```

Open a new terminal and check if the tool is working (also make sure that
~/.local/bin/ is on the `PATH`)

```console
todo-tui --version
```

## ⇁ Config

The tools will look for a configuration file in `XDG_CONFIG_HOME`. It will
search for `todo/config.json`. If it cannot find the file, it will use the
default settings.

You can specify the path to a custom config file using the `-c/--config`
argument.

The configuration file is in JSON format, and it has the following properties.

- `path`: The path to the task's directory. This is where all the markdown
  files will be saved and loaded from. By default, it will be set to
  `$HOME/.config/todo/`.
- `date_format`: The format of date that will be used to name the files and use
  as a display title in the TUI. By default, it is `%Y-%m-%d`.
- `habits`: A list of custom items that will be prepended to each task file on
  creation. By default, it will be an empty list `[]`.

Configuration example

```json
{
    "path": "/home/alex/personal/todo",
    "date_format": "%Y-%m-%d",
    "habits": [
        "🧼 Morning Routine",
        "📕 Read",
        "💪 Gym",
        "✍️ Journal",
    ]
}
```
