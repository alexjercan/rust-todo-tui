# TODO TUI

Simple TUI TODO Application for daily tasks.

## QuickStart

```console
cargo run
```

## Config

The tool creates a folder `todo-tui/todo` in `XDG_CONFIG_HOME`. This is usually in
the home folder in `.config`. When creating new todo lists all the todos are
saved as lists in the `todo` folder.

### Default Name

When running the tool with no arguments, a file corresponding to the current date
is created in the `todo` directory, with the format `YYYY-MM-DD` and all the
changes from the tool will be saved on exit.

### Custom Name

You can also specify a name that you want to use instead of the current day.
You can do that with the `name` flag and specify the new name to use for the
file.

```console
cargo run -- --name other-name
```

This will create a file with the name `other-name` in the `todo` directory.

### Tomorrow

If you use the `tomorrow` flag, and do not specify a name, the default name
will be set to the tomorrow date. This allows you to create a todo list for the
next day in an easy way.

```console
cargo run -- --tomorrow
```

This will create a file in the `todo` directory with the format `YYYY-MM-DD`
for the next day.

### Import

To use a todo list a predefined set of todo's, you can use the `import` flag
and specify the name of the todo file to use as default todo's. Say you have a
file `default` in the `todo` directory, and you use

```console
cargo run -- --import default
```

This will append all the todo's from the `default` file to the current day.

I think that the most useful usage for this command is to set some todo's that
you have to perform daily in a `default` file and then run it with:

```console
cargo run -- --tomorrow --import default
```

This will create a todo list for tomorrow with the default items inside. Then
you can also add other todo's that you need to finish next day, but it already
adds the default ones by itself.

