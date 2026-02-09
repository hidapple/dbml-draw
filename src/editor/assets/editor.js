// =========================================================================
// Constants (layout / style)
// =========================================================================
const MIN_TABLE_WIDTH = 160;
const HEADER_HEIGHT = 36;
const ROW_HEIGHT = 28;
const PADDING_X = 12;
const BORDER_RADIUS = 4;
const FONT_SIZE = 14;
const HEADER_FONT_SIZE = 15;

const HEADER_BG = "#3498db";
const HEADER_TEXT = "#ffffff";
const TABLE_BG = "#ffffff";
const TABLE_BORDER = "#cccccc";
const COLUMN_TEXT = "#333333";
const TYPE_TEXT = "#888888";
const PK_COLOR = "#e74c3c";
const RELATION_STROKE = "#666666";
const RELATION_STROKE_WIDTH = 1.5;
const MARKER_LENGTH = 24;
const CANVAS_BG = "#f5f5f5";

// Auto-layout constants
const SPACING_X = 100;
const SPACING_Y = 80;
const START_X = 50;
const START_Y = 50;

// =========================================================================
// State
// =========================================================================
let diagram = null;
let canvas = null;
let ctx = null;

// Pan / zoom
let panX = 0;
let panY = 0;
let scale = 1;
let isPanning = false;
let panStart = { x: 0, y: 0 };

// Drag state
let dragState = null; // { tableIdx, offsetX, offsetY }

// =========================================================================
// Helpers
// =========================================================================
function tableHeight(table) {
  return HEADER_HEIGHT + table.columns.length * ROW_HEIGHT;
}

function tableFullName(table) {
  return table.id.schema + "." + table.id.name;
}

function computeTableWidths() {
  if (!ctx || !diagram) return;
  for (const table of diagram.tables) {
    let maxRowWidth = 0;

    ctx.font = FONT_SIZE + "px monospace";
    for (const col of table.columns) {
      const colText = col.is_pk ? "\u{1F511} " + col.name : col.name;
      const typeText = col.type_raw;
      const gap = 8;
      const rowWidth = ctx.measureText(colText).width + gap + ctx.measureText(typeText).width;
      maxRowWidth = Math.max(maxRowWidth, rowWidth);
    }

    ctx.font = "bold " + HEADER_FONT_SIZE + "px monospace";
    const headerWidth = ctx.measureText(table.id.name).width;

    const contentWidth = Math.max(maxRowWidth, headerWidth);
    table.width = Math.max(MIN_TABLE_WIDTH, contentWidth + 2 * PADDING_X);
  }
}

// =========================================================================
// Coordinate transforms
// =========================================================================
function screenToWorld(clientX, clientY) {
  const rect = canvas.getBoundingClientRect();
  return {
    x: (clientX - rect.left - panX) / scale,
    y: (clientY - rect.top - panY) / scale,
  };
}

// =========================================================================
// Hit test
// =========================================================================
function hitTest(worldX, worldY) {
  // Iterate in reverse so topmost (last drawn) table is found first
  for (let i = diagram.tables.length - 1; i >= 0; i--) {
    const t = diagram.tables[i];
    const p = t.position || { x: 0, y: 0 };
    const w = t.width || MIN_TABLE_WIDTH;
    const h = tableHeight(t);
    if (worldX >= p.x && worldX <= p.x + w &&
        worldY >= p.y && worldY <= p.y + h) {
      return i;
    }
  }
  return -1;
}

// =========================================================================
// Auto-layout (BFS cross layout)
// =========================================================================
function autoLayout() {
  if (!diagram) return;

  const tableIds = diagram.tables.map((_, i) => i);
  if (tableIds.length === 0) return;

  // Build adjacency list (by index)
  const adj = tableIds.map(() => []);
  for (const rel of diagram.relationships) {
    const fi = diagram.tables.findIndex(
      (t) => t.id.schema === rel.from.table_id.schema && t.id.name === rel.from.table_id.name
    );
    const ti = diagram.tables.findIndex(
      (t) => t.id.schema === rel.to.table_id.schema && t.id.name === rel.to.table_id.name
    );
    if (fi >= 0 && ti >= 0) {
      adj[fi].push(ti);
      adj[ti].push(fi);
    }
  }

  // Root = most connected
  let root = 0;
  let maxDeg = 0;
  for (let i = 0; i < adj.length; i++) {
    if (adj[i].length > maxDeg) {
      maxDeg = adj[i].length;
      root = i;
    }
  }

  // BFS with signed grid coords
  const grid = new Map(); // tableIdx -> [col, row]
  const occupied = new Set();
  const visited = new Set();
  const queue = [];
  const directions = [[1,0],[0,1],[-1,0],[0,-1]];

  const key = (c, r) => c + "," + r;

  grid.set(root, [0, 0]);
  occupied.add(key(0, 0));
  visited.add(root);
  queue.push([root, 0, 0]);

  while (queue.length > 0) {
    const [cur, cx, cy] = queue.shift();
    for (const nb of adj[cur]) {
      if (visited.has(nb)) continue;
      visited.add(nb);

      let placed = false;
      for (const [dx, dy] of directions) {
        const nx = cx + dx;
        const ny = cy + dy;
        if (!occupied.has(key(nx, ny))) {
          grid.set(nb, [nx, ny]);
          occupied.add(key(nx, ny));
          queue.push([nb, nx, ny]);
          placed = true;
          break;
        }
      }

      if (!placed) {
        const pos = findNearestEmpty(cx, cy, occupied, key);
        if (pos) {
          grid.set(nb, pos);
          occupied.add(key(pos[0], pos[1]));
          queue.push([nb, pos[0], pos[1]]);
        }
      }
    }
  }

  // Place disconnected tables
  for (let i = 0; i < diagram.tables.length; i++) {
    if (!grid.has(i)) {
      const pos = findNearestEmpty(0, 0, occupied, key);
      if (pos) {
        grid.set(i, pos);
        occupied.add(key(pos[0], pos[1]));
      }
    }
  }

  // Normalize
  let minCol = Infinity, minRow = Infinity;
  for (const [c, r] of grid.values()) {
    minCol = Math.min(minCol, c);
    minRow = Math.min(minRow, r);
  }

  // Column widths and row heights
  const colWidths = {};
  const rowHeights = {};
  for (let i = 0; i < diagram.tables.length; i++) {
    const pos = grid.get(i);
    if (!pos) continue;
    const col = pos[0];
    const row = pos[1];
    const tw = diagram.tables[i].width || MIN_TABLE_WIDTH;
    const h = tableHeight(diagram.tables[i]);
    colWidths[col] = Math.max(colWidths[col] || 0, tw);
    rowHeights[row] = Math.max(rowHeights[row] || 0, h);
  }

  // Assign pixel positions
  for (let i = 0; i < diagram.tables.length; i++) {
    const pos = grid.get(i);
    if (!pos) continue;
    const col = pos[0];
    const row = pos[1];
    let x = START_X;
    for (let c = minCol; c < col; c++) {
      x += (colWidths[c] || MIN_TABLE_WIDTH) + SPACING_X;
    }
    let y = START_Y;
    for (let r = minRow; r < row; r++) {
      y += (rowHeights[r] || 200) + SPACING_Y;
    }
    diagram.tables[i].position = { x, y };
  }
}

function findNearestEmpty(cx, cy, occupied, key) {
  for (let radius = 1; radius < 20; radius++) {
    for (let dx = -radius; dx <= radius; dx++) {
      for (let dy = -radius; dy <= radius; dy++) {
        if (Math.abs(dx) !== radius && Math.abs(dy) !== radius) continue;
        const nx = cx + dx;
        const ny = cy + dy;
        if (!occupied.has(key(nx, ny))) {
          return [nx, ny];
        }
      }
    }
  }
  return null;
}

// =========================================================================
// IE Marker determination
// =========================================================================
function findColumn(table, name) {
  return table.columns.find((c) => c.name === name) || null;
}

function fkIsNullable(columnNames, table) {
  const colName = columnNames[0] || "";
  const col = findColumn(table, colName);
  return col ? col.is_nullable : true;
}

function determineIeMarkers(rel, fromTable, toTable) {
  const rt = rel.relation_type;
  if (rt === "ManyToOne") {
    const n = fkIsNullable(rel.from.column_names, fromTable);
    return [
      n ? "many-optional" : "many-mandatory",
      n ? "one-optional" : "one-mandatory",
    ];
  }
  if (rt === "OneToMany") {
    const n = fkIsNullable(rel.to.column_names, toTable);
    return [
      n ? "one-optional" : "one-mandatory",
      n ? "many-optional" : "many-mandatory",
    ];
  }
  if (rt === "OneToOne") {
    const n = fkIsNullable(rel.from.column_names, fromTable);
    return [
      n ? "one-optional" : "one-mandatory",
      n ? "one-optional" : "one-mandatory",
    ];
  }
  // ManyToMany
  return ["many-optional", "many-optional"];
}

// =========================================================================
// Relationship routing
// =========================================================================
function determineSides(fromPos, fromW, fromH, toPos, toW, toH) {
  const hOverlap =
    fromPos.x < toPos.x + toW && toPos.x < fromPos.x + fromW;
  if (hOverlap) {
    const fromCy = fromPos.y + fromH / 2;
    const toCy = toPos.y + toH / 2;
    return fromCy < toCy ? ["Bottom", "Top"] : ["Top", "Bottom"];
  }
  return fromPos.x < toPos.x ? ["Right", "Left"] : ["Left", "Right"];
}

function columnRowY(table, columnName) {
  const idx = table.columns.findIndex((c) => c.name === columnName);
  const i = idx >= 0 ? idx : 0;
  return HEADER_HEIGHT + i * ROW_HEIGHT + ROW_HEIGHT / 2;
}

function connectionPoint(pos, table, side, columnName) {
  const w = table.width || MIN_TABLE_WIDTH;
  switch (side) {
    case "Left":
      return { x: pos.x, y: pos.y + columnRowY(table, columnName) };
    case "Right":
      return { x: pos.x + w, y: pos.y + columnRowY(table, columnName) };
    case "Top":
      return { x: pos.x + w / 2, y: pos.y };
    case "Bottom":
      return { x: pos.x + w / 2, y: pos.y + tableHeight(table) };
  }
}

function isHorizontal(route) {
  return (
    (route.fromSide === "Left" || route.fromSide === "Right") &&
    (route.toSide === "Left" || route.toSide === "Right")
  );
}

function isVertical(route) {
  return (
    (route.fromSide === "Top" || route.fromSide === "Bottom") &&
    (route.toSide === "Top" || route.toSide === "Bottom")
  );
}

function computeRoutes() {
  const routes = [];
  for (const rel of diagram.relationships) {
    const fi = diagram.tables.findIndex(
      (t) => t.id.schema === rel.from.table_id.schema && t.id.name === rel.from.table_id.name
    );
    const ti = diagram.tables.findIndex(
      (t) => t.id.schema === rel.to.table_id.schema && t.id.name === rel.to.table_id.name
    );

    if (fi < 0 || ti < 0) {
      routes.push(null);
      continue;
    }

    const ft = diagram.tables[fi];
    const tt = diagram.tables[ti];
    const fp = ft.position || { x: 0, y: 0 };
    const tp = tt.position || { x: 0, y: 0 };
    const fw = ft.width || MIN_TABLE_WIDTH;
    const tw = tt.width || MIN_TABLE_WIDTH;

    const [fromSide, toSide] = determineSides(fp, fw, tableHeight(ft), tp, tw, tableHeight(tt));
    const fromCol = rel.from.column_names[0] || "";
    const toCol = rel.to.column_names[0] || "";
    const from = connectionPoint(fp, ft, fromSide, fromCol);
    const to = connectionPoint(tp, tt, toSide, toCol);

    routes.push({
      fromIdx: fi, toIdx: ti,
      fromSide, toSide,
      fromX: from.x, fromY: from.y,
      toX: to.x, toY: to.y,
    });
  }
  return routes;
}

function distributeConnectionPoints(routes) {
  // Group route indices by (tableIdx, side) for both from and to endpoints
  const groups = {}; // key -> [{ routeIdx, endpoint: "from"|"to" }]

  for (let i = 0; i < routes.length; i++) {
    const r = routes[i];
    if (!r) continue;
    const fromKey = r.fromIdx + ":" + r.fromSide;
    const toKey = r.toIdx + ":" + r.toSide;
    (groups[fromKey] = groups[fromKey] || []).push({ routeIdx: i, endpoint: "from" });
    (groups[toKey] = groups[toKey] || []).push({ routeIdx: i, endpoint: "to" });
  }

  for (const entries of Object.values(groups)) {
    if (entries.length <= 1) continue;

    const count = entries.length;
    // Use the first entry to determine table and side
    const first = routes[entries[0].routeIdx];
    const tableIdx = entries[0].endpoint === "from" ? first.fromIdx : first.toIdx;
    const side = entries[0].endpoint === "from" ? first.fromSide : first.toSide;
    const table = diagram.tables[tableIdx];
    const pos = table.position || { x: 0, y: 0 };
    const w = table.width || MIN_TABLE_WIDTH;
    const h = tableHeight(table);

    if (side === "Left" || side === "Right") {
      // Distribute Y coordinates within [pos.y + HEADER_HEIGHT, pos.y + h]
      const minY = pos.y + HEADER_HEIGHT;
      const rangeY = h - HEADER_HEIGHT;
      for (let j = 0; j < count; j++) {
        const e = entries[j];
        const r = routes[e.routeIdx];
        const newY = minY + rangeY * (j + 1) / (count + 1);
        if (e.endpoint === "from") r.fromY = newY;
        else r.toY = newY;
      }
    } else {
      // Top or Bottom: distribute X coordinates within [pos.x, pos.x + w]
      for (let j = 0; j < count; j++) {
        const e = entries[j];
        const r = routes[e.routeIdx];
        const newX = pos.x + w * (j + 1) / (count + 1);
        if (e.endpoint === "from") r.fromX = newX;
        else r.toX = newX;
      }
    }
  }
}

function sideAngle(side) {
  switch (side) {
    case "Right":  return 0;
    case "Left":   return Math.PI;
    case "Bottom": return Math.PI / 2;
    case "Top":    return -Math.PI / 2;
  }
}

function drawRelationshipPath(ctx, info) {
  const { fromX, fromY, toX, toY, fromSide, toSide } = info;

  // Offset points past the markers so bezier doesn't overlap with marker symbols
  const fa = sideAngle(fromSide);
  const ta = sideAngle(toSide);
  const oFromX = fromX + Math.cos(fa) * MARKER_LENGTH;
  const oFromY = fromY + Math.sin(fa) * MARKER_LENGTH;
  const oToX = toX + Math.cos(ta) * MARKER_LENGTH;
  const oToY = toY + Math.sin(ta) * MARKER_LENGTH;

  ctx.beginPath();
  ctx.moveTo(fromX, fromY);
  ctx.lineTo(oFromX, oFromY);

  if (isHorizontal(info)) {
    const midX = (oFromX + oToX) / 2;
    ctx.bezierCurveTo(midX, oFromY, midX, oToY, oToX, oToY);
  } else if (isVertical(info)) {
    const midY = (oFromY + oToY) / 2;
    ctx.bezierCurveTo(oFromX, midY, oToX, midY, oToX, oToY);
  } else {
    ctx.bezierCurveTo(oToX, oFromY, oToX, oToY, oToX, oToY);
  }

  ctx.lineTo(toX, toY);

  ctx.strokeStyle = RELATION_STROKE;
  ctx.lineWidth = RELATION_STROKE_WIDTH;
  ctx.stroke();
}

// =========================================================================
// Canvas drawing — IE Markers
// =========================================================================
function drawMarker(ctx, x, y, angle, markerType) {
  ctx.save();
  ctx.translate(x, y);
  ctx.rotate(angle);

  ctx.strokeStyle = RELATION_STROKE;
  ctx.lineWidth = RELATION_STROKE_WIDTH;
  ctx.fillStyle = "white";

  switch (markerType) {
    case "one-mandatory":
      // || two vertical lines
      ctx.beginPath();
      ctx.moveTo(6, -8); ctx.lineTo(6, 8);
      ctx.moveTo(12, -8); ctx.lineTo(12, 8);
      ctx.stroke();
      break;
    case "one-optional":
      // |O vertical line + circle
      ctx.beginPath();
      ctx.moveTo(6, -8); ctx.lineTo(6, 8);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(14, 0, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      break;
    case "many-mandatory":
      // |< vertical line + crow's foot
      ctx.beginPath();
      ctx.moveTo(16, -8); ctx.lineTo(16, 8);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(12, 0); ctx.lineTo(0, -8);
      ctx.moveTo(12, 0); ctx.lineTo(0, 8);
      ctx.stroke();
      break;
    case "many-optional":
      // O< circle + crow's foot
      ctx.beginPath();
      ctx.arc(18, 0, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(12, 0); ctx.lineTo(0, -8);
      ctx.moveTo(12, 0); ctx.lineTo(0, 8);
      ctx.stroke();
      break;
  }

  ctx.restore();
}

// =========================================================================
// Canvas drawing — Relationships
// =========================================================================
function drawRelationships() {
  const routes = computeRoutes();
  distributeConnectionPoints(routes);

  for (let i = 0; i < diagram.relationships.length; i++) {
    const info = routes[i];
    if (!info) continue;

    const rel = diagram.relationships[i];
    const fromTable = diagram.tables[info.fromIdx];
    const toTable = diagram.tables[info.toIdx];
    const [fromMarker, toMarker] = determineIeMarkers(rel, fromTable, toTable);

    // Draw bezier path with straight segments at endpoints
    drawRelationshipPath(ctx, info);

    // Markers aligned with the straight segments
    const startAngle = sideAngle(info.fromSide);
    drawMarker(ctx, info.fromX, info.fromY, startAngle, fromMarker);

    const endAngle = sideAngle(info.toSide);
    drawMarker(ctx, info.toX, info.toY, endAngle, toMarker);
  }
}

// =========================================================================
// Canvas drawing — Tables
// =========================================================================
function drawTable(table) {
  const pos = table.position || { x: 0, y: 0 };
  const w = table.width || MIN_TABLE_WIDTH;
  const h = tableHeight(table);

  // Shadow
  ctx.save();
  ctx.shadowColor = "rgba(0, 0, 0, 0.1)";
  ctx.shadowBlur = 6;
  ctx.shadowOffsetX = 0;
  ctx.shadowOffsetY = 2;

  // Border rect with rounded corners
  ctx.beginPath();
  ctx.roundRect(pos.x, pos.y, w, h, BORDER_RADIUS);
  ctx.fillStyle = TABLE_BG;
  ctx.fill();
  ctx.strokeStyle = TABLE_BORDER;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.restore();

  // Header background (top rounded, bottom square)
  ctx.beginPath();
  ctx.roundRect(pos.x, pos.y, w, HEADER_HEIGHT, [BORDER_RADIUS, BORDER_RADIUS, 0, 0]);
  ctx.fillStyle = HEADER_BG;
  ctx.fill();

  // Header text
  ctx.font = "bold " + HEADER_FONT_SIZE + "px monospace";
  ctx.fillStyle = HEADER_TEXT;
  ctx.textBaseline = "middle";
  ctx.fillText(table.id.name, pos.x + PADDING_X, pos.y + HEADER_HEIGHT / 2);

  // Separator line
  ctx.beginPath();
  ctx.moveTo(pos.x, pos.y + HEADER_HEIGHT);
  ctx.lineTo(pos.x + w, pos.y + HEADER_HEIGHT);
  ctx.strokeStyle = TABLE_BORDER;
  ctx.lineWidth = 1;
  ctx.stroke();

  // Columns
  ctx.font = FONT_SIZE + "px monospace";
  ctx.textBaseline = "middle";
  for (let i = 0; i < table.columns.length; i++) {
    const col = table.columns[i];
    const rowY = pos.y + HEADER_HEIGHT + i * ROW_HEIGHT;

    // Column name
    const colText = col.is_pk ? "\u{1F511} " + col.name : col.name;
    ctx.fillStyle = col.is_pk ? PK_COLOR : COLUMN_TEXT;
    ctx.textAlign = "left";
    ctx.fillText(colText, pos.x + PADDING_X, rowY + ROW_HEIGHT / 2);

    // Type (right aligned)
    ctx.fillStyle = TYPE_TEXT;
    ctx.textAlign = "right";
    ctx.fillText(col.type_raw, pos.x + w - PADDING_X, rowY + ROW_HEIGHT / 2);
  }

  // Reset text align
  ctx.textAlign = "left";
}

// =========================================================================
// Main render loop
// =========================================================================
function render() {
  if (!diagram || !ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;

  // Clear
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);

  // Background
  ctx.fillStyle = CANVAS_BG;
  ctx.fillRect(0, 0, w, h);

  // Apply pan/zoom transform
  ctx.translate(panX, panY);
  ctx.scale(scale, scale);

  // Draw relationships (below tables)
  drawRelationships();

  // Draw tables (on top)
  for (const table of diagram.tables) {
    drawTable(table);
  }
}

// =========================================================================
// Canvas sizing
// =========================================================================
function resizeCanvas() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * dpr;
  canvas.height = canvas.clientHeight * dpr;
  render();
}

// =========================================================================
// Fit to View
// =========================================================================
function fitToView() {
  if (!diagram || diagram.tables.length === 0) return;

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const t of diagram.tables) {
    const p = t.position || { x: 0, y: 0 };
    const tw = t.width || MIN_TABLE_WIDTH;
    const h = tableHeight(t);
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x + tw);
    maxY = Math.max(maxY, p.y + h);
  }

  const padding = 50;
  const contentW = maxX - minX + padding * 2;
  const contentH = maxY - minY + padding * 2;

  const canvasW = canvas.clientWidth;
  const canvasH = canvas.clientHeight;

  scale = Math.min(canvasW / contentW, canvasH / contentH, 2);
  panX = (canvasW - contentW * scale) / 2 - minX * scale + padding * scale;
  panY = (canvasH - contentH * scale) / 2 - minY * scale + padding * scale;

  render();
}

// =========================================================================
// Interaction — Pointer events
// =========================================================================
function onPointerDown(e) {
  const world = screenToWorld(e.clientX, e.clientY);
  const idx = hitTest(world.x, world.y);

  if (idx >= 0) {
    // Start dragging a table
    const pos = diagram.tables[idx].position || { x: 0, y: 0 };
    dragState = {
      tableIdx: idx,
      offsetX: world.x - pos.x,
      offsetY: world.y - pos.y,
    };
    canvas.style.cursor = "grabbing";
    canvas.setPointerCapture(e.pointerId);
    e.preventDefault();
    return;
  }

  // Start panning
  isPanning = true;
  panStart = { x: e.clientX, y: e.clientY };
  canvas.style.cursor = "grabbing";
  canvas.setPointerCapture(e.pointerId);
  e.preventDefault();
}

function onPointerMove(e) {
  if (dragState) {
    const world = screenToWorld(e.clientX, e.clientY);
    const newX = world.x - dragState.offsetX;
    const newY = world.y - dragState.offsetY;
    diagram.tables[dragState.tableIdx].position = { x: newX, y: newY };
    render();
    return;
  }

  if (isPanning) {
    const dx = e.clientX - panStart.x;
    const dy = e.clientY - panStart.y;
    panX += dx;
    panY += dy;
    panStart = { x: e.clientX, y: e.clientY };
    render();
  }
}

function onPointerUp(e) {
  if (dragState) {
    canvas.style.cursor = "";
    const table = diagram.tables[dragState.tableIdx];
    const pos = table.position || { x: 0, y: 0 };
    window.ipc.postMessage(
      JSON.stringify({
        type: "table_moved",
        table_id: tableFullName(table),
        x: pos.x,
        y: pos.y,
      })
    );
    dragState = null;
    return;
  }

  if (isPanning) {
    isPanning = false;
    canvas.style.cursor = "";
  }
}

// =========================================================================
// Interaction — Zoom (wheel)
// =========================================================================
function onWheel(e) {
  e.preventDefault();
  const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;

  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;

  // Zoom centered on cursor
  panX = mx - (mx - panX) * zoomFactor;
  panY = my - (my - panY) * zoomFactor;
  scale *= zoomFactor;

  render();
}

// =========================================================================
// Toolbar handlers
// =========================================================================
function onExportPng() {
  // Create an offscreen canvas at 1x scale for clean export
  const offscreen = document.createElement("canvas");
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const t of diagram.tables) {
    const p = t.position || { x: 0, y: 0 };
    const tw = t.width || MIN_TABLE_WIDTH;
    const h = tableHeight(t);
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x + tw);
    maxY = Math.max(maxY, p.y + h);
  }

  const padding = 50;
  const exportW = maxX - minX + padding * 2;
  const exportH = maxY - minY + padding * 2;
  const exportScale = 2; // 2x for crisp export

  offscreen.width = exportW * exportScale;
  offscreen.height = exportH * exportScale;

  const offCtx = offscreen.getContext("2d");
  offCtx.scale(exportScale, exportScale);
  offCtx.translate(-minX + padding, -minY + padding);

  // Background
  offCtx.fillStyle = CANVAS_BG;
  offCtx.fillRect(minX - padding, minY - padding, exportW, exportH);

  // Temporarily swap ctx
  const savedCtx = ctx;
  ctx = offCtx;
  drawRelationships();
  for (const table of diagram.tables) {
    drawTable(table);
  }
  ctx = savedCtx;

  offscreen.toBlob(function (blob) {
    const reader = new FileReader();
    reader.onloadend = function () {
      window.ipc.postMessage(
        JSON.stringify({ type: "export_png", data_url: reader.result })
      );
    };
    reader.readAsDataURL(blob);
  }, "image/png");
}

function onResetLayout() {
  if (!diagram) return;

  // Clear all positions
  for (const t of diagram.tables) {
    t.position = null;
  }

  // Re-apply auto layout
  autoLayout();
  render();

  // Save new layout to Rust
  const tables = {};
  for (const t of diagram.tables) {
    if (t.position) {
      tables[tableFullName(t)] = { x: t.position.x, y: t.position.y };
    }
  }
  window.ipc.postMessage(
    JSON.stringify({ type: "save_layout", tables: tables })
  );
}

function onFitToView() {
  fitToView();
}

// =========================================================================
// IPC callbacks (called from Rust via evaluate_script)
// =========================================================================
function showToast(message) {
  const toast = document.getElementById("toast");
  if (!toast) return;
  toast.textContent = message;
  toast.classList.add("show");
  setTimeout(() => { toast.classList.remove("show"); }, 3000);
}

window.__onExportComplete = function (path) {
  showToast(path ? "Exported: " + path : "Export failed");
};

// =========================================================================
// Init
// =========================================================================
function init() {
  // Toolbar
  document.getElementById("btn-export").addEventListener("click", onExportPng);
  document.getElementById("btn-reset").addEventListener("click", onResetLayout);
  document.getElementById("btn-fit").addEventListener("click", onFitToView);

  // Canvas setup
  canvas = document.getElementById("canvas");
  ctx = canvas.getContext("2d");

  resizeCanvas();
  window.addEventListener("resize", resizeCanvas);

  // Pointer events
  canvas.addEventListener("pointerdown", onPointerDown);
  canvas.addEventListener("pointermove", onPointerMove);
  canvas.addEventListener("pointerup", onPointerUp);
  canvas.addEventListener("wheel", onWheel, { passive: false });

  // Load initial data
  if (window.__INITIAL_DIAGRAM) {
    diagram = window.__INITIAL_DIAGRAM;

    // Compute per-table widths from content
    computeTableWidths();

    // Auto-layout tables without positions
    const needsLayout = diagram.tables.some((t) => !t.position);
    if (needsLayout) {
      autoLayout();
    }

    fitToView();
  }
}

document.addEventListener("DOMContentLoaded", init);
