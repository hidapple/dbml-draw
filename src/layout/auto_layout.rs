use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{Diagram, Position, TableId};

const TABLE_WIDTH: f64 = 260.0;
const HEADER_HEIGHT: f64 = 36.0;
const ROW_HEIGHT: f64 = 28.0;
const SPACING_X: f64 = 100.0;
const SPACING_Y: f64 = 80.0;
const START_X: f64 = 50.0;
const START_Y: f64 = 50.0;

/// Cross-shaped BFS layout.
///
/// 1. Pick the most-connected table as root and place it at grid (0, 0).
/// 2. BFS outward, placing neighbors in the four cardinal directions (right, down, left, up).
/// 3. Normalize grid coordinates so all positions are non-negative.
pub fn grid_layout(diagram: &mut Diagram) {
    let table_ids: Vec<TableId> = diagram
        .tables
        .iter()
        .filter(|t| t.position.is_none())
        .map(|t| t.id.clone())
        .collect();

    if table_ids.is_empty() {
        return;
    }

    // Build adjacency list
    let mut adjacency: HashMap<TableId, Vec<TableId>> = HashMap::new();
    for tid in &table_ids {
        adjacency.entry(tid.clone()).or_default();
    }
    for rel in &diagram.relationships {
        let from = &rel.from.table_id;
        let to = &rel.to.table_id;
        if adjacency.contains_key(from) && adjacency.contains_key(to) {
            adjacency.get_mut(from).unwrap().push(to.clone());
            adjacency.get_mut(to).unwrap().push(from.clone());
        }
    }

    // Root = most connected table
    let root = table_ids
        .iter()
        .max_by_key(|t| adjacency.get(*t).map(|v| v.len()).unwrap_or(0))
        .unwrap()
        .clone();

    // BFS with signed grid coordinates
    let grid = bfs_cross_layout(&root, &adjacency);

    // Normalize to non-negative
    let min_col = grid.values().map(|&(c, _)| c).min().unwrap_or(0);
    let min_row = grid.values().map(|&(_, r)| r).min().unwrap_or(0);

    // Compute max height per grid row
    let mut row_heights: HashMap<i32, f64> = HashMap::new();
    for table in diagram.tables.iter() {
        if let Some(&(_, row)) = grid.get(&table.id) {
            let h = HEADER_HEIGHT + table.columns.len() as f64 * ROW_HEIGHT;
            let entry = row_heights.entry(row).or_insert(0.0);
            *entry = entry.max(h);
        }
    }

    // Assign pixel positions
    for table in &mut diagram.tables {
        if table.position.is_some() {
            continue;
        }
        if let Some(&(col, row)) = grid.get(&table.id) {
            let x = START_X + (col - min_col) as f64 * (TABLE_WIDTH + SPACING_X);
            let mut y = START_Y;
            for r in min_row..row {
                y += row_heights.get(&r).copied().unwrap_or(200.0) + SPACING_Y;
            }
            table.position = Some(Position { x, y });
        }
    }
}

/// BFS from root, placing neighbors in cardinal directions (right, down, left, up).
/// Uses signed coordinates to allow placement in all directions from root.
fn bfs_cross_layout(
    root: &TableId,
    adjacency: &HashMap<TableId, Vec<TableId>>,
) -> HashMap<TableId, (i32, i32)> {
    let mut grid: HashMap<TableId, (i32, i32)> = HashMap::new();
    let mut occupied: HashSet<(i32, i32)> = HashSet::new();
    let mut visited: HashSet<TableId> = HashSet::new();
    let mut queue: VecDeque<(TableId, i32, i32)> = VecDeque::new();

    let directions: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

    grid.insert(root.clone(), (0, 0));
    occupied.insert((0, 0));
    visited.insert(root.clone());
    queue.push_back((root.clone(), 0, 0));

    while let Some((current, cx, cy)) = queue.pop_front() {
        let neighbors = match adjacency.get(&current) {
            Some(n) => n,
            None => continue,
        };

        for neighbor in neighbors {
            if visited.contains(neighbor) {
                continue;
            }
            visited.insert(neighbor.clone());

            let mut placed = false;
            for &(dx, dy) in &directions {
                let nx = cx + dx;
                let ny = cy + dy;
                if !occupied.contains(&(nx, ny)) {
                    grid.insert(neighbor.clone(), (nx, ny));
                    occupied.insert((nx, ny));
                    queue.push_back((neighbor.clone(), nx, ny));
                    placed = true;
                    break;
                }
            }

            if !placed {
                if let Some((fx, fy)) = find_nearest_empty(cx, cy, &occupied) {
                    grid.insert(neighbor.clone(), (fx, fy));
                    occupied.insert((fx, fy));
                    queue.push_back((neighbor.clone(), fx, fy));
                }
            }
        }
    }

    // Place disconnected tables
    for tid in adjacency.keys() {
        if !grid.contains_key(tid) {
            if let Some((fx, fy)) = find_nearest_empty(0, 0, &occupied) {
                grid.insert(tid.clone(), (fx, fy));
                occupied.insert((fx, fy));
            }
        }
    }

    grid
}

fn find_nearest_empty(cx: i32, cy: i32, occupied: &HashSet<(i32, i32)>) -> Option<(i32, i32)> {
    for radius in 1i32..20 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let nx = cx + dx;
                let ny = cy + dy;
                if !occupied.contains(&(nx, ny)) {
                    return Some((nx, ny));
                }
            }
        }
    }
    None
}
