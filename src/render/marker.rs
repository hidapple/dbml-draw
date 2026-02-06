use crate::ir::{Column, RelationType, Relationship, Table};
use svg::node::element::{Circle, Definitions, Line, Marker, Path};

use super::style::*;

/// IE notation marker types.
#[derive(Clone, Copy)]
pub enum IeMarker {
    OneMandatory,  // ||  (exactly one)
    OneOptional,   // |o  (zero or one)
    ManyMandatory, // |<  (one or many)
    ManyOptional,  // o<  (zero or many)
}

/// Create SVG `<defs>` containing all IE notation markers (4 types × start/end).
pub fn create_markers() -> Definitions {
    let mut defs = Definitions::new();

    defs = defs.add(build_marker("one-mandatory-start", 4, |m| {
        m.add(vertical_line(16)).add(vertical_line(22))
    }));
    defs = defs.add(build_marker("one-mandatory-end", 28, |m| {
        m.add(vertical_line(10)).add(vertical_line(16))
    }));

    defs = defs.add(build_marker("one-optional-start", 4, |m| {
        m.add(vertical_line(16)).add(circle_symbol(22))
    }));
    defs = defs.add(build_marker("one-optional-end", 28, |m| {
        m.add(circle_symbol(10)).add(vertical_line(16))
    }));

    defs = defs.add(build_marker("many-mandatory-start", 4, |m| {
        m.add(crow_foot_start(6)).add(vertical_line(24))
    }));
    defs = defs.add(build_marker("many-mandatory-end", 28, |m| {
        m.add(vertical_line(8)).add(crow_foot_end(26))
    }));

    defs = defs.add(build_marker("many-optional-start", 4, |m| {
        m.add(crow_foot_start(6)).add(circle_symbol(22))
    }));
    defs = defs.add(build_marker("many-optional-end", 28, |m| {
        m.add(circle_symbol(10)).add(crow_foot_end(26))
    }));

    defs
}

/// Determine the IE marker types for a relationship based on relation type and FK nullability.
pub fn determine_ie_markers(
    rel: &Relationship,
    from_table: &Table,
    to_table: &Table,
) -> (IeMarker, IeMarker) {
    match rel.relation_type {
        RelationType::ManyToOne => {
            let fk_nullable = fk_is_nullable(&rel.from.column_names, from_table);
            let from_marker = if fk_nullable { IeMarker::ManyOptional } else { IeMarker::ManyMandatory };
            let to_marker = if fk_nullable { IeMarker::OneOptional } else { IeMarker::OneMandatory };
            (from_marker, to_marker)
        }
        RelationType::OneToMany => {
            let fk_nullable = fk_is_nullable(&rel.to.column_names, to_table);
            let from_marker = if fk_nullable { IeMarker::OneOptional } else { IeMarker::OneMandatory };
            let to_marker = if fk_nullable { IeMarker::ManyOptional } else { IeMarker::ManyMandatory };
            (from_marker, to_marker)
        }
        RelationType::OneToOne => {
            let from_nullable = fk_is_nullable(&rel.from.column_names, from_table);
            let from_marker = if from_nullable { IeMarker::OneOptional } else { IeMarker::OneMandatory };
            let to_marker = if from_nullable { IeMarker::OneOptional } else { IeMarker::OneMandatory };
            (from_marker, to_marker)
        }
        RelationType::ManyToMany => (IeMarker::ManyOptional, IeMarker::ManyOptional),
    }
}

/// Return the SVG marker id string for a given IE marker type and position (start/end).
pub fn marker_id(marker: IeMarker, position: &str) -> String {
    let name = match marker {
        IeMarker::OneMandatory => "one-mandatory",
        IeMarker::OneOptional => "one-optional",
        IeMarker::ManyMandatory => "many-mandatory",
        IeMarker::ManyOptional => "many-optional",
    };
    format!("{}-{}", name, position)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn fk_is_nullable(column_names: &[String], table: &Table) -> bool {
    let col_name = column_names.first().map(|s| s.as_str()).unwrap_or("");
    find_column(table, col_name)
        .map(|c| c.is_nullable)
        .unwrap_or(true)
}

fn find_column<'a>(table: &'a Table, name: &str) -> Option<&'a Column> {
    table.columns.iter().find(|c| c.name == name)
}

fn build_marker(id: &str, ref_x: i32, builder: impl FnOnce(Marker) -> Marker) -> Marker {
    let m = Marker::new()
        .set("id", id)
        .set("markerWidth", 32)
        .set("markerHeight", 24)
        .set("refX", ref_x)
        .set("refY", 12)
        .set("orient", "auto")
        .set("markerUnits", "userSpaceOnUse");
    builder(m)
}

fn vertical_line(x: i32) -> Line {
    Line::new()
        .set("x1", x)
        .set("y1", 3)
        .set("x2", x)
        .set("y2", 21)
        .set("stroke", RELATION_STROKE)
        .set("stroke-width", RELATION_STROKE_WIDTH)
}

fn circle_symbol(cx: i32) -> Circle {
    Circle::new()
        .set("cx", cx)
        .set("cy", 12)
        .set("r", 5)
        .set("stroke", RELATION_STROKE)
        .set("stroke-width", RELATION_STROKE_WIDTH)
        .set("fill", "white")
}

fn crow_foot_start(tip_x: i32) -> Path {
    let base_x = tip_x + 10;
    Path::new()
        .set(
            "d",
            format!("M {} 12 L {} 3 M {} 12 L {} 21", base_x, tip_x, base_x, tip_x),
        )
        .set("stroke", RELATION_STROKE)
        .set("stroke-width", RELATION_STROKE_WIDTH)
        .set("fill", "none")
}

fn crow_foot_end(tip_x: i32) -> Path {
    let base_x = tip_x - 10;
    Path::new()
        .set(
            "d",
            format!("M {} 12 L {} 3 M {} 12 L {} 21", base_x, tip_x, base_x, tip_x),
        )
        .set("stroke", RELATION_STROKE)
        .set("stroke-width", RELATION_STROKE_WIDTH)
        .set("fill", "none")
}
