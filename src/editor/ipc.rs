use std::collections::HashMap;
use std::path::Path;

use base64::Engine;
use serde::Deserialize;

use crate::ir::{Diagram, Position};
use crate::layout::layout_file;
use crate::layout::types::{LayoutData, LayoutMeta, TableLayout};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "table_moved")]
    TableMoved { table_id: String, x: f64, y: f64 },
    #[serde(rename = "save_layout")]
    SaveLayout {
        tables: HashMap<String, TablePosition>,
    },
    #[serde(rename = "export_png")]
    ExportPng { data_url: String },
}

#[derive(Debug, Deserialize)]
pub struct TablePosition {
    pub x: f64,
    pub y: f64,
}

pub fn parse_ipc_message(body: &str) -> Result<IpcMessage, String> {
    serde_json::from_str(body).map_err(|e| format!("Failed to parse IPC message: {}", e))
}

/// Handle table_moved: update diagram position and save layout.toml
pub fn handle_table_moved(
    diagram: &mut Diagram,
    layout_path: &Path,
    dbml_path: &Path,
    table_id: &str,
    x: f64,
    y: f64,
) {
    if let Some(table) = diagram
        .tables
        .iter_mut()
        .find(|t| t.id.full_name() == table_id)
    {
        table.position = Some(Position { x, y });
    }

    save_all_positions(diagram, layout_path, dbml_path);
}

/// Handle save_layout: bulk-save all table positions
pub fn handle_save_layout(
    diagram: &mut Diagram,
    layout_path: &Path,
    dbml_path: &Path,
    tables: &HashMap<String, TablePosition>,
) {
    for (table_id, pos) in tables {
        if let Some(table) = diagram
            .tables
            .iter_mut()
            .find(|t| t.id.full_name() == *table_id)
        {
            table.position = Some(Position { x: pos.x, y: pos.y });
        }
    }

    save_all_positions(diagram, layout_path, dbml_path);
}

/// Handle export_png: decode base64 data URL and write PNG file
pub fn handle_export_png(dbml_path: &Path, data_url: &str) -> String {
    let base64_data = match data_url.strip_prefix("data:image/png;base64,") {
        Some(d) => d,
        None => {
            eprintln!("Invalid PNG data URL");
            return String::new();
        }
    };

    let png_bytes = match base64::engine::general_purpose::STANDARD.decode(base64_data) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to decode base64: {}", e);
            return String::new();
        }
    };

    let output_path = dbml_path.with_extension("png");
    match std::fs::write(&output_path, &png_bytes) {
        Ok(_) => output_path.display().to_string(),
        Err(e) => {
            eprintln!("Failed to write PNG: {}", e);
            String::new()
        }
    }
}

fn save_all_positions(diagram: &Diagram, layout_path: &Path, dbml_path: &Path) {
    let mut tables = HashMap::new();
    for table in &diagram.tables {
        if let Some(pos) = &table.position {
            tables.insert(table.id.full_name(), TableLayout { x: pos.x, y: pos.y });
        }
    }

    let layout_data = LayoutData {
        meta: LayoutMeta {
            version: 1,
            source: dbml_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default(),
        },
        tables,
    };

    if let Err(e) = layout_file::write_layout(layout_path, &layout_data) {
        eprintln!("Failed to save layout: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_moved() {
        let json = r#"{"type":"table_moved","table_id":"public.users","x":100.0,"y":200.0}"#;
        let msg = parse_ipc_message(json).unwrap();
        match msg {
            IpcMessage::TableMoved { table_id, x, y } => {
                assert_eq!(table_id, "public.users");
                assert!((x - 100.0).abs() < f64::EPSILON);
                assert!((y - 200.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected TableMoved"),
        }
    }

    #[test]
    fn test_parse_save_layout() {
        let json = r#"{"type":"save_layout","tables":{"public.users":{"x":100,"y":200},"public.posts":{"x":400,"y":200}}}"#;
        let msg = parse_ipc_message(json).unwrap();
        match msg {
            IpcMessage::SaveLayout { tables } => {
                assert_eq!(tables.len(), 2);
                let users = tables.get("public.users").unwrap();
                assert!((users.x - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected SaveLayout"),
        }
    }

    #[test]
    fn test_parse_export_png() {
        let json = r#"{"type":"export_png","data_url":"data:image/png;base64,iVBOR"}"#;
        let msg = parse_ipc_message(json).unwrap();
        assert!(matches!(msg, IpcMessage::ExportPng { .. }));
    }

    #[test]
    fn test_parse_invalid_message() {
        let json = r#"{"type":"unknown"}"#;
        assert!(parse_ipc_message(json).is_err());
    }

    #[test]
    fn test_layout_roundtrip() {
        use crate::ir::{Column, Position, Table, TableId};
        use tempfile::NamedTempFile;

        let mut diagram = Diagram {
            tables: vec![
                Table {
                    id: TableId::new("public", "users"),
                    columns: vec![Column {
                        name: "id".into(),
                        type_raw: "int".into(),
                        is_pk: true,
                        is_nullable: false,
                    }],
                    position: Some(Position { x: 100.0, y: 200.0 }),
                },
                Table {
                    id: TableId::new("public", "posts"),
                    columns: vec![Column {
                        name: "id".into(),
                        type_raw: "int".into(),
                        is_pk: true,
                        is_nullable: false,
                    }],
                    position: Some(Position { x: 400.0, y: 200.0 }),
                },
            ],
            relationships: vec![],
        };

        let tmp = NamedTempFile::new().unwrap();
        let layout_path = tmp.path().to_path_buf();
        let dbml_path = std::path::PathBuf::from("test.dbml");

        handle_table_moved(
            &mut diagram,
            &layout_path,
            &dbml_path,
            "public.users",
            150.0,
            250.0,
        );

        let layout_data = layout_file::read_layout(&layout_path).unwrap();
        let users = layout_data.tables.get("public.users").unwrap();
        assert!((users.x - 150.0).abs() < f64::EPSILON);
        assert!((users.y - 250.0).abs() < f64::EPSILON);

        let posts = layout_data.tables.get("public.posts").unwrap();
        assert!((posts.x - 400.0).abs() < f64::EPSILON);
        assert!((posts.y - 200.0).abs() < f64::EPSILON);
    }
}
