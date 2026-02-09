# dbml-draw

A CLI tool that visualizes ER diagrams from [DBML](https://dbml.dbdiagram.io/) files.
Opens an interactive viewer with IE notation (crow's foot) relationship markers. Tables can be dragged to rearrange, and the diagram can be exported as PNG.

![demo](docs/demo.gif)

## Install

```sh
cargo install --path .
```

## Usage

```sh
dbml-draw <COMMAND>
```

### `open`

Open an interactive viewer for a DBML file.

```sh
dbml-draw open <INPUT>
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input DBML file path |

#### Example

```sh
dbml-draw open schema.dbml
```

## License

MIT
