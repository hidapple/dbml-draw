use crate::ir::{Diagram, Position};

pub fn grid_layout(diagram: &mut Diagram) {
    const TABLE_WIDTH: f64 = 260.0;
    const SPACING_X: f64 = 100.0;
    const SPACING_Y: f64 = 80.0;
    const ROW_HEIGHT: f64 = 28.0;
    const HEADER_HEIGHT: f64 = 36.0;
    const COLS_PER_ROW: usize = 3; // 3 tables in one row
    const START_X: f64 = 50.0;
    const START_Y: f64 = 50.0;

    let mut x = START_X;
    let mut y = START_Y;
    let mut col_index = 0;
    let mut max_row_height: f64 = 0.0;

    for table in &mut diagram.tables {
        if table.position.is_some() {
            continue;
        }
        table.position = Some(Position { x, y });

        let table_height = HEADER_HEIGHT + (table.columns.len() as f64 * ROW_HEIGHT);
        max_row_height = max_row_height.max(table_height);

        col_index += 1;
        if col_index >= COLS_PER_ROW {
            col_index = 0;
            x = START_X;
            y += max_row_height + SPACING_Y;
            max_row_height = 0.0;
        } else {
            x += TABLE_WIDTH + SPACING_X;
        }
    }
}
