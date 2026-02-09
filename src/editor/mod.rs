mod assets;
mod ipc;
mod webview;

use std::path::PathBuf;

use crate::error::AppError;
use crate::ir::Diagram;

pub fn open_editor(
    diagram: Diagram,
    dbml_path: PathBuf,
    layout_path: PathBuf,
) -> Result<(), AppError> {
    webview::run(diagram, dbml_path, layout_path)
}
