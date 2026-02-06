use crate::ir::{Position, Table};
use svg::node::element::{Group, Line, Rectangle, Text};

use super::style::*;
use super::table_height;

pub fn render_table(table: &Table) -> Group {
    let pos = table.position.unwrap_or(Position { x: 0.0, y: 0.0 });
    let mut group = Group::new().set("transform", format!("translate({}, {})", pos.x, pos.y));

    let h = table_height(table);
    let border = Rectangle::new()
        .set("width", TABLE_WIDTH)
        .set("height", h)
        .set("fill", TABLE_BG)
        .set("stroke", TABLE_BORDER)
        .set("rx", BORDER_RADIUS);
    group = group.add(border);

    let header_bg = Rectangle::new()
        .set("width", TABLE_WIDTH)
        .set("height", HEADER_HEIGHT)
        .set("fill", HEADER_BG);
    group = group.add(header_bg);

    let header_text = Text::new(&table.id.name)
        .set("x", PADDING_X)
        .set("y", HEADER_HEIGHT / 2.0 + HEADER_FONT_SIZE / 3.0)
        .set("font-family", "monospace")
        .set("font-size", HEADER_FONT_SIZE)
        .set("font-weight", "bold")
        .set("fill", HEADER_TEXT);
    group = group.add(header_text);

    let sep = Line::new()
        .set("x1", 0)
        .set("y1", HEADER_HEIGHT)
        .set("x2", TABLE_WIDTH)
        .set("y2", HEADER_HEIGHT)
        .set("stroke", TABLE_BORDER);
    group = group.add(sep);

    for (i, col) in table.columns.iter().enumerate() {
        let row_y = HEADER_HEIGHT + (i as f64 * ROW_HEIGHT);
        let col_text = if col.is_pk {
            format!("🔑 {}", col.name)
        } else {
            col.name.clone()
        };
        let name_text = Text::new(&col_text)
            .set("x", PADDING_X)
            .set("y", row_y + ROW_HEIGHT / 2.0 + FONT_SIZE / 3.0)
            .set("font-family", "monospace")
            .set("font-size", FONT_SIZE)
            .set("fill", if col.is_pk { PK_COLOR } else { COLUMN_TEXT });
        group = group.add(name_text);

        let type_text = Text::new(&col.type_raw)
            .set("x", TABLE_WIDTH - PADDING_X)
            .set("y", row_y + ROW_HEIGHT / 2.0 + FONT_SIZE / 3.0)
            .set("font-family", "monospace")
            .set("font-size", FONT_SIZE)
            .set("fill", TYPE_TEXT)
            .set("text-anchor", "end");
        group = group.add(type_text);
    }
    group
}
