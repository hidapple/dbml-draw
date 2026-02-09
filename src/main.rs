mod cli;
mod editor;
mod error;
mod ir;
mod layout;
mod parser;

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Commands};
use error::AppError;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Edit { input } => cmd_edit(input)?,
    }

    Ok(())
}

fn cmd_edit(input: PathBuf) -> Result<(), AppError> {
    let dbml_content = std::fs::read_to_string(&input)?;
    let mut diagram = parser::parse_dbml(&dbml_content)?;

    // Derive layout file path from input (e.g., schema.dbml -> schema.layout.toml)
    let layout_path = input.with_extension("layout.toml");
    layout::apply_layout(&mut diagram, Some(layout_path.as_path()));

    editor::open_editor(diagram, input, layout_path)
}
