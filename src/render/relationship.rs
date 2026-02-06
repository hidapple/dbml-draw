use std::collections::HashMap;

use crate::ir::{Diagram, Position, Table};
use svg::node::element::{Group, Path};

use super::marker::{determine_ie_markers, marker_id};
use super::style::*;
use super::table_height;

/// Which side of a table a connection attaches to.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// Pre-compute all relationship routes, distribute mid-points to avoid overlap,
/// then render as SVG groups.
pub fn render_relationships(diagram: &Diagram) -> Vec<Group> {
    let n = diagram.relationships.len();
    if n == 0 {
        return vec![];
    }

    let routes = compute_routes(diagram);
    let mid_values = distribute_corridors(&routes);

    let mut groups: Vec<Group> = Vec::new();

    for (i, rel) in diagram.relationships.iter().enumerate() {
        let info = match &routes[i] {
            Some(r) => r,
            None => continue,
        };

        let from_table = &diagram.tables[info.from_idx];
        let to_table = &diagram.tables[info.to_idx];

        let path_d = build_path(info, mid_values[i]);
        let (from_marker, to_marker) = determine_ie_markers(rel, from_table, to_table);

        let path = Path::new()
            .set("d", path_d.as_str())
            .set("stroke", RELATION_STROKE)
            .set("stroke-width", RELATION_STROKE_WIDTH)
            .set("fill", "none")
            .set("marker-start", format!("url(#{})", marker_id(from_marker, "start")))
            .set("marker-end", format!("url(#{})", marker_id(to_marker, "end")));

        groups.push(Group::new().add(path));
    }

    groups
}

// ---------------------------------------------------------------------------
// Route computation
// ---------------------------------------------------------------------------

struct RouteInfo {
    from_idx: usize,
    to_idx: usize,
    from_side: Side,
    to_side: Side,
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
}

/// For each relationship, determine connection sides and coordinates.
fn compute_routes(diagram: &Diagram) -> Vec<Option<RouteInfo>> {
    let mut routes = Vec::with_capacity(diagram.relationships.len());

    for rel in &diagram.relationships {
        let from_idx = diagram.tables.iter().position(|t| t.id == rel.from.table_id);
        let to_idx = diagram.tables.iter().position(|t| t.id == rel.to.table_id);

        if let (Some(fi), Some(ti)) = (from_idx, to_idx) {
            let ft = &diagram.tables[fi];
            let tt = &diagram.tables[ti];
            let fp = ft.position.unwrap_or(Position { x: 0.0, y: 0.0 });
            let tp = tt.position.unwrap_or(Position { x: 0.0, y: 0.0 });

            let (from_side, to_side) = determine_sides(fp, table_height(ft), tp, table_height(tt));

            let from_col = rel.from.column_names.first().map(|s| s.as_str()).unwrap_or("");
            let to_col = rel.to.column_names.first().map(|s| s.as_str()).unwrap_or("");

            let (from_x, from_y) = connection_point(fp, ft, from_side, from_col);
            let (to_x, to_y) = connection_point(tp, tt, to_side, to_col);

            routes.push(Some(RouteInfo {
                from_idx: fi,
                to_idx: ti,
                from_side,
                to_side,
                from_x,
                from_y,
                to_x,
                to_y,
            }));
        } else {
            routes.push(None);
        }
    }

    routes
}

/// Determine which sides to connect based on relative table positions.
/// Same column (horizontal overlap) → vertical (Top/Bottom).
/// Different columns → horizontal (Left/Right).
fn determine_sides(
    from_pos: Position,
    from_h: f64,
    to_pos: Position,
    to_h: f64,
) -> (Side, Side) {
    let h_overlap =
        from_pos.x < to_pos.x + TABLE_WIDTH && to_pos.x < from_pos.x + TABLE_WIDTH;

    if h_overlap {
        let from_cy = from_pos.y + from_h / 2.0;
        let to_cy = to_pos.y + to_h / 2.0;
        if from_cy < to_cy {
            (Side::Bottom, Side::Top)
        } else {
            (Side::Top, Side::Bottom)
        }
    } else if from_pos.x < to_pos.x {
        (Side::Right, Side::Left)
    } else {
        (Side::Left, Side::Right)
    }
}

/// Connection point for a table endpoint.
/// Horizontal sides: X = table edge, Y = column row center.
/// Vertical sides: X = table center, Y = table edge.
fn connection_point(
    pos: Position,
    table: &Table,
    side: Side,
    column_name: &str,
) -> (f64, f64) {
    match side {
        Side::Left => (pos.x, pos.y + column_row_y(table, column_name)),
        Side::Right => (pos.x + TABLE_WIDTH, pos.y + column_row_y(table, column_name)),
        Side::Top => (pos.x + TABLE_WIDTH / 2.0, pos.y),
        Side::Bottom => (pos.x + TABLE_WIDTH / 2.0, pos.y + table_height(table)),
    }
}

/// Column Y coordinate relative to table top.
fn column_row_y(table: &Table, column_name: &str) -> f64 {
    let idx = table
        .columns
        .iter()
        .position(|c| c.name == column_name)
        .unwrap_or(0);
    HEADER_HEIGHT + (idx as f64 * ROW_HEIGHT) + (ROW_HEIGHT / 2.0)
}

// ---------------------------------------------------------------------------
// Corridor distribution
// ---------------------------------------------------------------------------

/// Compute mid-point values for each route, distributing overlapping corridors.
fn distribute_corridors(routes: &[Option<RouteInfo>]) -> Vec<f64> {
    let n = routes.len();
    let mut mid_values = vec![0.0_f64; n];
    let mid_spacing = 20.0;

    let mut h_corridors: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut v_corridors: HashMap<i32, Vec<usize>> = HashMap::new();

    for (i, route) in routes.iter().enumerate() {
        let r = match route {
            Some(r) => r,
            None => continue,
        };

        if is_horizontal(r) {
            let base_mid = (r.from_x + r.to_x) / 2.0;
            let key = (base_mid / 10.0).round() as i32;
            h_corridors.entry(key).or_default().push(i);
            mid_values[i] = base_mid;
        } else if is_vertical(r) {
            let base_mid = (r.from_y + r.to_y) / 2.0;
            let key = (base_mid / 10.0).round() as i32;
            v_corridors.entry(key).or_default().push(i);
            mid_values[i] = base_mid;
        }
    }

    distribute_group(&mut mid_values, &h_corridors, mid_spacing);
    distribute_group(&mut mid_values, &v_corridors, mid_spacing);

    mid_values
}

fn distribute_group(
    mid_values: &mut [f64],
    groups: &HashMap<i32, Vec<usize>>,
    spacing: f64,
) {
    for indices in groups.values() {
        if indices.len() <= 1 {
            continue;
        }
        let center = mid_values[indices[0]];
        let count = indices.len();
        for (j, &idx) in indices.iter().enumerate() {
            mid_values[idx] = center + (j as f64 - (count as f64 - 1.0) / 2.0) * spacing;
        }
    }
}

// ---------------------------------------------------------------------------
// Path generation
// ---------------------------------------------------------------------------

/// Build SVG path string for a route.
fn build_path(info: &RouteInfo, mid_value: f64) -> String {
    if is_horizontal(info) {
        format!(
            "M {} {} L {} {} L {} {} L {} {}",
            info.from_x, info.from_y,
            mid_value, info.from_y,
            mid_value, info.to_y,
            info.to_x, info.to_y
        )
    } else if is_vertical(info) {
        format!(
            "M {} {} L {} {} L {} {} L {} {}",
            info.from_x, info.from_y,
            info.from_x, mid_value,
            info.to_x, mid_value,
            info.to_x, info.to_y
        )
    } else {
        // Mixed sides: L-shaped route
        format!(
            "M {} {} L {} {} L {} {}",
            info.from_x, info.from_y,
            info.to_x, info.from_y,
            info.to_x, info.to_y
        )
    }
}

fn is_horizontal(r: &RouteInfo) -> bool {
    matches!(r.from_side, Side::Left | Side::Right)
        && matches!(r.to_side, Side::Left | Side::Right)
}

fn is_vertical(r: &RouteInfo) -> bool {
    matches!(r.from_side, Side::Top | Side::Bottom)
        && matches!(r.to_side, Side::Top | Side::Bottom)
}
