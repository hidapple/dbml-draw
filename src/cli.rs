use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dbml-draw", version, about = "Generate ER diagrams from DBML files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate an SVG diagram from a DBML file
    Generate {
        /// Input DBML file path
        input: PathBuf,

        /// Output SVG file path (defaults to <input>.svg)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Layout file path (defaults to <input>.layout.toml)
        #[arg(long)]
        layout: Option<PathBuf>,

        /// Force auto-layout, ignoring existing layout file
        #[arg(long)]
        auto_layout: bool,
    },
}
