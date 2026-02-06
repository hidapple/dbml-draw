//! Types for layout.toml serialization.
//!
//! Example layout.toml:
//! ```toml
//! [meta]
//! version = 1
//! source = "schema.dbml"
//!
//! [tables."public.users"]
//! x = 100.0
//! y = 200.0
//!
//! [tables."public.posts"]
//! x = 450.0
//! y = 200.0
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entire layout file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {
    pub meta: LayoutMeta,
    pub tables: HashMap<String, TableLayout>, // Key is "schema.table" (e.g., "public.users")
}

/// Metadata about the layout file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutMeta {
    pub version: u32,
    pub source: String,
}

/// Position of a single table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableLayout {
    pub x: f64,
    pub y: f64,
}
