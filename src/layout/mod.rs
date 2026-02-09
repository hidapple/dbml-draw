use crate::ir::Position;
use crate::ir::Diagram;
use std::path::Path;

pub(crate) mod layout_file;
pub(crate) mod types;

pub fn apply_layout(diagram: &mut Diagram, layout_path: Option<&Path>) {
    if let Some(path) = layout_path {
        if path.exists() {
            if let Ok(layout_data) = layout_file::read_layout(path) {
                for table in &mut diagram.tables {
                    let key = table.id.full_name();
                    if let Some(tl) = layout_data.tables.get(&key) {
                        table.position = Some(Position { x: tl.x, y: tl.y });
                    }
                }
            }
        }
    }
}
