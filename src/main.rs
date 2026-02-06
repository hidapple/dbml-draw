mod cli;
mod error;
mod ir;
mod layout;
mod parser;
mod render;

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
        } => cmd_generate(input, output, layout)?,
    }

    Ok(())
}

fn cmd_generate(
    input: PathBuf,
    output: Option<PathBuf>,
    layout_path: Option<PathBuf>,
) -> Result<(), AppError> {
    // Read DBML and parse it into IR
    let dbml_content = std::fs::read_to_string(&input)?;
    let mut diagram = parser::parse_dbml(&dbml_content)?;

    // Apply layout
    layout::apply_layout(&mut diagram, layout_path.as_deref());

    // Render SVG
    let svg = render::render_svg(&diagram);

    // Write output file
    let output_path = output.unwrap_or_else(|| {
        let mut p = input.clone();
        p.set_extension("svg");
        p
    });
    std::fs::write(&output_path, &svg)?;
    eprintln!("Generated: {}", output_path.display());

    Ok(())
}
