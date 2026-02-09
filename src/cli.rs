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
    /// Open an interactive editor for a DBML file
    Edit {
        /// Input DBML file path
        input: PathBuf,
    },
}
