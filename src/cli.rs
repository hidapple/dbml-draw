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
    /// Open an interactive viewer for a DBML file
    Open {
        /// Input DBML file path
        input: PathBuf,
    },
}
