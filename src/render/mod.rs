mod marker;
mod relationship;
mod style;
mod table;

use crate::ir::{Diagram, Table};
use svg::node::element::Rectangle;
use svg::Document;

use style::*;

pub fn render_svg(diagram: &Diagram) -> String {
    let (width, height) = compute_canvas_size(diagram);

    let mut doc = Document::new()
        .set("width", width)
        .set("height", height)
        .set("viewBox", format!("0 0 {} {}", width, height));

    let bg = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", CANVAS_BG);
    doc = doc.add(bg);

    doc = doc.add(marker::create_markers());

    // Relationships (draw first, so tables appear on top)
    for group in relationship::render_relationships(diagram) {
        doc = doc.add(group);
    }

    // Tables
    for t in &diagram.tables {
        doc = doc.add(table::render_table(t));
    }

    doc.to_string()
}

fn compute_canvas_size(diagram: &Diagram) -> (f64, f64) {
    let mut max_x: f64 = 800.0;
    let mut max_y: f64 = 600.0;

    for t in &diagram.tables {
        if let Some(pos) = &t.position {
            let h = table_height(t);
            max_x = max_x.max(pos.x + TABLE_WIDTH + 50.0);
            max_y = max_y.max(pos.y + h + 50.0);
        }
    }

    (max_x, max_y)
}

pub(crate) fn table_height(table: &Table) -> f64 {
    HEADER_HEIGHT + (table.columns.len() as f64 * ROW_HEIGHT)
}
