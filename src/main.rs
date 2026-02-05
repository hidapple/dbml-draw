mod cli;
mod error;
mod ir;
mod parser;

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Commands};
use error::AppError;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input,
            output,
            layout,
            auto_layout,
        } => cmd_generate(input, output, layout, auto_layout)?,
    }

    Ok(())
}

fn cmd_generate(
    input: PathBuf,
    _output: Option<PathBuf>,
    _layout_path: Option<PathBuf>,
    _auto_layout: bool,
) -> Result<(), AppError> {
    let dbml_content = std::fs::read_to_string(&input)?;
    let diagram = parser::parse_dbml(&dbml_content)?;

    eprintln!(
        "Parsed: {} tables, {} relationships",
        diagram.tables.len(),
        diagram.relationships.len()
    );

    Ok(())
}
